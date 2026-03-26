# rCore-Tutorial 学习记录

## 第一章 → 第二章（ch2）主要变化

### 1. 链接器脚本 (linker.ld)

**用户态链接器脚本** (`user/src/linker.ld`):
- 设置基地址 `BASE_ADDRESS = 0x80400000`
- 定义段布局：`.text` → `.rodata` → `.data` → `.bss`
- `ENTRY(_start)` 指定入口点
- `/DISCARD/` 丢弃调试信息

**关键点**：
- `ENTRY(_start)` 指定入口符号
- `*(.text.entry)` 确保 `_start` 在代码段最前面
- `#[link_section = ".text.entry"]` 把 `_start` 函数放到该段

### 2. TrapContext 结构体

**位置**: `os/src/trap/context.rs`

```rust
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],    // 通用寄存器 x0-x31
    pub sstatus: Sstatus,  // 保存特权级状态
    pub sepc: usize,       // 保存发生 trap 时的 PC
}
```

**作用**：用户态↔内核态切换时保存/恢复处理器状态

**使用时机**：
- 保存：用户程序 trap 进入内核时
- 恢复：从内核返回用户程序时

### 3. UPSafeCell<T>

**位置**: `os/src/sync/up.rs`

```rust
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}
unsafe impl<T> Sync for UPSafeCell<T> {}
```

**作用**：单处理器内核中的内部可变性类型

**设计原理**：
- `RefCell` 提供运行时借用检查
- `unsafe impl Sync` 手动标记为线程安全（单处理器环境）
- 后续多核版本会被真正的 `Mutex` 替代

**关键方法**：
- `new()`: 创建新实例
- `exclusive_access()`: 获取可变引用

### 4. 栈的布局

#### 内核栈 (KERNEL_STACK)
- 大小：8KB
- 位置：`batch.rs` 静态变量
- 用途：
  1. `push_context()` 存放 TrapContext
  2. `__alltraps` 保存用户寄存器
  3. 内核函数的栈帧

#### 用户栈 (USER_STACK)
- 大小：8KB
- 用途：用户程序运行时使用
- 配置方式：通过 `app_init_context` 设置到 `TrapContext.x[2]`

#### 启动栈 (boot_stack)
- 大小：16KB
- 位置：`entry.asm`
- 用途：早期启动（`_start` → `rust_main`）

### 5. trap 处理流程

#### 初始化 (`trap::init()`)
```rust
stvec::write(__alltraps as usize, TrapMode::Direct);
```
- 把 `__alltraps` 地址写入 `stvec` CSR

#### trap 进入 (`__alltraps`)
```assembly
csrrw sp, sscratch, sp    # 交换 sp 和 sscratch
addi sp, sp, -34*8        # 分配 TrapContext 空间
# 保存寄存器...
call trap_handler
```

#### trap 返回 (`__restore`)
```assembly
# 恢复寄存器...
addi sp, sp, 34*8         # 释放 TrapContext
csrrw sp, sscratch, sp    # 交换回用户栈
sret                      # 返回用户态
```

### 6. sscratch 寄存器的初始化

**关键机制**：`sscratch` 在 `__restore` 中通过 `csrrw sp, sscratch, sp` 初始化

| 阶段 | sp | sscratch |
|------|-----|----------|
| `__restore` 开始 | 内核栈 | 用户栈 |
| `csrrw` 后 | 用户栈 | 内核栈 |
| 用户程序运行 | 用户栈 | 内核栈（保存） |
| trap 发生 | ↓ | ↓ |
| `__alltraps` 执行 | 用户栈 | 内核栈 |
| `csrrw` 后 | 内核栈 | 用户栈 |

### 7. app_init_context 调用位置

**位置**: `batch.rs::run_next_app()`

```rust
TrapContext::app_init_context(
    APP_BASE_ADDRESS,    // 0x80400000
    USER_STACK.get_sp(), // 用户栈顶
)
```

**作用**：为用户程序创建初始执行环境

### 8. 系统调用处理

```rust
Trap::Exception(Exception::UserEnvCall) => {
    cx.sepc += 4;                                    // 跳过 ecall
    cx.x[10] = syscall(cx.x[17], [...]) as usize;    // 返回值
}
```

**修改原因**：
- `sepc += 4`：跳过 `ecall` 指令，避免死循环
- `x[10] = ...`：系统调用返回值约定（a0 寄存器）

---

## 第二章 → 第三章（ch3-coop）主要变化

### 1. 新增 task 子系统

**新增文件**：
- `os/src/task/mod.rs` - 任务管理器
- `os/src/task/task.rs` - 任务控制块 (TCB) 和任务状态
- `os/src/task/context.rs` - 任务上下文结构
- `os/src/task/switch.S` - 上下文切换汇编代码

**重构文件**：
- `os/src/batch.rs` → 拆分为 `os/src/config.rs` + `os/src/loader.rs`

### 2. 任务控制块 (TaskControlBlock)

```rust
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,  // 任务状态
    pub task_cx: TaskContext,      // 任务上下文
}

pub enum TaskStatus {
    UnInit,   // 未初始化
    Ready,    // 就绪
    Running,  // 运行中
    Exited,   // 已退出
}
```

### 3. 任务上下文 (TaskContext)

```rust
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,           // 返回地址
    sp: usize,           // 内核栈指针
    s: [usize; 12],      // callee-saved 寄存器 s0-s11
}
```

**作用**：保存内核态任务切换时需要恢复的寄存器

### 4. goto_restore 的作用

```rust
pub fn goto_restore(kstack_ptr: usize) -> Self {
    unsafe extern "C" {
        unsafe fn __restore();
    }
    Self {
        ra: __restore as usize,  // 关键：ra 指向 __restore
        sp: kstack_ptr,
        s: [0; 12],
    }
}
```

**原理**：
- `__switch` 恢复 TaskContext 后执行 `ret`
- `ret` 跳转到 `ra`，即 `__restore`
- `__restore` 恢复 TrapContext 并通过 `sret` 返回用户态

### 5. __switch 上下文切换

```assembly
__switch:
    # 保存当前任务
    sd sp, 8(a0)
    sd ra, 0(a0)
    .rept 12
        SAVE_SN %n    # 保存 s0-s11
    .endr

    # 恢复下一个任务
    ld ra, 0(a1)
    .rept 12
        LOAD_SN %n    # 恢复 s0-s11
    .endr
    ld sp, 8(a1)      # 最后恢复 sp
    ret
```

### 6. 两层上下文机制

```
┌─────────────────────────────────────────────────┐
│  TaskContext (内核态上下文)                      │
│  - ra, sp, s0-s11                               │
│  - 由 __switch 保存/恢复                         │
│  - 存储在 task_cx 字段中                        │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│  TrapContext (用户态上下文)                      │
│  - 所有通用寄存器, sepc, sstatus                 │
│  - 由 __alltraps/__restore 保存/恢复             │
│  - 存储在内核栈上                                │
└─────────────────────────────────────────────────┘
```

### 7. yield 流程详解

**任务 A 调用 yield**：
```
用户态: yield_()
    → ecall
    → __alltraps (保存用户态到 TrapContext)
    → trap_handler
    → sys_yield
    → suspend_current_and_run_next
    → __switch(&taskA.task_cx, &taskB.task_cx)
        保存: taskA.task_cx.ra = __switch 之后的地址
        保存: taskA.task_cx.sp = 当前内核栈
        切换到任务 B
```

