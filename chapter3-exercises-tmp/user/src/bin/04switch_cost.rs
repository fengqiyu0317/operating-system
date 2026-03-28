#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::yield_;

/// Test task switching cost by calling yield multiple times
/// This program will yield 100 times to generate enough switch statistics
#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("=== Task Switch Cost Test ===");
    println!("This test will yield {} times to trigger task switches.", 100);

    for i in 0..100 {
        if i % 10 == 0 {
            println!("Yield iteration: {}", i);
        }
        yield_();
    }

    println!("Switch cost test completed!");
    println!("Check kernel output for switch statistics.");
    0
}
