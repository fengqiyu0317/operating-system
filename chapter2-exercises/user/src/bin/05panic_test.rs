#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::println;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Hello, I'm going to panic!");
    println!("This should cause the system to hang if panic handler uses loop!");
    panic!("Intentional panic for testing!");
}