**任务 A 被恢复**：
```
任务 B 完成后
    → __switch(&taskB.task_cx, &taskA.task_cx)
        恢复: ra = __switch 之后的地址
        恢复: sp = 任务 A 的内核栈
        ret → 跳转到 __restore
    → __restore (从内核栈的 TrapContext 恢复用户态)
    → sret → 回到用户态 yield_() 之后
```

### 8. sys_yield 系统调用

```rust
// 新增系统调用
const SYSCALL_YIELD: usize = 124;

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}
```

### 9. 应用加载变化

**第二章**：串行加载，一次一个
```rust
batch::load_app(current_app);  // 加载单个应用
batch::run_next_app();          // 运行后加载下一个
```

**第三章**：批量预加载
```rust
loader::load_apps();  // 一次性加载所有应用
task::run_first_task(); // 运行第一个任务
```

每个应用有独立的内存区域：
```rust
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}
```

### 10. lib.rs 与 bin 应用的协作

**关键机制**：
1. `extern crate user_lib` - 将库链接到每个 bin 应用
2. `#[linkage = "weak"]` - 弱符号机制
3. `_start` 是真正的入口点

```rust
// lib.rs
#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    panic!("Cannot find main!");
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());  // 调用 bin 中的 main
}
```

```rust
// bin/00write_a.rs
#[unsafe(no_mangle)]
fn main() -> i32 {  // 覆盖 lib.rs 中的弱 main
    yield_();
    0
}
```

**Cargo 约定**：
- `src/lib.rs` → 库入口，crate 名来自 `Cargo.toml` 的 `name`
- `src/main.rs` → 默认二进制入口
- `src/bin/*.rs` → 额外的二进制文件

### 11. &task0.task_cx 的含义

```rust
// goto_restore 返回临时值
task.task_cx = TaskContext::goto_restore(init_app_cx(i));
//                ↑ 临时值，在寄存器/栈上
//           ↓ 通过赋值复制到 task_cx 字段

// task 数组在 TASK_MANAGER 静态变量中
let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
//                       ↑ 对静态变量中字段的引用，有固定地址
```

### 12. switch.S 中 sp 的恢复顺序

```assembly
__switch:
    # 恢复下一个任务
    ld ra, 0(a1)
    .rept 12
        LOAD_SN %n    # 先恢复寄存器
    .endr
    ld sp, 8(a1)      # 最后恢复 sp
    ret
```

**为什么 sp 最后恢复**：
- 虽然技术上这个简单函数中可以交换顺序
- 但"先恢复寄存器，最后恢复栈指针"是标准做法
- 保证代码可读性和与保存顺序的对称性
- 避免潜在问题（如果函数使用栈）

---

## 关键概念总结

### 特权级与寄存器
- `stvec`：trap 入口地址
- `sscratch`：辅助栈指针
- `sstatus`：状态寄存器（包含 SPP 特权级）
- `sepc`：异常 PC

### 特权级切换
- U 模式（用户态）→ S 模式（内核态）：trap 发生
- S 模式（内核态）→ U 模式（用户态）：`sret` 指令

### 内存布局
- 内核：0x80000000 附近
- 用户程序：0x80400000（第二章）
- 用户程序：0x80400000 + app_id * 0x20000（第三章）

### 寄存器约定
- `a0` (x10)：函数返回值 / 系统调用返回值
- `a7` (x17)：系统调用号
- `sp` (x2)：栈指针
- `ra` (x1)：返回地址
- `s0-s11`：callee-saved 寄存器

---

## 第二章练习：调用栈打印

### 1. 裸机应用程序概念

**定义**：直接在硬件上运行、没有操作系统支持的程序

**特征**：
- `#![no_std]` - 不使用 Rust 标准库
- `#![no_main]` - 不使用默认的 main 入口
- 自定义运行时环境（`_start` 入口、系统调用封装等）

### 2. RISC-V 栈帧布局（通过反汇编验证）

通过 `rust-objdump -d` 反汇编分析实际代码，确认了栈帧布局：

```
高地址
    ↓
┌────────────────────────────────┐
│ fp (栈帧高边界)                 │ ← current_fp() 返回这个
│                                 │   = 函数被调用前的 sp
├────────────────────────────────┤
│ ... 局部变量 ...                │
├────────────────────────────────┤
│ fp - 8  [保存的 ra]            │ ← 返回地址
├────────────────────────────────┤
│ fp - 16 [保存的上一个 fp]       │ ← 调用者的 fp
├────────────────────────────────┤
│ ...                            │
    ↓ 低地址（栈向下增长）
```

**关键发现**：
- `fp` 指向**栈帧的高地址边界**（即函数被调用前的 sp 值）
- 栈是**向下增长**的（高地址 → 低地址）
- 返回地址在 `fp - 8`
- 上一个 fp 在 `fp - 16`

### 3. 函数序言的标准模式

```assembly
# RISC-V 标准函数序言
addi	sp, sp, -XX     # 分配栈空间（向下增长）
sd	ra, X(sp)        # 保存返回地址
sd	s0, Y(sp)        # 保存帧指针
addi	s0, sp, XX      # 设置新的 fp = sp + 栈帧大小
```

**反汇编示例**（recursive_function）：
```assembly
804007f8: 7151      addi	sp, sp, -0xf0    # 栈帧大小 240 字节
804007fa: f586      sd	ra, 0xe8(sp)
804007fc: f1a2      sd	s0, 0xe0(sp)
804007fe: 1980      addi	s0, sp, 0xf0     # fp = sp + 0xf0
```

### 4. 调用栈遍历实现

```rust
fn from_fp(fp: FramePointer) -> Option<StackFrame> {
    unsafe {
        if fp == 0 { return None; }

        let saved_ra = *((fp - 8) as *const usize);   // 返回地址
        let saved_fp = *((fp - 16) as *const usize);  // 上一个 fp

        Some(StackFrame { fp: saved_fp, ra: saved_ra })
    }
}

// 遍历调用栈
let mut fp = current_fp();
loop {
    if fp == 0 { break; }
    let frame = StackFrame::from_fp(fp)?;
    println!("FP: {:#x}, RA: {:#x}", fp, frame.ra);
    fp = frame.fp;  // 移动到上一个栈帧
}
```

### 5. 验证机制设计

**问题**：如何验证调用栈打印的正确性？

**方法**：在递归调用时记录 fp，然后在遍历调用栈时验证这些 fp 是否都能被找到

```rust
static mut FP_RECORDS: [FramePointer; MAX] = [0; MAX];
static mut FP_COUNT: usize = 0;

fn record_fp() {
    unsafe {
        FP_RECORDS[FP_COUNT] = current_fp();
        FP_COUNT += 1;
    }
}

// 遍历时验证
for each_frame in stack_trace {
    if FP_RECORDS.contains(&frame.fp) {
        println!("[RECORDED]");
    }
}
```

### 6. 关键注意事项

1. **fp 与 sp 的关系**：fp = 函数被调用前的 sp（不是当前的 sp）
2. **栈帧大小可能不同**：不同函数可能有不同的栈帧大小
3. **记录顺序与遍历顺序相反**：
   - 记录：从外向内（递归深度递减）
   - 遍历：从内向外（从最深的递归层开始返回）

