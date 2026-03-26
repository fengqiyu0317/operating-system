//! batch subsystem

use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;
use core::slice;
use lazy_static::*;

// 应用名称表，由 build.rs 自动生成
unsafe extern "C" {
    fn _app_names();
}

// ==================== 异常统计相关 ====================

/// 异常类型枚举
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExceptionType {
    /// 指令地址未对齐异常
    InstructionAddressMisaligned,
    /// 指令访问错误
    InstructionAccessFault,
    /// 非法指令
    IllegalInstruction,
    /// 加载地址未对齐
    LoadAddressMisaligned,
    /// 加载访问错误
    LoadAccessFault,
    /// 存储地址未对齐
    StoreAddressMisaligned,
    /// 存储错误
    StoreFault,
    /// 存储页错误
    StorePageFault,
    /// 未知异常
    Unknown,
}

impl ExceptionType {
    /// 获取异常类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            ExceptionType::InstructionAddressMisaligned => "InstructionAddressMisaligned",
            ExceptionType::InstructionAccessFault => "InstructionAccessFault",
            ExceptionType::IllegalInstruction => "IllegalInstruction",
            ExceptionType::LoadAddressMisaligned => "LoadAddressMisaligned",
            ExceptionType::LoadAccessFault => "LoadAccessFault",
            ExceptionType::StoreAddressMisaligned => "StoreAddressMisaligned",
            ExceptionType::StoreFault => "StoreFault",
            ExceptionType::StorePageFault => "StorePageFault",
            ExceptionType::Unknown => "Unknown",
        }
    }
}

/// 异常记录
#[derive(Clone, Copy, Debug)]
pub struct ExceptionRecord {
    /// 异常类型
    pub exception_type: ExceptionType,
    /// 出错指令地址
    pub sepc: usize,
    /// 相关内存地址
    pub stval: usize,
}

// 每个应用最多记录的异常数量
const MAX_EXCEPTIONS_PER_APP: usize = 16;

lazy_static! {
    /// 每个应用的异常记录数组
    /// APP_EXCEPTIONS[app_id][index] 存储该应用的异常记录
    static ref APP_EXCEPTIONS: UPSafeCell<[[ExceptionRecord; MAX_EXCEPTIONS_PER_APP]; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([[ExceptionRecord {
            exception_type: ExceptionType::Unknown,
            sepc: 0,
            stval: 0,
        }; MAX_EXCEPTIONS_PER_APP]; MAX_APP_NUM])
    };

    /// 每个应用的异常计数器
    static ref APP_EXCEPTION_COUNTS: UPSafeCell<[usize; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([0; MAX_APP_NUM])
    };
}

// ==================== 异常统计相关结束 ====================

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

// 系统调用统计相关常量
/// 系统调用编号的最大值，用于统计数组大小
pub const MAX_SYSCALL_ID: usize = 256;

/// 获取当前时间（CPU cycles）
fn get_time() -> usize {
    unsafe {
        let time: usize;
        asm!(
            "rdtime {}",
            out(reg) time,
        );
        time
    }
}

/// 记录应用开始时间
pub fn record_app_start_time() {
    *APP_START_TIME.exclusive_access() = get_time();
}

/// 记录应用结束时间并存储
pub fn record_app_end_time(app_id: usize) {
    let end_time = get_time();
    let start_time = *APP_START_TIME.exclusive_access();
    let exec_time = end_time - start_time;
    APP_EXEC_TIME.exclusive_access()[app_id] = exec_time;
}

/// 获取应用执行时间（CPU cycles）
pub fn get_app_exec_time(app_id: usize) -> usize {
    APP_EXEC_TIME.exclusive_access()[app_id]
}

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

