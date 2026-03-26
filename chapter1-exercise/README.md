# rCore Tutorial 第一章练习

本目录包含《操作系统》课程第一章练习的编程题和问答题解答。

## 目录结构

```
chapter1-exercise/
├── README.md                    # 本文件
├── 问答题解答.md                # 问答题详细解答
├── 编程题说明.md                # 编程题说明和测试方法
├── exercise_a_ls.rs            # 练习A: 显示当前目录文件
├── exercise_b_backtrace.rs     # 练习B: 打印调用栈
└── exercise_c_sleep.c          # 练习C: sleep系统调用
```

## 课后练习

### 编程题

#### 练习A: 显示当前目录下的文件名
**文件**: [exercise_a_ls.rs](exercise_a_ls.rs)

**功能**: 实现类似`ls`的功能，列出当前目录的所有文件

**编译运行**:
```bash
# 编译
rustc exercise_a_ls.rs -o exercise_a_ls

# 运行
./exercise_a_ls
```

**说明**:
- 使用标准库的`fs::read_dir`读取目录
- 遍历并打印每个文件的名称
- 包含错误处理

#### 练习B: 打印调用栈信息
**文件**: [exercise_b_backtrace.rs](exercise_b_backtrace.rs)

**功能**: 演示如何获取和打印程序的调用栈

**编译运行**:
```bash
# 编译
rustc exercise_b_backtrace.rs -o exercise_b_backtrace

# 运行
./exercise_b_backtrace

# 也可以设置环境变量获取完整backtrace
RUST_BACKTRACE=1 ./exercise_b_backtrace
```

**说明**:
- 方法1: 使用`std::backtrace`模块获取调用栈
- 方法2: 手动追踪调用链
- 演示了多层函数调用的栈结构

#### 练习C: 使用sleep系统调用
**状态**: ✅ 已完成并集成到rCore

**功能**: 在rCore用户程序中使用sleep系统调用睡眠5秒

**文件位置**:
- 概念代码: [exercise_c_sleep.c](exercise_c_sleep.c)
- rCore实现: `rCore-Tutorial-v3/user/src/bin/exercise_c.rs` ✅

**说明**:
- ✅ 已在rCore-Tutorial-v3项目中创建 `exercise_c.rs`
- ✅ 程序使用`sleep(5000)`系统调用（睡眠5000毫秒=5秒）
- ✅ 包含时间测量功能，可验证实际睡眠时间
- ✅ 已验证rCore可以识别并加载该程序

**在rCore中测试**:
```bash
# 进入rCore的os目录
cd ../rCore-Tutorial-v3/os

# 编译并运行rCore
make run

# 在出现的shell中输入
>> exercise_c
```

**预期输出**:
```
===============================================
Exercise C: Sleep System Call Test
===============================================

开始时间: XXXXX ms
准备睡眠 5000 ms (5秒)...

结束时间: XXXXX ms
实际睡眠时间: 5000 ms (约 5 秒)

===============================================
Exercise C: Test Passed!
===============================================
```

**详细测试指南**: 参见 [练习C测试指南.md](练习C测试指南.md)

### 问答题

所有问答题的详细解答请查看: [问答题解答.md](问答题解答.md)

#### 问题列表

1. **应用程序占用的计算机资源**
   - CPU、内存、存储、I/O设备、系统资源等

2. **应用程序地址空间分析**
   - 使用`readelf`、`size`、`pmap`等工具分析
   - 代码段、数据段、堆、栈的地址范围

3. **应用程序与操作系统的异同**
   - 对比表格说明二者的相同点和不同点

4. **RISC-V硬件加电后的执行流程**
   - 从0x1000的Boot ROM到内核执行的完整过程
   - Boot ROM → OpenSBI → rCore内核的跳转链

5. **RISC-V中SBI的含义和功能**
   - SBI (Supervisor Binary Execution Interface) 详解
   - 固件层提供的硬件抽象和运行时服务