### 7. 实际输出示例

```
Recursive depth: 5 (fp=0x80208f60)
Recursive depth: 4 (fp=0x80208e70)
...
=== Stack Trace ===
[0] Frame: 0x80208ab0 [RECORDED]
[1] Frame: 0x80208ba0 [RECORDED]
...
Found 6 of 6 in stack trace
All recorded frame pointers verified!
```

### 8. 常见错误

**错误版本**：
```rust
let saved_fp = *(fp as *const usize);        // ❌ 错误偏移
let saved_ra = *((fp - 8) as *const usize);
```

**正确版本**：
```rust
let saved_ra = *((fp - 8) as *const usize);   // ✓
let saved_fp = *((fp - 16) as *const usize);  // ✓
```

---

## 第二章编程题：get_taskinfo 系统调用

### 题目要求
**编程题第二题**：扩展内核，实现新系统调用 `get_taskinfo`，能显示当前 task 的 id 和 task name；实现一个裸机应用程序 B，能访问 `get_taskinfo` 系统调用。

### 实现方案

#### 1. 应用名称表的自动生成

**问题**：如何在运行时获取应用程序名称，而不硬编码？

**解决方案**：修改 `os/build.rs`，在生成 `link_app.S` 时同时生成应用名称表

```rust
// os/build.rs (修改后)
for (idx, app) in apps.iter().enumerate() {
    // ... 生成 app_{idx}_start/end ...
}

// 生成应用名称表
writeln!(f, r#"
    .section .data
    .global _app_names
_app_names:"#)?;
for app in apps.iter() {
    writeln!(f, r#"    .string "{}""#, app)?;
}
```

生成的 `link_app.S` 片段：
```assembly
    .section .data
    .global _app_names
_app_names:
    .string "00hello_world"
    .string "01store_fault"
    .string "02power"
    .string "03priv_inst"
    .string "04priv_csr"
    .string "05panic_test"
    .string "06stack_trace"
    .string "07task_info"
```

#### 2. 内核端实现

**os/src/batch.rs** - 添加获取应用名称的函数和任务信息接口：

```rust
// 声明外部符号
unsafe extern "C" {
    fn _app_names();
}

// 根据 app_id 从名称表中获取应用名称
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
    // 获取当前正在运行的应用 ID
    // 注意：current_app 在 move_to_next_app 后已经指向下一个应用
    pub fn get_running_app(&self) -> usize {
        self.current_app - 1
    }
}

// 公共接口：获取当前任务信息
pub fn get_current_task_info() -> (usize, &'static str) {
    let app_manager = APP_MANAGER.exclusive_access();
    let app_id = app_manager.get_running_app();
    let app_name = get_app_name(app_id);
    (app_id, app_name)
}
```

**os/src/syscall/mod.rs** - 添加系统调用号：
```rust
const SYSCALL_GET_TASKINFO: usize = 2000;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        // ...
        SYSCALL_GET_TASKINFO => sys_get_taskinfo(args[0] as *mut u8, args[1]),
        // ...
    }
}
```

**os/src/syscall/process.rs** - 实现系统调用：
```rust
pub fn sys_get_taskinfo(buf: *mut u8, max_len: usize) -> isize {
    if buf.is_null() || max_len == 0 {
        return -1;
    }
    let (task_id, task_name) = get_current_task_info();
    let name_bytes = task_name.as_bytes();
    let copy_len = core::cmp::min(name_bytes.len(), max_len - 1);

    unsafe {
        core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), buf, copy_len);
        *buf.add(copy_len) = 0; // 添加 null 终止符
    }

    task_id as isize
}
```

#### 3. 用户态实现

**user/src/syscall.rs** - 用户态封装：
```rust
const SYSCALL_GET_TASKINFO: usize = 2000;

pub fn sys_get_taskinfo(buf: &mut [u8]) -> isize {
    syscall(SYSCALL_GET_TASKINFO, [buf.as_mut_ptr() as usize, buf.len(), 0])
}
```

**user/src/lib.rs** - 公共接口：
```rust
pub fn get_taskinfo(buf: &mut [u8]) -> isize {
    sys_get_taskinfo(buf)
}
```

**user/src/bin/07task_info.rs** - 测试程序：
```rust
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::get_taskinfo;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("=== Task Info Test ===");

    let mut buf = [0u8; 64];
    let task_id = get_taskinfo(&mut buf);

    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let task_name = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };

    println!("Current task ID: {}", task_id);
    println!("Current task name: {}", task_name);

    0
}
```

### 关键设计要点

1. **current_app 的时机问题**
   - `run_next_app` 中先调用 `load_app(current_app)`，然后 `move_to_next_app()`
   - 当应用程序运行时，`current_app` 已经指向下一个应用
   - 解决方案：使用 `current_app - 1` 获取当前正在运行的应用 ID

2. **Rust 2024 edition 的变化**
   - `extern "C"` 块需要改为 `unsafe extern "C"`
   - bin 文件中需要显式导入函数：`use user_lib::get_taskinfo;`
   - `#[macro_use]` 只对宏生效，对函数无效

3. **C 字符串的处理**
   - 名称表是按顺序存储的以 null 结尾的字符串
   - 需要手动遍历找到目标字符串
   - 返回给用户时需要复制到用户缓冲区并添加 null 终止符

### 运行结果

```
[kernel] Loading app_7
=== Task Info Test ===
Current task ID: 7
Current task name: 07task_info
Test completed successfully!
```

### 修改的文件清单

| 文件 | 修改内容 |
|------|----------|
| `os/build.rs` | 生成应用名称表 `_app_names` |
| `os/src/batch.rs` | 添加 `get_app_name`、`get_running_app`、`get_current_task_info` |
| `os/src/syscall/mod.rs` | 添加 `SYSCALL_GET_TASKINFO` |
| `os/src/syscall/process.rs` | 实现 `sys_get_taskinfo` |
| `user/src/syscall.rs` | 添加 `sys_get_taskinfo` |
| `user/src/lib.rs` | 添加 `get_taskinfo` 公共接口 |
| `user/src/bin/07task_info.rs` | 新建测试程序 |

---

## 第二章编程题：系统调用统计

### 题目要求
**编程题第三题**：扩展内核，能够统计多个应用的执行过程中系统调用编号和访问此系统调用的次数。

### 实现方案

#### 1. 数据结构设计

在 `batch.rs` 中添加全局统计数据：

```rust
// 系统调用统计相关常量
/// 系统调用编号的最大值，用于统计数组大小
pub const MAX_SYSCALL_ID: usize = 256;

lazy_static! {
    // 系统调用统计数组：SYSCALL_STATS[app_id][syscall_id] = count
    static ref SYSCALL_STATS: UPSafeCell<[[usize; MAX_SYSCALL_ID]; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([[0; MAX_SYSCALL_ID]; MAX_APP_NUM])
    };
}
```

**设计要点**：
- 使用二维数组存储：`[应用][系统调用号] = 次数`
- 第二章没有 HashMap，使用固定大小数组
- `MAX_SYSCALL_ID = 256` 覆盖所有可能的系统调用号
- 使用 `lazy_static!` 包裹，因为 `UPSafeCell::new` 不是 const 函数

#### 2. 记录系统调用

在 `os/src/syscall/mod.rs` 的 `syscall` 函数中添加记录逻辑：

