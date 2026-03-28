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
//!
//! When the kernel is running, `stvec` is switched to `__alltraps_k` so that
//! interrupts in kernel mode are handled correctly (without sp/sscratch swap).

mod context;

use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, sstatus, stval, stvec,
};

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_user_trap_entry();
}

/// Set `stvec` to kernel trap entry `__alltraps_k`
fn set_kernel_trap_entry() {
    unsafe extern "C" {
        safe fn __alltraps_k();
    }
    unsafe {
        stvec::write(__alltraps_k as usize, TrapMode::Direct);
    }
}

/// Set `stvec` to user trap entry `__alltraps`
fn set_user_trap_entry() {
    unsafe extern "C" {
        safe fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    // Switch stvec to kernel trap entry so interrupts in kernel mode
    // go through __alltraps_k (no sp/sscratch swap)
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // Enable S-mode interrupts so timer interrupts are received in kernel mode
    unsafe {
        sstatus::set_sie();
    }
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                stval, cx.sepc
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // Disable S-mode interrupts before returning to user mode
    unsafe {
        sstatus::clear_sie();
    }
    // Switch stvec back to user trap entry
    set_user_trap_entry();
    cx
}

#[unsafe(no_mangle)]
/// handle an interrupt or exception from kernel space
pub fn trap_handler_k(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            println!(
                "[kernel] Kernel trap: SupervisorTimer at sepc={:#x}",
                cx.sepc
            );
        }
        _ => {
            panic!(
                "Unsupported kernel trap {:?}, stval = {:#x}, sepc = {:#x}!",
                scause.cause(),
                stval,
                cx.sepc
            );
        }
    }
    cx
}

pub use context::TrapContext;
