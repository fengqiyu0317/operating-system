//! 编程练习4：编写一个应用程序B，体现操作系统的并发性、异步性、共享性和持久性
//!
//! 本程序演示操作系统的四大特征：
//! 1. 并发性(Concurrency)：多个线程同时执行
//! 2. 异步性(Asynchrony)：事件的发生顺序和时间不可预测
//! 3. 共享性(Sharing)：多个进程/线程共享资源（内存、文件等）
//! 4. 持久性(Persistence)：数据持久化存储在文件系统中
//!
//! 编译运行方法：
//! ```
//! cargo run --bin ex4_os_features --release
//! ```

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 共享文件路径
const SHARED_FILE: &str = "shared_data.txt";

/// 共享计数器 - 用于演示共享性
struct SharedCounter {
    value: Mutex<i32>,
}

impl SharedCounter {
    fn new() -> Self {
        Self {
            value: Mutex::new(0),
        }
    }

    fn increment(&self) -> i32 {
        let mut val = self.value.lock().unwrap();
        *val += 1;
        *val
    }

    fn get(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     操作系统四大特征演示程序 B                              ║");
    println!("║     Concurrency | Asynchrony | Sharing | Persistence       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // 初始化共享文件（持久性准备）
    init_shared_file();

    // 创建共享计数器
    let counter = Arc::new(SharedCounter::new());
    
    // 记录开始时间
    let start_time = Instant::now();
    let main_thread_id = thread::current().id();
    println!("[主线程 {:?}] 程序启动于 {:?}", main_thread_id, start_time);

    // ========== 1. 并发性演示 ==========
    println!("\n【1. 并发性 Concurrency】");
    println!("创建多个线程并行执行，操作系统通过时间片轮转实现并发...\n");

    let mut handles = vec![];
    
    // 创建4个工作线程
    for i in 1..=4 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let thread_id = thread::current().id();
            
            // 每个线程执行不同的任务
            for j in 1..=3 {
                // 模拟工作
                let work_time = Duration::from_millis(50 * i);
                thread::sleep(work_time);
                
                // ========== 3. 共享性演示 ==========
                // 多个线程共享同一个计数器变量
                let count = counter_clone.increment();
                
                // 同时也共享文件资源
                let log_msg = format!(
                    "[线程{} {:?}] 任务{}.{} - 共享计数器={}\n",
                    i, thread_id, i, j, count
                );
                append_to_shared_file(&log_msg);
                
                print!("{}", log_msg);
            }
            
            format!("线程{} 完成", i)
        });
        