6. **操作系统与编译器之间的协议**
   - 系统调用约定、ELF格式、地址空间布局
   - 函数调用约定、启动协议、ABI规范

7. **从加电到应用程序执行的完整过程**
   - 详细的时间线和地址总结
   - 7个关键阶段的执行流程

8. **为什么应用程序员不需要管理栈和地址空间**
   - 操作系统的抽象和自动管理
   - 编译器、链接器、加载器的分工

9. **无帧指针情况下的调用栈复原**
   - 基于栈帧模式分析
   - DWARF调试信息的使用
   - 调试器的栈回溯算法

## 实验练习提示

### 实践作业：彩色化LOG

如果在rCore Tutorial中完成实验练习，需要：

1. **实现彩色输出宏**
   - 使用ANSI转义序列: `\x1b[31m...\x1b[0m`
   - 支持不同日志等级: ERROR(红)、WARN(黄)、INFO(蓝)、DEBUG(绿)

2. **输出内存布局**
   ```rust
   info!(".text [{:#x}, {:#x})", s_text, e_text);
   info!(".rodata [{:#x}, {:#x})", s_rodata, e_rodata);
   info!(".data [{:#x}, {:#x})", s_data, e_data);
   ```

3. **参考实现**
   - 可以查看 `rCore-Tutorial-v3/os/src/` 中的日志实现
   - 参考 `log` crate 的使用方法

### 问答作业：GDB调试

使用GDB跟踪从机器加电到内核执行的跳转过程：

```bash
# 启动调试
cd ../rCore-Tutorial-v3/os
make debug

# 在GDB中执行
(gdb) break *0x80200000  # 在内核入口设置断点
(gdb) continue           # 继续执行
(gdb) x/10i $pc          # 查看当前指令
(gdb) info registers     # 查看寄存器状态
(gdb) si                 # 单步执行
```

## 调试技巧

### 使用GDB调试Rust程序

```bash
# 编译时包含调试信息
rustc -g exercise_b_backtrace.rs -o exercise_b_backtrace

# 启动GDB
gdb ./exercise_b_backtrace

# 常用GDB命令
(gdb) break main          # 在main函数设置断点
(gdb) run                 # 运行程序
(gdb) backtrace           # 查看调用栈
(gdb) info locals         # 查看局部变量
(gdb) step                # 单步执行
```

### 分析可执行文件

```bash
# 查看ELF信息
readelf -h program        # ELF头
readelf -l program        # 程序头（加载信息）
readelf -S program        # 节表

# 查看段大小
size program

# 反汇编
objdump -d program

# 查看动态链接
ldd program
```

## 学习资源

### rCore Tutorial
- 官方文档: https://rcore-os.cn/rCore-Tutorial-Book-v3/
- GitHub仓库: https://github.com/rcore-os/rCore-Tutorial-v3

### RISC-V架构
- RISC-V规范: https://riscv.org/technical/specifications/
- RISC-V特权架构: Volume 2

### 调试工具
- GDB文档: https://www.gnu.org/software/gdb/documentation/
- QEMU文档: https://www.qemu.org/docs/master/

## 答案参考

- 编程题提供了可直接运行的示例代码
- 问答题提供了详细的理论分析和说明
- 建议先独立思考，再参考答案进行验证

## 提交要求

如果这是课程作业，通常需要提交：

1. 编程题的源代码和运行截图
2. 问答题的书面解答
3. 实验练习的代码和报告（包括彩色输出截图）

## 作者说明

本练习解答基于:
- rCore-Tutorial-Book-v3 3.6.0-alpha.1
- RISC-V 64位架构
- QEMU 7.0.0 模拟器
- Rust 1.70+ 编译器

如有问题或建议，欢迎反馈。

---

**完成日期**: 2026-03-13
**课程**: 操作系统
**章节**: 第一章：应用程序与基本执行环境
