# rCore-Tutorial-v3 第一章实验教程

## 实验概述

本章实验的主要目标是：
1. 完成实验指导书中的内容，在裸机上实现 `hello world` 输出
2. 实现彩色输出宏
3. 利用彩色输出宏输出 OS 内存空间布局

## 实验环境

- 开发语言：Rust
- 目标架构：RISC-V 64位
- 模拟器：QEMU
- Bootloader：RustSBI

## 项目结构

```
os/
├── Cargo.toml       # Rust 项目配置文件
├── Makefile         # 构建配置
├── src/
│   ├── main.rs      # 内核主函数
│   ├── console.rs   # 控制台输出模块
│   ├── sbi.rs       # SBI 调用封装
│   ├── lang_items.rs # 语言项处理
│   ├── entry.asm    # 汇编入口
│   └── linker.ld    # 链接脚本
└── target/          # 编译输出目录
```

## 实验步骤

### 第一步：理解当前代码结构

#### 1.1 链接脚本分析 ([linker.ld](../os/src/linker.ld))

链接脚本定义了内核在内存中的布局：

```ld
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;    # 内核加载地址
```

关键内存段：
- `.text`：代码段，存放指令
- `.rodata`：只读数据段
- `.data`：数据段，存放已初始化的全局变量
- `.bss`：BSS 段，存放未初始化的全局变量

各段的边界符号：
- `stext` / `etext`：代码段起止
- `srodata` / `erodata`：只读数据段起止
- `sdata` / `edata`：数据段起止
- `sbss` / `ebss`：BSS 段起止

#### 1.2 主函数分析 ([main.rs](../os/src/main.rs))

```rust
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();              // 清零 BSS 段
    println!("Hello, world!"); // 输出欢迎信息
    panic!("Shutdown machine!"); // 关机
}
```

### 第二步：实现彩色输出宏

#### 2.1 ANSI 转义序列原理

ANSI 转义序列是一种控制终端格式的标准。彩色输出的格式：

```
\x1b[颜色码m内容\x1b[0m
```

常见颜色码：
- `31`：红色
- `32`：绿色
- `33`：黄色
- `34`：蓝色
- `90`：灰色
- `93`：亮黄色
- `0`：重置所有属性

#### 2.2 修改 console.rs

在 [console.rs](../os/src/console.rs) 中添加彩色输出功能：

```rust
// ANSI 颜色码
const ANSI_COLOR_RED: &str = "\x1b[31m";
const ANSI_COLOR_GREEN: &str = "\x1b[32m";
const ANSI_COLOR_YELLOW: &str = "\x1b[33m";
const ANSI_COLOR_BLUE: &str = "\x1b[34m";
const ANSI_COLOR_GRAY: &str = "\x1b[90m";
const ANSI_COLOR_RESET: &str = "\x1b[0m";

// 彩色打印函数
pub fn print_color(color: &str, args: fmt::Arguments) {
    print!("{}{}{}", color, args, ANSI_COLOR_RESET);
}
```

#### 2.3 实现日志等级宏

在 [console.rs](../os/src/console.rs) 中添加日志宏：

```rust
#[macro_export]
macro_rules! error {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_RED,
            format_args!(concat!("[ERROR] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_YELLOW,
            format_args!(concat!("[WARN] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! info {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_BLUE,
            format_args!(concat!("[INFO] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_GREEN,
            format_args!(concat!("[DEBUG] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! trace {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_GRAY,
            format_args!(concat!("[TRACE] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}
```

### 第三步：输出内存布局

#### 3.1 在 main.rs 中获取内存段信息

需要从链接脚本中获取各段的地址。在 [main.rs](../os/src/main.rs) 中添加：

```rust
extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
}

fn print_memory_layout() {
    info!(".text   [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    warn!(".data   [{:#x}, {:#x})", sdata as usize, edata as usize);
    error!(".bss    [{:#x}, {:#x})", sbss as usize, ebss as usize);
}
```

#### 3.2 修改 rust_main 函数

```rust
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    print_memory_layout();  // 输出内存布局
    panic!("Shutdown machine!");
}
```

### 第四步：完善 Makefile

修改 [Makefile](../os/Makefile) 支持 LOG 参数：

```makefile
.PHONY: run build clean

APP=target/riscv64gc-unknown-none-elf/release/os
BOOTLOADER=../bootloader/rustsbi-qemu.bin
LOG?=INFO

build:
	cargo build --release

run: build
	qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -kernel $(APP)

clean:
	cargo clean
```

