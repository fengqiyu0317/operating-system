//! 编程练习3：在Linux环境下编写一个可以睡眠5秒后打印出一个字符串，
//! 并把字符串内容存入一个文件中的应用程序A
//!
//! 编译运行方法：
//! ```
//! cargo run --bin ex3_sleep_print_file
//! ```
//!
//! GDB 调试方法：
//! ```
//! cargo build --bin ex3_sleep_print_file
//! gdb ./target/debug/ex3_sleep_print_file
//! (gdb) break main
//! (gdb) run
//! (gdb) step
//! (gdb) print message
//! ```

use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

/// 要打印和保存的消息
const MESSAGE: &str = "Hello, Operating System! - 来自 rCore 练习";

/// 输出文件名
const OUTPUT_FILE: &str = "output.txt";

fn main() {
    println!("=== 程序A：睡眠5秒后打印字符串并存入文件 ===\n");

    // 记录开始时间
    let start_time = Instant::now();
    println!("[{}] 程序启动", chrono_timestamp());

    // 定义要输出的字符串
    let message = MESSAGE;
    println!("[{}] 准备睡眠5秒...", chrono_timestamp());

    // 睡眠5秒
    // 这会触发系统调用 nanosleep，操作系统会将当前进程挂起
    // 5秒后，操作系统会唤醒该进程继续执行
    thread::sleep(Duration::from_secs(5));

    // 计算实际睡眠时间
    let elapsed = start_time.elapsed();
    println!("[{}] 睡眠结束，实际耗时: {:.2}秒", chrono_timestamp(), elapsed.as_secs_f64());

    // 打印字符串到控制台
    // 这会触发 write 系统调用
    println!("[{}] 打印字符串: {}", chrono_timestamp(), message);

    // 将字符串写入文件
    // 这会触发 open, write, close 等系统调用
    println!("[{}] 正在将字符串写入文件 '{}'...", chrono_timestamp(), OUTPUT_FILE);
    
    match write_to_file(OUTPUT_FILE, message) {
        Ok(_) => {
            println!("[{}] 字符串已成功写入文件: {}", chrono_timestamp(), OUTPUT_FILE);
        }
        Err(e) => {
            eprintln!("[{}] 写入文件失败: {}", chrono_timestamp(), e);
            std::process::exit(1);
        }
    }

    // 读取文件内容验证
    println!("\n--- 验证文件内容 ---");
    match std::fs::read_to_string(OUTPUT_FILE) {
        Ok(content) => {
            println!("文件内容: {}", content);
        }
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
        }
    }

    let total_time = start_time.elapsed();
    println!("\n[{}] 程序执行完毕，总耗时: {:.2}秒", chrono_timestamp(), total_time.as_secs_f64());
}

/// 将内容写入文件
fn write_to_file(filename: &str, content: &str) -> std::io::Result<()> {
    // 创建或打开文件（如果文件存在则截断）
    // 系统调用: open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644)
    let mut file = File::create(filename)?;

    // 写入内容
    // 系统调用: write(fd, content, len)
    file.write_all(content.as_bytes())?;

    // 文件会在作用域结束时自动关闭
    // 系统调用: close(fd)
    
    Ok(())
}

/// 获取当前时间戳字符串
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    
    // 简单格式化：时:分:秒.毫秒
    let total_secs = secs % 86400;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = nanos / 1_000_000;
    
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

/*
=== 程序执行流程分析 ===

1. 程序启动
   - 操作系统加载程序到内存
   - 创建进程控制块(PCB)
   - 分配地址空间

2. 睡眠5秒
   - 调用 thread::sleep() 触发 nanosleep 系统调用
   - 操作系统将进程状态从"运行"改为"阻塞"
   - 进程被移出CPU调度队列
   - 5秒后，操作系统时钟中断处理程序将进程唤醒
   - 进程状态变为"就绪"，等待调度

3. 打印字符串
   - 调用 println!() 触发 write 系统调用
   - 将数据从用户空间复制到内核缓冲区
   - 内核将数据输出到终端设备

4. 写入文件
   - 调用 File::create() 触发 open 系统调用
   - 调用 write_all() 触发 write 系统调用
   - 文件句柄离开作用域时触发 close 系统调用
   - 数据最终写入磁盘（持久化）

=== GDB 调试演示 ===

$ gdb ./target/debug/ex3_sleep_print_file
(gdb) break main                  # 在 main 函数设置断点
Breakpoint 1 at 0xxxxx: file src/bin/ex3_sleep_print_file.rs, line xx

(gdb) run                         # 运行程序
Starting program: ./target/debug/ex3_sleep_print_file
Breakpoint 1, ex3_sleep_print_file::main () at src/bin/ex3_sleep_print_file.rs:xx

(gdb) step                        # 单步执行

(gdb) print message               # 显示变量信息
$1 = "Hello, Operating System! - 来自 rCore 练习"

(gdb) next                        # 执行下一行（不进入函数）

(gdb) continue                    # 继续执行
*/