        handles.push(handle);
    }

    // 主线程也进行一些工作，体现并发
    println!("[主线程] 在工作线程执行的同时，主线程也在运行...\n");
    
    // ========== 2. 异步性演示 ==========
    println!("【2. 异步性 Asynchrony】");
    println!("线程的执行顺序和完成时间由操作系统调度决定，不可预测...\n");

    // 等待所有线程完成，但它们的完成顺序是异步的
    let mut results = Vec::new();
    for (idx, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                results.push(result);
                println!("  → 线程{} 在 {:?} 完成", idx + 1, elapsed);
            }
            Err(e) => eprintln!("线程 {} 执行出错: {:?}", idx + 1, e),
        }
    }

    println!("\n线程完成顺序（每次运行可能不同，体现异步性）:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, result);
    }

    // ========== 3. 共享性详细演示 ==========
    println!("\n【3. 共享性 Sharing】");
    println!("多个线程共享了以下资源:");
    println!("  - 共享内存: 计数器最终值 = {}", counter.get());
    println!("  - 共享文件: {}", SHARED_FILE);
    println!("  - 共享CPU: 操作系统通过调度让所有线程都能执行");
    println!("  - 共享终端: 所有线程都向同一终端输出\n");

    // 显示共享文件内容
    println!("共享文件内容（所有线程都写入同一文件）:");
    println!("─────────────────────────────────────────");
    match read_shared_file() {
        Ok(content) => print!("{}", content),
        Err(e) => eprintln!("读取文件失败: {}", e),
    }
    println!("─────────────────────────────────────────");

    // ========== 4. 持久性演示 ==========
    println!("\n【4. 持久性 Persistence】");
    println!("数据已持久化保存到文件系统，即使程序结束后数据仍然存在。");
    
    // 写入最终结果到持久化存储
    let final_report = format!(
        "\n=== 最终报告 ===\n\
         时间: {:?}\n\
         共享计数器最终值: {}\n\
         程序执行总耗时: {:?}\n\
         数据已持久化保存。\n",
        chrono_timestamp(),
        counter.get(),
        start_time.elapsed()
    );
    
    append_to_shared_file(&final_report);
    println!("最终报告已追加到文件: {}", SHARED_FILE);

    // 模拟程序结束后数据仍然存在
    println!("\n验证持久性 - 重新读取文件:");
    match read_shared_file() {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            println!("文件共有 {} 行数据（程序结束后数据仍然存在）", lines.len());
        }
        Err(e) => eprintln!("读取失败: {}", e),
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    演示总结                                 ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ 1. 并发性: 多线程同时执行，操作系统进行调度                 ║");
    println!("║ 2. 异步性: 线程完成顺序不确定，由操作系统决定               ║");
    println!("║ 3. 共享性: 线程共享内存(计数器)、文件、CPU、终端            ║");
    println!("║ 4. 持久性: 数据写入文件后，程序结束仍可读取                 ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

/// 初始化共享文件
fn init_shared_file() {
    let mut file = File::create(SHARED_FILE).expect("无法创建共享文件");
    writeln!(file, "=== 操作系统特征演示 - 共享文件 ===").unwrap();
    writeln!(file, "创建时间: {}", chrono_timestamp()).unwrap();
    writeln!(file, "------------------------------------").unwrap();
}

/// 追加内容到共享文件
fn append_to_shared_file(content: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(SHARED_FILE)
        .expect("无法打开共享文件");
    
    file.write_all(content.as_bytes()).expect("写入文件失败");
}

/// 读取共享文件内容
fn read_shared_file() -> std::io::Result<String> {
    let mut file = File::open(SHARED_FILE)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// 获取当前时间戳
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    
    let total_secs = secs % 86400;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = nanos / 1_000_000;
    
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

/*
=== 操作系统特征分析 ===

1. 【并发性 Concurrency】
   - 定义：多个任务在同一时间段内交替执行
   - 体现：程序创建4个线程，它们"同时"执行
   - 实现：操作系统通过时间片轮转调度，快速切换CPU上下文
   - 代码：thread::spawn() 创建新线程

2. 【异步性 Asynchrony】
   - 定义：事件发生的顺序和时间不可预测
   - 体现：每次运行时，线程完成顺序可能不同
   - 原因：操作系统调度决策受多种因素影响（CPU负载、优先级等）
   - 观察：多次运行程序，查看线程完成顺序

3. 【共享性 Sharing】
   - 定义：多个进程/线程可以共享系统资源
   - 体现：
     a) 内存共享：Arc<Mutex<i32>> 计数器被多个线程访问
     b) 文件共享：所有线程写入同一个文件
     c) CPU共享：操作系统调度器分配CPU时间
     d) I/O共享：多个线程向同一终端输出

4. 【持久性 Persistence】
   - 定义：数据在程序结束后仍然存在
   - 体现：程序将数据写入文件，程序结束后可再次读取
   - 实现：操作系统通过文件系统将数据保存到磁盘
   - 对比：内存中的数据在进程结束后会丢失

=== 运行示例 ===

$ cargo run --bin ex4_os_features --release
[主线程 ThreadId(1)] 程序启动于 Instant { ... }

【1. 并发性 Concurrency】
创建多个线程并行执行...

【2. 异步性 Asynchrony】
[线程1 ThreadId(2)] 任务1.1 - 共享计数器=1
[线程2 ThreadId(3)] 任务2.1 - 共享计数器=2
[线程3 ThreadId(4)] 任务3.1 - 共享计数器=3
...

线程完成顺序（每次运行可能不同）:
  1. 线程1 完成
  2. 线程2 完成
  3. 线程3 完成
  4. 线程4 完成

【3. 共享性 Sharing】
  - 共享计数器最终值 = 12

【4. 持久性 Persistence】
数据已持久化保存到文件系统...
*/