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

    // 找到字符串结束位置
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let task_name = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };

    println!("Current task ID: {}", task_id);
    println!("Current task name: {}", task_name);

    println!("Test completed successfully!");
    0
}
