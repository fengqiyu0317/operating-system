#![no_main]
#![no_std]

#[macro_use]
mod console;
mod lang_items;
mod sbi;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    print_memory_layout();
    panic!("Shutdown machine!");
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as *const () as usize..ebss as *const () as usize).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0)
    });
}

fn print_memory_layout() {
    unsafe extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
    }

    info!(".text   [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
    warn!(".data   [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
    error!(".bss    [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);
}