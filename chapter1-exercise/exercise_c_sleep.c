// 练习C: 基于rcore tutorial的应用程序，使用sleep系统调用睡眠5秒
// 这是为rCore-Tutorial-v3 ch1分支编写的用户态应用程序

#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
pub fn main() -> i32 {
    println!("Exercise C: Sleep system call test");
    println!("====================================");

    // 打印开始信息
    println!("开始睡眠...");

    // 使用sleep系统调用睡眠5秒（5000毫秒）
    // 注意：具体的sleep API取决于rCore教程的实现
    // 常见的实现可能是：sleep(seconds) 或 usleep(milliseconds)
    sleep(5);

    println!("睡眠结束！已睡眠5秒");

    println!("测试完成");
    0
}