```rust
use crate::batch::MAX_SYSCALL_ID;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // 记录系统调用（如果 syscall_id 在有效范围内）
    if syscall_id < MAX_SYSCALL_ID {
        crate::batch::record_syscall(syscall_id);
    }

    match syscall_id {
        // ... 系统调用分发
    }
}
```

**os/src/batch.rs** 中的记录函数：
```rust
/// 记录系统调用
pub fn record_syscall(syscall_id: usize) {
    let mut stats = SYSCALL_STATS.exclusive_access();
    let app_id = APP_MANAGER.exclusive_access().get_running_app();
    if app_id < MAX_APP_NUM && syscall_id < MAX_SYSCALL_ID {
        stats[app_id][syscall_id] += 1;
    }
}
```

#### 3. 输出统计信息

在 `os/src/batch.rs` 中添加输出函数：
```rust
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
```

在 `os/src/syscall/process.rs` 的 `sys_exit` 中调用：
```rust
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // 输出系统调用统计信息
    print_syscall_stats(get_current_app_id());
    run_next_app()
}
```

#### 4. 关键设计要点

1. **使用常量而非硬编码**
   - 使用 `pub const MAX_SYSCALL_ID` 而非硬编码 256
   - 便于维护和修改

2. **lazy_static! 的使用**
   - `UPSafeCell::new` 不是 const 函数，不能在 static 中直接调用
   - 必须使用 `lazy_static!` 宏包裹
   - 需要用 `unsafe` 块包裹（因为 `new` 是 unsafe 函数）

3. **边界检查**
   - 检查 `app_id < MAX_APP_NUM`
   - 检查 `syscall_id < MAX_SYSCALL_ID`
   - 防止数组越界

4. **输出时机**
   - 在 `sys_exit` 中输出，即应用退出时
   - 也可以在所有应用完成后输出汇总信息

### 运行结果

```
[kernel] Loading app_0
Hello, world!
[kernel] Application exited with code 0

[kernel] === Syscall Statistics for 00hello_world (app_0) ===
[kernel]   Syscall 64: 1 calls
[kernel]   Syscall 93: 1 calls
[kernel]   Total: 2 syscalls

[kernel] Loading app_2
3^10000=5079(MOD 10007)
...
[kernel] Application exited with code 0

[kernel] === Syscall Statistics for 02power (app_2) ===
[kernel]   Syscall 64: 81 calls
[kernel]   Syscall 93: 1 calls
[kernel]   Total: 82 syscalls
```

### 系统调用编号对照

| 编号 | 名称 | 说明 |
|------|------|------|
| 64 | sys_write | 写入输出 |
| 93 | sys_exit | 退出应用 |

### 修改的文件清单

| 文件 | 修改内容 |
|------|----------|
| `os/src/batch.rs` | 添加 `MAX_SYSCALL_ID`、`SYSCALL_STATS`、`record_syscall()`、`print_syscall_stats()`、`get_current_app_id()` |
| `os/src/syscall/mod.rs` | 在 `syscall()` 中添加记录调用，导入 `MAX_SYSCALL_ID` |
| `os/src/syscall/process.rs` | 在 `sys_exit()` 中调用 `print_syscall_stats()` |

### 总结

这个功能的实现展示了：
1. 全局状态的管理（使用 `lazy_static!` 和 `UPSafeCell`）
2. 系统调用的拦截和统计
3. 在合适的时机输出统计信息
4. 边界检查的重要性

---

## 第二章编程题：应用执行时间统计

### 题目要求
**编程题第四题**：扩展内核，能够统计每个应用执行后的完成时间。

### 实现方案

#### 1. 时间读取方式

RISC-V 提供了 `time` CSR 寄存器（mtime），可以通过 `rdtime` 指令读取：

```rust
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
```

**注意**：`rdtime` 指令只需要一个输出寄存器，不是两个。

#### 2. 数据结构设计

在 `batch.rs` 中添加时间统计数据结构（使用 `lazy_static!`）：

```rust
lazy_static! {
    /// 应用执行时间统计（单位：CPU cycles）
    static ref APP_EXEC_TIME: UPSafeCell<[usize; MAX_APP_NUM]> = unsafe {
        UPSafeCell::new([0; MAX_APP_NUM])
    };

    /// 应用开始时间（临时记录）
    static ref APP_START_TIME: UPSafeCell<usize> = unsafe {
        UPSafeCell::new(0)
    };
}
```

**为什么使用 `lazy_static!`**：
- `UPSafeCell::new` 不是 const 函数，不能在 static 中直接调用
- 必须使用 `lazy_static!` 宏包裹
- 需要用 `unsafe` 块包裹（因为 `new` 是 unsafe 函数）

#### 3. 记录时机

| 时机 | 位置 | 操作 |
|------|------|------|
| 应用开始 | `run_next_app()` 中，`load_app()` 之后 | `record_app_start_time()` |
| 应用结束 | `sys_exit()` 中 | `record_app_end_time(app_id)` |

```rust
// batch.rs
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    app_manager.load_app(current_app);
    app_manager.move_to_next_app();
    drop(app_manager);
    // 记录应用开始时间
    record_app_start_time();
    // ...
}

pub fn record_app_start_time() {
    *APP_START_TIME.exclusive_access() = get_time();
}

pub fn record_app_end_time(app_id: usize) {
    let end_time = get_time();
    let start_time = *APP_START_TIME.exclusive_access();
    let exec_time = end_time - start_time;
    APP_EXEC_TIME.exclusive_access()[app_id] = exec_time;
}
```

#### 4. CPU 频率验证

**QEMU virt 平台的 mtime 频率**：

从 QEMU 源码中可以查到：
```c
// qemu-7.0.0/include/hw/intc/riscv_aclint.h:75
RISCV_ACLINT_DEFAULT_TIMEBASE_FREQ = 10000000  // 10 MHz
```

**验证方法**：
1. 查看 QEMU 源码 `hw/intc/riscv_aclint.h` 中的 `RISCV_ACLINT_DEFAULT_TIMEBASE_FREQ`
2. QEMU 在启动时将这个值写入设备树的 `/cpus/timebase-frequency` 属性

**时间转换公式**：
```
微秒 = CPU_cycles / 频率(MHz)
微秒 = CPU_cycles / 10
```

#### 5. 输出格式

在 `process.rs` 的 `sys_exit` 中输出执行时间：

```rust
pub fn sys_exit(exit_code: i32) -> ! {
    let app_id = get_current_app_id();
    record_app_end_time(app_id);
    let exec_cycles = get_app_exec_time(app_id);
    // 转换为微秒（QEMU virt 的 CPU 频率为 10 MHz）
    let exec_time_us = exec_cycles / 10;
    println!("[kernel] Application exited with code {}", exit_code);
    println!("[kernel] Execution time: {} us ({} cycles)", exec_time_us, exec_cycles);
    print_syscall_stats(app_id);
    run_next_app()
}
```

### 运行结果

```
[kernel] Loading app_0
Hello, world!
[kernel] Application exited with code 0
[kernel] Execution time: 773 us (7739 cycles)

[kernel] === Syscall Statistics for 00hello_world (app_0) ===
[kernel]   Syscall 64: 1 calls
[kernel]   Syscall 93: 1 calls
[kernel]   Total: 2 syscalls

[kernel] Loading app_2
3^10000=5079(MOD 10007)
...
[kernel] Application exited with code 0
[kernel] Execution time: 1381 us (13810 cycles)
```