注意：LOG 参数的实际控制需要在代码中实现条件编译或运行时检查。

## 实验检查清单

### 编译和运行

```bash
cd os
make build
make run
```

### 预期输出

程序运行后应该看到：
1. "Hello, world!" 的输出
2. 彩色的内存布局信息：
   - `.text` 段范围（蓝色/INFO 级别）
   - `.rodata` 段范围（绿色/DEBUG 级别）
   - `.data` 段范围（黄色/WARN 级别）
   - `.bss` 段范围（红色/ERROR 级别）

### 输出示例

```
Hello, world!
[INFO] .text   [0x80200000, 0x80200100)
[DEBUG] .rodata [0x80200200, 0x80200300)
[WARN] .data   [0x80200400, 0x80200500)
[ERROR] .bss    [0x80200600, 0x80200700)
Panicked at main.rs:17 Shutdown machine!
```

## GDB 调试技巧

### 启动调试

在 Makefile 中添加 debug 目标：

```makefile
debug: build
	qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -kernel $(APP) -s -S
```

然后在新终端启动 GDB：

```bash
riscv64-unknown-elf-gdb $(APP)
(gdb) target remote :1234
```

### 常用 GDB 命令

| 命令 | 功能 |
|------|------|
| `x/10i 0x80000000` | 显示 0x80000000 处的10条汇编指令 |
| `x/10i $pc` | 显示即将执行的10条汇编指令 |
| `x/10xw 0x80000000` | 显示 0x80000000 处的10个数据字（16进制） |
| `info registers` | 显示当前所有寄存器信息 |
| `info reg t0` | 显示 t0 寄存器的值 |
| `break rust_main` | 在 rust_main 函数处设置断点 |
| `break *0x80200000` | 在 0x80200000 地址处设置断点 |
| `continue` | 继续执行直到碰到断点 |
| `si` | 单步执行一条汇编指令 |

## 课后练习参考

### 编程题

1. **实现 ls 程序**：显示当前目录下的文件名
   - 提示：使用 Rust 的 `std::fs` 模块

2. **实现调用栈打印程序**
   - 提示：使用 `backtrace` crate

3. **实现 sleep 系统调用**
   - 需要在内核中实现系统调用处理

### 问答题要点

1. **应用程序占用的资源**：CPU时间、内存、I/O设备等
2. **地址空间分析**：使用 `readelf` 或 `objdump` 工具
3. **应用程序与操作系统的区别**：
   - 应用程序：为用户完成特定任务
   - 操作系统：管理硬件资源，为应用提供服务
4. **RISC-V 加电后的执行流程**：
   - 0x1000：RustSBI 入口
   - 0x80000000：OpenSBI 固件
   - 0x80200000：内核入口
5. **SBI 的功能**：提供机器模式的特权操作接口
6. **操作系统与编译器的协议**：
   - 调用约定
   - 内存布局
   - 系统调用接口
7. **从加电到应用的执行流程**：
   - 硬件初始化 → Bootloader → 内核加载 → 应用执行
8. **不需要手动建立栈的原因**：
   - 编译器和操作系统自动处理
   - 链接器定义栈空间
   - 启动代码初始化栈指针

## 实验报告要求

1. 简要总结实验内容（5行以内）
2. 附上彩色输出的截图
3. 完成问答作业
4. （可选）对实验设计和难度的反馈

## 扩展挑战

### Challenge：支持多核启动

实现多个 Hart（硬件线程）的启动流程：
1. 在 entry.asm 中添加多核启动代码
2. 为每个核分配独立的栈空间
3. 实现核间通信机制

## 常见问题

### Q1：编译失败，提示找不到 sbi-rt

确保 Rust 版本正确，并使用 nightly 工具链：
```bash
rustup override set nightly
```

### Q2：QEMU 启动后没有输出

检查 bootloader 路径是否正确，确保 RustSBI 文件存在。

### Q3：彩色输出不工作

确保终端支持 ANSI 转义序列，或在 QEMU 中使用 `-serial mon:stdio` 参数。

## 参考资料

- [rCore-Tutorial-Book-v3 官方文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [ANSI 转义序列](https://en.wikipedia.org/wiki/ANSI_escape_code)
- [RISC-V 特权架构规范](https://riscv.org/technical/specifications/)
- [RustSBI 项目](https://github.com/rustsbi/rustsbi)
