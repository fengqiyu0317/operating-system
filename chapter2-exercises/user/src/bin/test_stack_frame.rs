#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Starting recursive test...");
    recursive(3);
    0
}

fn recursive(n: i32) -> i32 {
    if n <= 0 {
        println!("Base case reached");
        return 0;
    }
    println!("recursive({})", n);
    recursive(n - 1) + 1
}