### 统计数据示例

| 应用 | 执行时间（微秒） | CPU 周期数 |
|------|----------------|-----------|
| 00hello_world | 773 us | 7739 cycles |
| 02power | 1381 us | 13810 cycles |
| 05panic_test | 584 us | 5848 cycles |
| 06stack_trace | 3323 us | 33234 cycles |
| 07task_info | 512 us | 5120 cycles |
| test_stack_frame | 389 us | 3891 cycles |

### 修改的文件清单

| 文件 | 修改内容 |
|------|----------|
| `os/src/batch.rs` | 添加 `get_time()`、`APP_EXEC_TIME`、`APP_START_TIME`、`record_app_start_time()`、`record_app_end_time()`、`get_app_exec_time()` |
| `os/src/syscall/process.rs` | 在 `sys_exit()` 中调用 `record_app_end_time()` 和 `get_app_exec_time()`，输出执行时间 |

### 关键设计要点

1. **rdtime 指令的正确用法**
   - `rdtime` 只需要一个输出寄存器：`asm!("rdtime {}", out(reg) time)`
   - 不要写成 `rdtime {}, {}` 两个输出

2. **lazy_static! 的必要性**
   - 静态变量初始化需要 `lazy_static!` 宏
   - 必须用 `unsafe` 块包裹 `UPSafeCell::new`

3. **时间记录的时机**
   - 开始时间：在 `run_next_app()` 中，应用加载完成后、跳转到用户态前
   - 结束时间：在 `sys_exit()` 中，应用退出时

4. **CPU 频率的获取**
   - QEMU virt 平台的 mtime 频率固定为 10 MHz
   - 可以从 QEMU 源码或设备树中获取
   - 不要用输出数据"反推"频率，应该从硬件配置中获取

---

## 第二章编程题：异常统计（编程题第五题）

### 题目要求
**编程题第五题**：扩展内核，统计执行异常的程序的异常情况（主要是各种特权级涉及的异常），能够打印异常程序的出错的地址和指令等信息。

### 实现方案

#### 1. 数据结构设计

在 `batch.rs` 中添加异常统计数据结构：

```rust
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
```

#### 2. 异常统计数组

使用固定大小数组存储异常记录（第二章没有 `alloc`，不能使用 `Vec`）：

```rust
// 每个应用最多记录的异常数量
const MAX_EXCEPTIONS_PER_APP: usize = 16;

lazy_static! {
    /// 每个应用的异常记录数组
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
```

#### 3. 异常记录函数

```rust
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

/// 读取指定地址的指令（用于调试）
pub fn fetch_instruction(sepc: usize) -> u32 {
    unsafe { (sepc as *const u32).read_volatile() }
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
```

#### 4. trap 处理扩展

在 `trap/mod.rs` 中扩展异常处理：

```rust
use crate::batch::{print_exception_details, record_exception, run_next_app, ExceptionType};

pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        // 指令地址未对齐异常
        Trap::Exception(Exception::InstructionMisaligned) => {
            record_exception(ExceptionType::InstructionAddressMisaligned, cx.sepc, stval);
            print_exception_details(ExceptionType::InstructionAddressMisaligned, cx.sepc, stval);
            run_next_app();
        }
        // 指令页错误
        Trap::Exception(Exception::InstructionPageFault) => {
            record_exception(ExceptionType::InstructionAccessFault, cx.sepc, stval);
            print_exception_details(ExceptionType::InstructionAccessFault, cx.sepc, stval);
            run_next_app();
        }
        // 非法指令
        Trap::Exception(Exception::IllegalInstruction) => {
            record_exception(ExceptionType::IllegalInstruction, cx.sepc, stval);
            print_exception_details(ExceptionType::IllegalInstruction, cx.sepc, stval);
            run_next_app();
        }
        // 加载页错误
        Trap::Exception(Exception::LoadPageFault) => {
            record_exception(ExceptionType::LoadAccessFault, cx.sepc, stval);
            print_exception_details(ExceptionType::LoadAccessFault, cx.sepc, stval);
            run_next_app();
        }
        // 存储地址未对齐
        Trap::Exception(Exception::StoreMisaligned) => {
            record_exception(ExceptionType::StoreAddressMisaligned, cx.sepc, stval);
            print_exception_details(ExceptionType::StoreAddressMisaligned, cx.sepc, stval);
            run_next_app();
        }
        // 存储错误
        Trap::Exception(Exception::StoreFault) => {
            record_exception(ExceptionType::StoreFault, cx.sepc, stval);
            print_exception_details(ExceptionType::StoreFault, cx.sepc, stval);
            run_next_app();
        }
        // 存储页错误
        Trap::Exception(Exception::StorePageFault) => {
            record_exception(ExceptionType::StorePageFault, cx.sepc, stval);
            print_exception_details(ExceptionType::StorePageFault, cx.sepc, stval);
            run_next_app();
        }
        _ => {
            record_exception(ExceptionType::Unknown, cx.sepc, stval);
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}
```

### RISC-V 异常类型映射

| RISC-V Exception | 内部 ExceptionType | 说明 |
|-----------------|-------------------|------|
| `InstructionMisaligned` | `InstructionAddressMisaligned` | 指令地址未对齐 |
| `InstructionPageFault` | `InstructionAccessFault` | 指令访问错误 |
| `IllegalInstruction` | `IllegalInstruction` | 非法指令 |
| `LoadPageFault` | `LoadAccessFault` | 加载访问错误 |
| `StoreMisaligned` | `StoreAddressMisaligned` | 存储地址未对齐 |
| `StoreFault` | `StoreFault` | 存储错误 |
| `StorePageFault` | `StorePageFault` | 存储页错误 |

### 运行结果

#### StoreFault 异常（01store_fault 应用）
```
[kernel] ==================== Exception Details ====================
[kernel] Application: 01store_fault (app_1)
[kernel] Exception Type: StoreFault
[kernel] Fault Instruction Address (sepc): 0x804000ba
[kernel] Fault Address (stval): 0x0
[kernel] Instruction: 0x23
[kernel] =========================================================
```

#### IllegalInstruction 异常（03priv_inst 应用）
```
[kernel] ==================== Exception Details ====================
[kernel] Application: 03priv_inst (app_3)
[kernel] Exception Type: IllegalInstruction
[kernel] Fault Instruction Address (sepc): 0x804000ba
[kernel] Fault Address (stval): 0x0
[kernel] Instruction: 0x10200073
[kernel] =========================================================
```

#### IllegalInstruction 异常（04priv_csr 应用）
```
[kernel] ==================== Exception Details ====================
[kernel] Application: 04priv_csr (app_4)
[kernel] Exception Type: IllegalInstruction
[kernel] Fault Instruction Address (sepc): 0x804000be
[kernel] Fault Address (stval): 0x0
[kernel] Instruction: 0x10053073
[kernel] =========================================================
```

### 修改的文件清单

| 文件 | 修改内容 |
|------|----------|
| `os/src/batch.rs` | 添加 `ExceptionType`、`ExceptionRecord`、`APP_EXCEPTIONS`、`APP_EXCEPTION_COUNTS`、`record_exception()`、`print_exception_details()`、`fetch_instruction()` |
| `os/src/trap/mod.rs` | 扩展异常处理分支，导入 `ExceptionType` 和相关函数 |

