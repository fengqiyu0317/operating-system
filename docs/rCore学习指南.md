# rCore 开发环境学习指南

> 本文档介绍如何使用 rCore 开发环境学习操作系统原理

---

## 目录

1. [学习资源](#1-学习资源)
2. [推荐学习路径](#2-推荐学习路径)
3. [实践练习](#3-实践练习)
4. [常用命令](#4-常用命令)
5. [调试技巧](#5-调试技巧)
6. [常见问题](#6-常见问题)

---

## 1. 学习资源

### 官方教程

- **rCore-Tutorial-Book-v3**: https://rcore-os.cn/rCore-Tutorial-Book-v3/
- **rCore-Tutorial-v3 代码**: https://github.com/rcore-os/rCore-Tutorial-v3

### 推荐书籍

- 《操作系统概念》（Operating System Concepts）
- 《现代操作系统》（Modern Operating Systems）
- 《深入理解计算机系统》（CSAPP）

### 视频课程

- 清华大学操作系统课程（公开课）
- rCore-Tutorial 配套视频讲解

---

## 2. 推荐学习路径

### 章节与代码对应关系

| 章节 | 主题 | 核心概念 | 对应代码目录 |
|------|------|----------|-------------|
| 第1章 | 操作系统概述 | 什么是OS、目标、功能 | - |
| 第2章 | 批处理系统 | 特权级、系统调用 | `os/src/` 基础结构 |
| 第3章 | 多道程序与分时 | 任务切换、trap | `os/src/task/`, `os/src/trap/` |
| 第4章 | 地址空间 | 虚拟内存、页表 | `os/src/mm/` |
| 第5章 | 文件系统 | 文件、目录、磁盘 | `easy-fs/`, `os/src/fs/` |
| 第6章 | 进程管理 | 进程、fork、exec | `os/src/process/` |
| 第7章 | 进程间通信 | 管道、信号 | `os/src/ipc/` |
| 第8章 | 并发 | 锁、信号量、死锁 | `os/src/sync/` |
| 第9章 | 设备驱动 | 块设备、GPU、网络 | `os/src/drivers/` |

### 学习步骤

1. **阅读教程章节**
   - 先理解概念和原理
   - 重点关注设计思路

2. **查看代码实现**
   ```bash
   cd rCore-Tutorial-v3
   code .  # 用 VS Code 打开项目
   ```

3. **运行并观察**
   ```bash
   cd os
   make run
   ```

4. **修改并实验**
   - 修改代码中的关键部分
   - 重新编译运行，观察变化

---

## 3. 实践练习

### 3.1 运行示例程序

启动 rCore 后，在 shell 中输入程序名：

```
Rust user shell
> hello_world
> forktest
> gui_snake
```

### 3.2 示例程序分类

| 类别 | 程序名称 | 说明 |
|------|----------|------|
| 基础 | `hello_world`, `exit`, `yield` | 基本系统调用 |
| 进程 | `forktest`, `forktree`, `threads` | 进程创建与管理 |
| 同步 | `adder_mutex`, `phil_din_mutex`, `barrier_condvar` | 并发与同步 |
| 文件 | `cat`, `filetest_simple`, `huge_write` | 文件系统操作 |
| 网络 | `tcp_simplehttp`, `udp` | 网络编程 |
| GUI | `gui_snake`, `gui_move`, `gui_shape` | 图形界面 |
| 协程 | `stackless_coroutine`, `stackful_coroutine` | 异步编程 |

### 3.3 实验练习

按照教程要求，从零实现以下操作系统：

#### 实验一：批处理操作系统
- 实现简单的批处理调度
- 理解特权级切换

#### 实验二：多道程序操作系统
- 实现协作式调度
- 理解任务切换机制

#### 实验三：虚拟内存管理
- 实现地址空间抽象
- 理解页表映射

#### 实验四：文件系统
- 实现简单的文件系统
- 理解文件和目录管理

#### 实验五：进程管理
- 实现进程创建（fork）
- 实现进程执行（exec）

---

## 4. 常用命令

### 编译与运行

```bash
# 进入 os 目录
cd rCore-Tutorial-v3/os

# 编译
make

# 运行
make run

# 清理
make clean
```

### 调试

```bash
# 启动调试模式
make debug

# 在另一个终端连接 GDB
riscv64-unknown-elf-gdb
(gdb) target remote :1234
```

### 文档生成

```bash
# 生成代码文档
cargo doc --open
```

### 代码分析

```bash
# 查看汇编输出
cargo objdump --release -- -d

# 查看符号表
cargo nm --release

# 查看二进制大小
cargo size --release
```

---

## 5. 调试技巧

### 5.1 使用 println! 调试

在内核代码中添加打印语句：

```rust
println!("[DEBUG] current task id: {}", current_task_id);
```

### 5.2 使用 GDB 调试

```bash
# 终端1：启动 QEMU 等待调试
make debug

# 终端2：连接 GDB
riscv64-unknown-elf-gdb
(gdb) target remote :1234
(gdb) break main
(gdb) continue
```

### 5.3 常用 GDB 命令

| 命令 | 说明 |
|------|------|
| `break main` | 在 main 函数设置断点 |
| `continue` | 继续执行 |
| `step` | 单步执行（进入函数） |
| `next` | 单步执行（不进入函数） |
| `backtrace` | 查看调用栈 |
| `info registers` | 查看寄存器 |
| `x/10i $pc` | 查看当前指令 |

---

## 6. 常见问题

### Q1: 编译时提示找不到目标

```bash
rustup target add riscv64gc-unknown-none-elf
```

### Q2: QEMU 无法启动

检查 QEMU 版本是否 >= 7.0：

```bash
qemu-system-riscv64 --version
```

### Q3: 如何退出 QEMU

按 `Ctrl+A`，然后按 `X`

### Q4: 如何切换教程章节

```bash
# 查看所有分支
git branch -a

# 切换到对应章节的分支
git checkout ch3
```

### Q5: 代码修改后没有生效

```bash
# 清理后重新编译
make clean
make
```

---

## 附录：项目结构

```
rCore-Tutorial-v3/
├── os/                 # 内核代码
│   ├── src/
│   │   ├── main.rs     # 入口
│   │   ├── task/       # 任务管理
│   │   ├── mm/         # 内存管理
│   │   ├── fs/         # 文件系统
│   │   ├── process/    # 进程管理
│   │   ├── sync/       # 同步原语
│   │   ├── drivers/    # 设备驱动
│   │   └── trap/       # 中断处理
│   └── Makefile
├── user/               # 用户程序
│   └── src/bin/        # 示例程序
├── easy-fs/            # 文件系统实现
├── bootloader/         # 启动加载器
└── Makefile
```

---

## 参考链接

- [rCore-Tutorial-Book-v3 官方文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [rCore-Tutorial-v3 GitHub](https://github.com/rcore-os/rCore-Tutorial-v3)
- [Rust 官方网站](https://www.rust-lang.org/)
- [RISC-V 规范](https://riscv.org/)