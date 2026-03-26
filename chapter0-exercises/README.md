# 第零章 编程练习

> 本目录包含 rCore-Tutorial-Book-v3 第零章的编程练习答案
> 
> 练习来源: https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/7exercise.html

## 练习列表

| 练习 | 难度 | 文件 | 说明 |
|------|------|------|------|
| 练习1 | * | `ex1_env_setup.rs` | 实验环境配置与检测 |
| 练习2 | * | `ex2_exception.rs` | 产生异常的应用程序 |
| 练习3 | ** | `ex3_sleep_print_file.rs` | 睡眠5秒后打印字符串并存入文件 |
| 练习4 | *** | `ex4_os_features.rs` | 体现操作系统四大特征的程序 |

## 编译与运行

### 编译所有程序
```bash
cd chapter0-exercises
cargo build
```

### 运行各个练习

#### 练习1：环境配置检测
```bash
cargo run --bin ex1_env_setup
```
检测 Rust 工具链、QEMU、GDB 等是否正确安装。

#### 练习2：异常演示
```bash
# 除零异常
cargo run --bin ex2_exception -- 1

# 数组越界
cargo run --bin ex2_exception -- 2

# 空指针解引用
cargo run --bin ex2_exception -- 3

# 栈溢出
cargo run --bin ex2_exception -- 4
```

#### 练习3：睡眠打印存文件
```bash
cargo run --bin ex3_sleep_print_file
```
程序会睡眠5秒，然后打印字符串并将内容存入 `output.txt` 文件。

#### 练习4：操作系统特征演示
```bash
cargo run --bin ex4_os_features --release
```
演示操作系统的并发性、异步性、共享性和持久性。

## GDB 调试练习3

```bash
# 编译
cargo build --bin ex3_sleep_print_file

# 启动 GDB
gdb ./target/debug/ex3_sleep_print_file

# GDB 命令
(gdb) break main          # 设置断点
(gdb) run                 # 运行程序
(gdb) step                # 单步执行
(gdb) next                # 执行下一行
(gdb) print message       # 显示变量
(gdb) continue            # 继续执行
(gdb) quit                # 退出
```

## 练习详细说明

### 练习1：实验环境配置

**要求**: 在日常使用的操作系统环境中安装并配置好实验环境。

**解决方案**: 
- 程序检测 Rust、QEMU、GDB 等工具是否正确安装
- 提供详细的环境配置指南
- 列出常见问题及解决方法

### 练习2：异常程序

**要求**: 在Linux环境下编写一个会产生异常的应用程序。

**解决方案**:
- 除零异常 (SIGFPE / panic)
- 数组越界访问 (panic)
- 空指针解引用 (SIGSEGV)
- 栈溢出 (SIGSEGV)

### 练习3：睡眠打印存文件

**要求**: 编写一个可以睡眠5秒后打印出字符串，并把字符串内容存入文件的应用程序。

**解决方案**:
- 使用 `thread::sleep()` 实现睡眠
- 使用 `println!()` 打印到控制台
- 使用 `std::fs::File` 写入文件

### 练习4：操作系统特征

**要求**: 编写一个体现操作系统并发性、异步性、共享性和持久性的应用程序。

**解决方案**:
1. **并发性**: 创建多个线程并行执行
2. **异步性**: 线程完成顺序由操作系统调度决定
3. **共享性**: 多个线程共享计数器、文件、CPU、终端
4. **持久性**: 数据写入文件后，程序结束仍可读取

## 知识点总结

- **系统调用**: sleep (nanosleep), print (write), file (open/read/write/close)
- **进程管理**: 线程创建、调度、同步
- **内存管理**: 共享内存、栈溢出检测
- **文件系统**: 文件读写、持久化存储
- **异常处理**: 信号机制、panic 处理