### 关键设计要点

1. **不使用 Vec**
   - 第二章没有 `alloc` crate，不能使用 `Vec`
   - 使用固定大小的数组和计数器来存储异常记录

2. **RISC-V 异常名称差异**
   - riscv crate 中的异常名称与规范不同
   - `InstructionMisaligned` 而非 `InstructionAddressMisaligned`
   - `InstructionPageFault` 而非 `InstructionAccessFault`
   - 需要进行映射

3. **指令读取**
   - 使用 `fetch_instruction()` 从 `sepc` 地址读取指令
   - 指令是 32 位（4 字节），使用 `u32` 类型
   - 使用 `read_volatile()` 防止编译器优化

4. **异常信息输出**
   - 异常发生时立即打印详细信息
   - 包含应用名称、异常类型、出错地址 (sepc)、相关地址 (stval)、指令内容

---

## 第二章：rcore_tutorial_tests 测试仓库

### 测试仓库说明

**仓库地址**: https://github.com/rcore-os/rcore_tutorial_tests

这是 rCore Tutorial v3.5 的专门测试用例仓库，用于每一章后面的练习题测试。

### 仓库结构

```
rcore_tutorial_tests/
├── Makefile          # 测试构建文件
├── README.md
├── guide.md          # 测试使用指导
├── check/            # 自动化测试脚本
│   ├── base.py       # 基础测试框架
│   ├── ch2.py        # 第2章测试
│   ├── ch3_0.py      # 第3章测试（分3个部分）
│   └── ...
├── overwrite/        # 覆盖文件（build.rs, Makefile等）
└── user/             # 用户态测试程序
    ├── Cargo.toml
    ├── Makefile
    └── src/bin/      # 按章节组织的测试用例
```

### 第二章测试用例

| 文件 | 测试内容 |
|------|---------|
| `ch2_hello_world.rs` | 基本输出测试 |
| `ch2_power.rs` | 计算功能测试 |
| `ch2_write1.rs` | sys_write 正确调用 + 参数验证 |
| `ch2_exit.rs` | sys_exit 调用 |
| `ch2t_write0.rs` | sys_write 错误处理（地址检查） |
| `_ch2_bad_instruction.rs` | 非法指令异常测试 |
| `_ch2_bad_register.rs` | 特权寄存器异常测试 |
| `_ch2t_bad_address.rs` | 地址访问异常测试 |

### 使用方法

```bash
# 1. Clone 测试仓库
git clone https://github.com/rcore-os/rcore_tutorial_tests.git

# 2. 编译测试用例
cd rcore_tutorial_tests/user
make all CHAPTER=2

# 3. 运行自动化测试
cd ..
make test CHAPTER=2
```

### 测试框架说明

测试脚本使用 Python 实现，检查内核输出是否包含预期的字符串：

```python
# check/ch2.py
EXPECTED = [
    "Hello world from user mode program!\nTest hello_world OK!",
    "Test power OK!",
    "string from data section\nstrinstring from stack section\nstrin\nTest write1 OK!",
]
NOT_EXPECTED = [
    "FAIL: T.T",
]
```

### 测试结果

**Test passed: 7/7** ✅

所有第二章测试用例均通过。

---

## 第二章：测试环境问题修复记录

### 问题1：Rust 镜像源无法访问

**错误信息**:
```
failed to fetch `sparse+https://rsproxy.cn/index/`
git: 'remote-sparse+https' is not a git command
```

**解决方案**: 更改 `~/.cargo/config.toml` 使用清华镜像

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

[net]
git-fetch-with-cli = true
```

### 问题2：依赖版本不兼容

**错误信息**:
```
error: package `lock_api v0.4.14` cannot be built because it requires rustc 1.71.0 or newer,
while the currently active rustc version is 1.51.0-nightly
```

**解决方案**: 降级 `spin` 版本从 0.9 到 0.7

```toml
# rcore_tutorial_tests/user/Cargo.toml
[dependencies]
spin = "0.7"  # 从 0.9 降级
```

### 问题3：函数项转换错误

**错误信息**:
```
error: direct cast of function item into an integer
```

**解决方案**: 使用 `as *const () as usize` 替代 `as usize`

```rust
// 错误写法
sbss as usize

// 正确写法
sbss as *const () as usize
```

### 问题4：sys_write 错误处理

**问题描述**: `ch2_write1` 测试期望传入非法 fd 时返回 -1，但实现中使用了 panic

**解决方案**: 修改 `sys_write` 返回 -1 而非 panic

```rust
_ => {
    -1  // 返回 -1 而非 panic
}
```

### 问题5：用户地址检查

**问题描述**: `ch2t_write0` 测试传入空指针和非法地址，需要在内核中进行地址验证

**解决方案**: 添加 `check_user_buffer` 函数

```rust
const USER_STACK_BASE: usize = 0x80400000 - 0x1000;
const USER_STACK_TOP: usize = 0x80400000;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

fn check_user_buffer(buf: *const u8, len: usize) -> bool {
    let addr = buf as usize;
    if addr == 0 { return false; }
    // 检查用户栈和应用内存范围
    // ...
}
```

### 关键系统调用

| syscall ID | 名称 | 参数 | 返回值 |
|-----------|------|------|--------|
| 64 | sys_write | fd, buf, len | 实际写入字节数 / -1 |
| 93 | sys_exit | exit_code | 不返回 |

### 总结

1. **测试仓库独立管理** - rcore_tutorial_tests 是独立仓库，需要单独 clone
2. **测试自动化** - 使用 Python 脚本检查输出是否符合预期
3. **依赖版本管理** - 旧版 Rust 需要使用兼容的依赖版本
4. **错误处理规范** - 系统调用应返回错误码而非 panic
5. **地址安全检查** - 内核需要验证用户传入的地址合法性

---

## 第三章编程题第一题：显示任务切换过程

### 题目要求
**编程题第一题（*）**：扩展内核，能够显示操作系统切换任务的过程。

### 实现方案

#### 1. build.rs - 生成应用名称表

在生成 `link_app.S` 时同时生成应用名称表 `_app_names`，包含所有应用名称的 C 字符串。

```rust
// 在文件末尾添加：
// 生成应用名称表
writeln!(f, r#"
    .section .data
    .global _app_names
_app_names:"#)?;
for app in apps.iter() {
    writeln!(f, r#"    .string "{}""#, app)?;
}
```

#### 2. loader.rs - 添加获取应用名称函数

```rust
/// 根据 app_id 从应用名称表获取应用名称
pub fn get_app_name(app_id: usize) -> &'static str {
    unsafe extern "C" {
        safe fn _app_names();
    }
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
        let slice = core::slice::from_raw_parts(start as *const u8, end - start);
        core::str::from_utf8_unchecked(slice)
    }
}
```

#### 3. task/mod.rs - 任务切换追踪

**添加切换原因枚举：**
```rust
/// 任务切换原因
#[derive(Clone, Copy)]
pub enum SwitchReason {
    /// 首次启动
    Start,
    /// 主动让出 CPU (yield)
    Yield,
    /// 任务退出 (exit)
    Exit,
}
```

