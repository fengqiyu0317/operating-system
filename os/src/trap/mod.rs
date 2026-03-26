//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;

use crate::batch::{print_exception_details, record_exception, run_next_app, ExceptionType};
use crate::syscall::syscall;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as *const () as usize, TrapMode::Direct);
    }
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        // 指令地址未对齐异常 (InstructionMisaligned)
        Trap::Exception(Exception::InstructionMisaligned) => {
            record_exception(ExceptionType::InstructionAddressMisaligned, cx.sepc, stval);
            print_exception_details(ExceptionType::InstructionAddressMisaligned, cx.sepc, stval);
            run_next_app();
        }
        // 指令页错误 (InstructionPageFault)
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
        // 加载页错误 (LoadPageFault)
        Trap::Exception(Exception::LoadPageFault) => {
            record_exception(ExceptionType::LoadAccessFault, cx.sepc, stval);
            print_exception_details(ExceptionType::LoadAccessFault, cx.sepc, stval);
            run_next_app();
        }
        // 存储地址未对齐 (StoreMisaligned)
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
            // 未知异常类型，也记录下来
            record_exception(ExceptionType::Unknown, cx.sepc, stval);
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}

pub use context::TrapContext;
