//! 编程练习2：在Linux环境下编写一个会产生异常的应用程序
//!
//! 本程序演示几种常见的异常：
//! 1. 除零异常（panic）
//! 2. 数组越界访问（panic）
//! 3. 空指针解引用（segmentation fault）
//! 4. 栈溢出（stack overflow）
//!
//! 编译运行方法：
//! ```
//! cargo run --bin ex2_exception -- <异常类型>
//! ```
//!
//! 异常类型：
//! - 1: 除零异常
//! - 2: 数组越界
//! - 3: 空指针解引用（unsafe）
//! - 4: 栈溢出

use std::env;
use std::ptr;

/// 除零异常
fn divide_by_zero() {
    println!("正在执行除零操作...");
    let a: i32 = 10;
    // 使用 std::hint::black_box 阻止编译时优化
    // 这样除零操作会在运行时发生
    let b: i32 = std::hint::black_box(0);
    // Rust 在调试模式下会检查除零并 panic
    // 在 release 模式下，有符号整数除零的行为是未定义的
    let _c = a / b;
    println!("结果: {}", _c); // 这行不会执行
}

/// 数组越界访问
fn array_out_of_bounds() {
    println!("正在访问数组越界索引...");
    let arr = [1, 2, 3, 4, 5];
    // 使用 black_box 阻止编译时检测
    let index: usize = std::hint::black_box(10);
    // Rust 会进行边界检查并 panic
    let _val = arr[index];
    println!("值: {}", _val); // 这行不会执行
}

/// 空指针解引用（需要 unsafe 块）
fn null_pointer_dereference() {
    println!("正在解引用空指针...");
    unsafe {
        let ptr: *const i32 = ptr::null();
        // 解引用空指针会导致段错误
        let _val = *ptr;
        println!("值: {}", _val); // 这行不会执行
    }
}

/// 栈溢出
fn stack_overflow(n: u32) {
    println!("递归深度: {}", n);
    // 创建一个较大的栈上变量来加速栈溢出
    let _buffer = [0u8; 1024];
    // 无限递归
    stack_overflow(n + 1);
}

/// 解引用已释放的内存（use after free）
fn use_after_free() {
    println!("正在访问已释放的内存...");
    unsafe {
        let ptr = Box::into_raw(Box::new(42i32));
        // 释放内存
        drop(Box::from_raw(ptr));
        // 访问已释放的内存（未定义行为）
        let _val = *ptr;
        println!("值: {}", _val);
    }
}

fn print_usage(prog_name: &str) {
    println!("使用方法: {} --bin ex2_exception -- <异常类型>", prog_name);
    println!("异常类型:");
    println!("  1 - 除零异常 (panic / SIGFPE)");
    println!("  2 - 数组越界访问 (panic)");
    println!("  3 - 空指针解引用 (SIGSEGV)");
    println!("  4 - 栈溢出 (SIGSEGV)");
    println!("  5 - 访问已释放内存 (undefined behavior)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        print_usage(&args[0]);
        return;
    }

    let choice: i32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            print_usage(&args[0]);
            return;
        }
    };

    println!("=== 操作系统异常处理演示 ===");
    println!("即将触发异常，操作系统将进行相应的处理...\n");

    match choice {
        1 => divide_by_zero(),
        2 => array_out_of_bounds(),
        3 => null_pointer_dereference(),
        4 => stack_overflow(1),
        5 => use_after_free(),
        _ => print_usage(&args[0]),
    }

    println!("程序正常结束。"); // 对于异常情况，这行不会执行
}

/*
=== 操作系统处理结果说明 ===

1. 除零异常:
   - Rust 在调试模式下会进行除零检查
   - 检测到除零后触发 panic!
   - 程序展开栈并打印错误信息后退出
   - 在 release 模式下可能导致 SIGFPE 信号

2. 数组越界访问:
   - Rust 进行运行时边界检查
   - 检测到越界后触发 panic!
   - 输出: "index out of bounds: the len is 5 but the index is 10"

3. 空指针解引用:
   - 需要 unsafe 块才能进行
   - 操作系统检测到访问无效内存地址
   - 向进程发送 SIGSEGV 信号
   - 终端输出: Segmentation fault (core dumped)

4. 栈溢出:
   - 无限递归导致栈空间耗尽
   - 可能触发 Rust 的栈检查（stack guard）或操作系统的检测
   - 可能输出 "stack overflow" 或 SIGSEGV

5. 访问已释放内存:
   - 这是未定义行为
   - 可能导致段错误或数据损坏
   - 体现了 Rust 安全性的重要性

操作系统通过信号机制来处理这些严重的运行时错误，
保护系统不受错误程序的影响。Rust 的安全机制
可以在编译时和运行时防止大多数这类错误。
*/