**添加显示切换信息方法：**
```rust
impl TaskManager {
    /// 显示任务切换信息
    fn show_task_switch(&self, from: usize, to: usize, reason: SwitchReason) {
        let from_name = get_app_name(from);
        let to_name = get_app_name(to);
        let from_status = self.inner.exclusive_access().tasks[from].task_status;

        let reason_str = match reason {
            SwitchReason::Start => "START",
            SwitchReason::Yield => "YIELD",
            SwitchReason::Exit => "EXIT",
        };

        println!(
            "[kernel] >>> Task Switch: {} ({}) ({:?}) -> {} ({}) [{}]",
            from, from_name, from_status, to, to_name, reason_str
        );
    }
}
```

**修改启动函数添加日志：**
```rust
fn run_first_task(&self) -> ! {
    // ...
    println!("[kernel] >>> Starting first task: 0 ({})", get_app_name(0));
    // ...
}
```

**添加退出并切换方法：**
```rust
fn exit_and_run_next(&self) {
    if let Some(next) = self.find_next_task() {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.tasks[next].task_status = TaskStatus::Running;
        inner.current_task = next;
        let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
        let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
        drop(inner);
        // 显示任务退出信息
        self.show_task_switch(current, next, SwitchReason::Exit);
        unsafe {
            __switch(current_task_cx_ptr, next_task_cx_ptr);
        }
    } else {
        println!("All applications completed!");
        shutdown(false);
    }
}
```

**修改普通切换添加日志：**
```rust
fn run_next_task(&self) {
    if let Some(next) = self.find_next_task() {
        // ...
        self.show_task_switch(current, next, SwitchReason::Yield);
        // ...
    }
}
```

#### 4. task/task.rs - 添加 Debug trait

```rust
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
```

### 修改的文件清单

| 文件 | 修改内容 |
|------|----------|
| `build.rs` | 生成 `_app_names` 应用名称表 |
| `src/loader.rs` | 添加 `get_app_name()` 函数 |
| `src/task/mod.rs` | 添加 `SwitchReason` 枚举、`show_task_switch()` 方法、`exit_and_run_next()` 方法 |
| `src/task/task.rs` | 给 `TaskStatus` 添加 `Debug` derive |
| `.cargo/config.toml` | 修正链接器脚本路径为 `linker-qemu.ld`（原有bug） |

### 运行效果

```
[kernel] >>> Starting first task: 0 (00power_3)
Test power_3 OK!
[kernel] Application exited with code 0
[kernel] >>> Task Switch: 0 (00power_3) (Exited) -> 1 (01power_5) [EXIT]
Test power_5 OK!
[kernel] Application exited with code 0
[kernel] >>> Task Switch: 1 (01power_5) (Exited) -> 2 (02power_7) [EXIT]
Test power_7 OK!
[kernel] Application exited with code 0
[kernel] >>> Task Switch: 2 (02power_7) (Exited) -> 3 (03sleep) [EXIT]
[kernel] >>> Task Switch: 3 (03sleep) (Running) -> 3 (03sleep) [YIELD]
```

### 显示格式说明

```
[kernel] >>> Task Switch: 源任务ID (源任务名) (状态) ->目标任务ID (目标任务名) [切换原因]
```

**切换原因类型：**
- `START` - 首次启动任务
- `YIELD` - 主动让出 CPU（通过 sys_yield）
- `EXIT` - 任务退出（通过 sys_exit）

### 关键设计要点

1. **应用名称表机制**
   - 在编译时通过 `build.rs` 生成所有应用名称
   - 存储为连续的 C 字符串（以 null 结尾）
   - 运行时通过遍历查找指定索引的名称

2. **切换时机追踪**
   - `run_first_task()` - 首次启动时显示
   - `run_next_task()` - yield 触发的切换
   - `exit_and_run_next()` - exit 触发的切换

3. **状态显示**
   - 显示源任务状态（Ready/Running/Exited）
   - 显示源任务和目标任务名称
   - 显示切换原因便于调试

---

## Ch3 编程题二：用户态/内核态时间统计

**题目要求**：扩展内核，能够统计每个应用执行后的完成时间：用户态完成时间和内核态完成时间。

**实现位置**：`chapter3-exercises-tmp` 目录

### 修改的文件

| 文件 | 修改内容 |
|------|----------|
| `os/src/task/task.rs` | 添加 `AppTimeStats` 结构体，嵌入 `TaskControlBlock` |
| `os/src/task/mod.rs` | 添加时间统计函数：`get_current_task_id()`, `add_user_time()`, `add_kernel_time()`, `get_task_time_stats()`，以及任务切换显示控制 `set_show_task_switch()` |
| `os/src/trap/context.rs` | 在 `TrapContext` 添加 `trap_entry_time` 和 `user_base_time` 字段 |
| `os/src/trap/mod.rs` | 在 `trap_handler` 中计算并累加用户态/内核态时间 |
| `os/src/syscall/process.rs` | 在 `sys_exit` 中输出时间统计信息 |

### 实现原理

**核心思路**：在 trap 进入和退出时记录时间戳，分别累加到用户态和内核态时间。

```
用户态运行 ──────> __alltraps ──────> trap_handler
     ↑                                    │
     │                                    ↓ 用户态时间 = now - user_base_time
     │                            累加用户态时间
     │                                    │
     │                                    ↓ 记录 trap_entry_time
     │                            执行内核处理
     │                                    │
     │                                    ↓ 内核态时间 = now - trap_entry_time
     └──────────── __restore <────────── 累加内核态时间
                              更新 user_base_time
```

### 关键数据结构

```rust
/// 应用时间统计
#[derive(Copy, Clone, Debug)]
pub struct AppTimeStats {
    pub user_time: usize,    // 用户态执行时间（CPU cycles）
    pub kernel_time: usize,  // 内核态执行时间（CPU cycles）
}

/// TrapContext 添加时间戳字段
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub trap_entry_time: usize,   // 进入 trap 时的时间戳
    pub user_base_time: usize,    // 上次进入用户态的时间戳
}
```

### 运行效果

```
[kernel] Application 00power_3 (task 0) exited with code 0
[kernel]   User time:   25844 cycles
[kernel]   Kernel time: 25698 cycles
[kernel]   Total time:  51542 cycles
```

### 任务切换显示控制

添加了 `set_show_task_switch(bool)` 函数来控制是否显示任务切换信息：
- 默认 `false` - 不显示，输出更简洁
- 可调用 `set_show_task_switch(true)` 开启调试信息

### 关于分时多任务的观察

**问题**：为什么任务看起来是串行执行的？

**原因**：
- `TICKS_PER_SEC = 100` → 定时器中断间隔 = 10ms
- 简单任务（如 power_3）执行时间约 4-5ms，在定时器触发前就执行完了
- 改为 `TICKS_PER_SEC = 1000`（1ms 间隔）可以看到明显的任务交替执行

**修改位置**：`os/src/timer.rs`
```rust
const TICKS_PER_SEC: usize = 1000;  // 默认 100，改为 1000 可看到明显并发效果
```

---

## Ch3 编程题三：浮点应用支持

**题目要求**：编写浮点应用程序A，并扩展内核，支持面向浮点应用的正常切换与抢占。

**实现位置**：`chapter3-exercises-tmp` 目录

### 修改的文件