// 获取指定应用 ID 的名称
fn get_app_name(app_id: usize) -> &'static str {
    unsafe {
        let mut name_ptr = _app_names as usize;
        // 跳过前面的 app_id 个字符串
        for _ in 0..app_id {
            while *(name_ptr as *const u8) != 0 {
                name_ptr += 1;
            }
            name_ptr += 1; // 跳过 null 终止符
        }
        // 找到当前字符串的结束位置
        let start = name_ptr;
        let mut end = name_ptr;
        while *(end as *const u8) != 0 {
            end += 1;
        }
        let slice = slice::from_raw_parts(start as *const u8, end - start);
        core::str::from_utf8_unchecked(slice)
    }
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}", app_id);
        unsafe {
            // clear app area
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
            let app_src = core::slice::from_raw_parts(
                self.app_start[app_id] as *const u8,
                self.app_start[app_id + 1] - self.app_start[app_id],
            );
            let app_dst =
                core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src);
            // Memory fence about fetching the instruction memory
            // It is guaranteed that a subsequent instruction fetch must
            // observes all previous writes to the instruction memory.
            // Therefore, fence.i must be executed after we have loaded
            // the code of the next app into the instruction memory.
            // See also: riscv non-priv spec chapter 3, 'Zifencei' extension.
            asm!("fence.i");
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    /// 获取当前正在运行的应用 ID
    /// 注意：current_app 在 move_to_next_app 后已经指向下一个应用
    /// 所以这里需要返回 current_app - 1
    pub fn get_running_app(&self) -> usize {
        self.current_app - 1
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            unsafe extern "C" {
                safe fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };

    // 系统调用统计数组：SYSCALL_STATS[app_id][syscall_id] = count
    static ref SYSCALL_STATS: UPSafeCell<[[usize; MAX_SYSCALL_ID]; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([[0; MAX_SYSCALL_ID]; MAX_APP_NUM])
    };

    /// 应用执行时间统计（单位：CPU cycles）
    static ref APP_EXEC_TIME: UPSafeCell<[usize; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([0; MAX_APP_NUM])
    };

    /// 应用开始时间（临时记录）
    static ref APP_START_TIME: UPSafeCell<usize> = unsafe {
        UPSafeCell::new(0)
    };
}

/// init batch subsystem
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// run next app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    app_manager.load_app(current_app);
    app_manager.move_to_next_app();
    drop(app_manager);
    // 记录应用开始时间
    record_app_start_time();
    // before this we have to drop local variables related to resources manually
    // and release the resources
    unsafe extern "C" {
        unsafe fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}

/// 获取当前应用的信息 (id, name)
pub fn get_current_task_info() -> (usize, &'static str) {
    let app_manager = APP_MANAGER.exclusive_access();
    let app_id = app_manager.get_running_app();
    let app_name = get_app_name(app_id);
    (app_id, app_name)
}

/// 记录系统调用
pub fn record_syscall(syscall_id: usize) {
    let mut stats = SYSCALL_STATS.exclusive_access();
    let app_id = APP_MANAGER.exclusive_access().get_running_app();
    if app_id < MAX_APP_NUM && syscall_id < MAX_SYSCALL_ID {
        stats[app_id][syscall_id] += 1;
    }
}

/// 输出系统调用统计信息
pub fn print_syscall_stats(app_id: usize) {
    let stats = SYSCALL_STATS.exclusive_access();
    let app_name = get_app_name(app_id);
    println!("\n[kernel] === Syscall Statistics for {} (app_{}) ===", app_name, app_id);

    let mut total = 0;
    for id in 0..MAX_SYSCALL_ID {
        let count = stats[app_id][id];
        if count > 0 {
            println!("[kernel]   Syscall {}: {} calls", id, count);
            total += count;
        }
    }
    println!("[kernel]   Total: {} syscalls\n", total);
}

/// 获取当前正在运行的应用 ID（供其他模块使用）
pub fn get_current_app_id() -> usize {
    APP_MANAGER.exclusive_access().get_running_app()
}

// ==================== 异常统计相关函数 ====================

/// 读取指定地址的指令（用于调试）
pub fn fetch_instruction(sepc: usize) -> u32 {
    unsafe { (sepc as *const u32).read_volatile() }
}

/// 记录异常
pub fn record_exception(exception_type: ExceptionType, sepc: usize, stval: usize) {
    let app_id = get_current_app_id();
    if app_id < MAX_APP_NUM {
        let mut counts = APP_EXCEPTION_COUNTS.exclusive_access();
        let mut exceptions = APP_EXCEPTIONS.exclusive_access();
        let count = counts[app_id];
        if count < MAX_EXCEPTIONS_PER_APP {
            exceptions[app_id][count] = ExceptionRecord {
                exception_type,
                sepc,
                stval,
            };
            counts[app_id] = count + 1;
        }
    }
}

/// 打印异常详细信息
pub fn print_exception_details(exception_type: ExceptionType, sepc: usize, stval: usize) {
    let app_id = get_current_app_id();
    let app_name = get_app_name(app_id);
    let instruction = fetch_instruction(sepc);

    println!("[kernel] ==================== Exception Details ====================");
    println!("[kernel] Application: {} (app_{})", app_name, app_id);
    println!("[kernel] Exception Type: {}", exception_type.name());
    println!("[kernel] Fault Instruction Address (sepc): {:#x}", sepc);
    println!("[kernel] Fault Address (stval): {:#x}", stval);
    println!("[kernel] Instruction: {:#x}", instruction);
    println!("[kernel] =========================================================");
}

/// 打印应用的异常汇总
pub fn print_exception_summary(app_id: usize) {
    let counts = APP_EXCEPTION_COUNTS.exclusive_access();
    let exceptions = APP_EXCEPTIONS.exclusive_access();
    let exception_count = counts[app_id];

    if exception_count == 0 {
        return;
    }

    let app_name = get_app_name(app_id);
    println!("\n[kernel] === Exception Summary for {} (app_{}) ===", app_name, app_id);
    println!("[kernel] Total exceptions: {}", exception_count);

    // 统计每种异常的数量
    let mut type_counts = [0usize; 9];
    for i in 0..exception_count {
        match exceptions[app_id][i].exception_type {
            ExceptionType::InstructionAddressMisaligned => type_counts[0] += 1,
            ExceptionType::InstructionAccessFault => type_counts[1] += 1,
            ExceptionType::IllegalInstruction => type_counts[2] += 1,
            ExceptionType::LoadAddressMisaligned => type_counts[3] += 1,
            ExceptionType::LoadAccessFault => type_counts[4] += 1,
            ExceptionType::StoreAddressMisaligned => type_counts[5] += 1,
            ExceptionType::StoreFault => type_counts[6] += 1,
            ExceptionType::StorePageFault => type_counts[7] += 1,
            ExceptionType::Unknown => type_counts[8] += 1,
        }
    }

    // 打印异常分类统计
    if type_counts[2] > 0 {
        println!("[kernel]   IllegalInstruction: {}", type_counts[2]);
    }
    if type_counts[0] > 0 {
        println!("[kernel]   InstructionAddressMisaligned: {}", type_counts[0]);
    }
    if type_counts[1] > 0 {
        println!("[kernel]   InstructionAccessFault: {}", type_counts[1]);
    }
    if type_counts[3] > 0 {
        println!("[kernel]   LoadAddressMisaligned: {}", type_counts[3]);
    }
    if type_counts[4] > 0 {
        println!("[kernel]   LoadAccessFault: {}", type_counts[4]);
    }
    if type_counts[5] > 0 {
        println!("[kernel]   StoreAddressMisaligned: {}", type_counts[5]);
    }
    if type_counts[6] > 0 {
        println!("[kernel]   StoreFault: {}", type_counts[6]);
    }
    if type_counts[7] > 0 {
        println!("[kernel]   StorePageFault: {}", type_counts[7]);
    }
    if type_counts[8] > 0 {
        println!("[kernel]   Unknown: {}", type_counts[8]);
    }
    println!("[kernel] ");
}

// ==================== 异常统计相关函数结束 ====================