| 文件 | 修改内容 |
|------|----------|
| `os/src/trap/context.rs` | 添加 `f: [usize; 32]` 和 `fcsr: usize` 字段到 `TrapContext` |
| `os/src/task/context.rs` | 添加 `f: [usize; 32]` 和 `fcsr: usize` 字段到 `TaskContext` |
| `os/src/trap/trap.S` | 添加 `.option arch, +d` 和 `.option arch, +f`，在 `__alltraps` 和 `__restore` 中保存/恢复 f0-f31 和 fcsr |
| `os/src/task/switch.S` | 添加 `.option arch, +d` 和 `.option arch, +f`，在 `__switch` 中保存/恢复 f0-f31 和 fcsr |
| `os/src/trap/mod.rs` | 在 `init()` 中使用内联汇编设置 sstatus 的 FS 位为 Dirty |
| `os/src/config.rs` | 将 `MAX_APP_NUM` 从 4 改为 5（因为新增了浮点测试程序） |
| `os/Makefile` | 添加 `RUSTFLAGS += -C target-feature=+d,+f`（虽然主要通过 `.option` 实现） |
| `user/src/bin/04float_test.rs` | 新建浮点测试程序 |

### 实现原理

**RISC-V 浮点寄存器**：
- 32 个浮点寄存器：f0-f31（每个 8 字节，双精度）
- 1 个浮点控制状态寄存器：fcsr

**两处需要保存/恢复浮点状态**：

1. **Trap 处理**（`trap.S`）：
   - 用户态→内核态切换时保存用户浮点状态
   - 内核态→用户态切换时恢复用户浮点状态
   - TrapContext 总大小：69*8 = 552 字节

2. **任务切换**（`switch.S`）：
   - 切换出任务时保存其浮点状态
   - 切换入任务时恢复其浮点状态
   - TaskContext 偏移：f0-f31 从 14*8 开始，fcsr 在 46*8

### 浮点单元初始化

**位置**：`os/src/trap/mod.rs::init()`

```rust
// 设置 sstatus 的 FS 位为 Dirty (0b11)
// 确保浮点状态在上下文切换时被保存
unsafe {
    let mut sstatus: usize;
    core::arch::asm!(
        "csrr {}, sstatus",
        out(reg) sstatus,
    );
    sstatus |= 0b11 << 13;  // 设置 FS 位
    core::arch::asm!(
        "csrw sstatus, {}",
        in(reg) sstatus,
    );
}
```

**sstatus.FS 字段含义**：
- Off (00)：浮点单元关闭，访问浮点寄存器会触发非法指令异常
- Initial (01)：首次使用
- Clean (10)：浮点状态未修改
- Dirty (11)：浮点状态已修改，需要在上下文切换时保存

### 浮点测试程序

**位置**：`user/src/bin/04float_test.rs`

```rust
fn main() -> i32 {
    let pi: f64 = 3.14159265359;
    let e: f64 = 2.71828182846;
    let mut result = pi + e;

    // 通过多次 yield_() 触发任务切换
    for i in 0..5 {
        result = result * 1.1;
        println!("[Iteration {}] result = {}", i, result);
        yield_();
    }

    // 验证浮点值没有被破坏
    let expected = (pi + e) * 1.61051_f64;  // 1.1^5 = 1.61051
    let diff = if result > expected { result - expected } else { expected - result };
    
    if diff < 0.0001 {
        println!("Float test PASSED!");
        0
    } else {
        println!("Float test FAILED!");
        -1
    }
}
```

### 运行效果

```
=== Float Test Application ===
Initial calculation:
  pi  = 3.14159265359
  e   = 2.71828182846
  pi + e = 5.8598744820499995

Starting task switch test...
[Iteration 0] result = 6.445861930255
[Iteration 1] result = 7.090448123280501
[Iteration 2] result = 7.799492935608551
[Iteration 3] result = 8.579442229169407
[Iteration 4] result = 9.437386452086349

Final result: 9.437386452086349
Expected: 9.437386452086345
Difference: 0.000000000000003552713678800501
Float test PASSED!
[kernel] Application 04float_test (task 4) exited with code 0
```

### 关键技术点

**1. 汇编宏的限制**

Rust 的 `global_asm!` 宏对复杂的汇编宏支持有限：
- `.rept` 配合 `\n` 转义符会与 Rust 字符串转义冲突
- `.altmacro` 的 `irps` 指令不被支持
- **解决方案**：直接展开所有 32 个浮点寄存器的保存/恢复指令

**2. 浮点扩展启用**

在汇编文件开头添加：
```assembly
.option arch, +d
.option arch, +f
```

或者在 Makefile 中添加：
```makefile
RUSTFLAGS += -C target-feature=+d,+f
```

**3. 输出交织现象**

**问题**：多任务并发时，`println!` 输出会出现混乱，如：
```
power_3 [power_5 [10000/140000]
```

**原因**：`println!` 不是原子操作：
1. 用户态格式化字符串
2. 调用 sys_write 系统调用
3. 内核逐字符输出到串口
4. 步骤 2-3 之间可能被抢占

**解决方案**（未实现）：在控制台输出层加锁，确保同一时间只有一个任务可以输出。

### 验证结果

浮点测试程序成功通过，证明了：
- 浮点寄存器在 trap 切换时被正确保存/恢复
- 浮点寄存器在任务切换时被正确保存/恢复
- 浮点运算结果正确，精度损失在可接受范围内

## 第三章编程题第四题：任务切换开销统计（尝试记录）

### 题目要求
编写应用程序或扩展内核，能够统计任务切换的大致开销。

### 尝试过程

#### 1. 初步编译错误
用户在实现过程中遇到编译错误：
```
error: missing documentation for an associated function
   --> src/batch.rs:102:5
    |
102 |     pub const fn new() -> Self {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
```
原因：`AppTimeStats::new()` 缺少文档注释，而项目启用了 `#![deny(missing_docs)]`。

#### 2. 第二个编译错误
修复后遇到另一个错误：
```
error[E0599]: no method named `as_ptr` found for struct `RefMut<'_, usize>`
```
原因：`TOTAL_SWITCH_OVERHEAD.exclusive_access()` 返回 `RefMut`，不能直接调用 `as_ptr()`。

#### 3. 尝试修复方案
- 方案1：使用 `&raw const TOTAL_SWITCH_OVERHEAD` 获取地址，但获取的是 `UPSafeCell` 结构体地址而非内部值地址
- 方案2：修改 `switch.S` 将开销存回 `switch_temp` 字段，在 Rust 代码中累加

#### 4. 运行时错误
编译通过后运行时出现：
```
Exception(InstructionFault), stval = 0x2
```
问题根源：`TaskContext` 结构体被扩大（添加了浮点寄存器和 `switch_temp` 字段），但 `switch.S` 和 `trap.S` 的偏移量不匹配。

#### 5. 问题分析
- `TaskContext` 从 112 字节扩大到 384 字节（添加 f[32] + fcsr + switch_temp）
- `TrapContext` 也被扩大（添加时间字段）
- 但汇编代码中的偏移量没有正确同步

### 关键教训
1. 修改结构体大小时，必须同步更新所有相关汇编代码中的偏移量
2. `&raw const` 获取的是结构体地址，不是内部值的地址
3. 使用 git 前要谨慎，避免误删工作

### 未完成状态
由于误操作 `git checkout .`，所有修改丢失，需要重新实现。

