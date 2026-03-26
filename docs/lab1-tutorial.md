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

**注意**：上面的 Makefile 只是定义了 `LOG` 变量，但还**没有真正实现**日志等级控制。接下来的第五步将教你如何实现。

### 第五步：实现日志等级控制（可选但推荐）

#### 5.1 为什么需要日志等级控制？

在实际开发中，不同场景需要不同的日志详细程度：
- **调试时**：需要看到所有 DEBUG 和 TRACE 日志
- **正常运行**：只需要 INFO 和 WARN
- **发布版本**：只显示 ERROR

日志等级控制可以让你**在不修改代码**的情况下调整输出详细程度。

#### 5.2 方案选择

有两种实现方式：

| 方案 | 运行时开销 | 灵活性 | 实现难度 | 推荐度 |
|------|-----------|--------|----------|--------|
| **运行时检查** | 极小（一次比较） | ⭐⭐⭐⭐⭐ | 简单 | ✅ 强烈推荐 |
| **条件编译** | 零 | ⭐⭐ | 复杂 | 可选 |

**推荐**：先实现运行时检查，满足需求后可以考虑优化为条件编译。

#### 5.3 实现方案 A：运行时检查（推荐）

##### 步骤 1：定义日志等级

在 `console.rs` 中添加等级定义：

```rust
// 日志等级定义（数值越大，优先级越低）
const LOG_ERROR: u8 = 1;
const LOG_WARN: u8 = 2;
const LOG_INFO: u8 = 3;
const LOG_DEBUG: u8 = 4;
const LOG_TRACE: u8 = 5;
const LOG_OFF: u8 = 6;  // 关闭所有日志

// 当前日志等级（可通过 set_log_level 修改）
static mut LOG_LEVEL: u8 = LOG_INFO;  // 默认 INFO 级别
```

**为什么用 `u8` 而不是 `enum`？**
- `u8` 占用空间小（1 字节）
- 比较操作简单快速
- 在裸机环境中更轻量

##### 步骤 2：实现等级设置函数

```rust
// 设置日志等级
pub fn set_log_level(level: &str) {
    unsafe {
        LOG_LEVEL = match level {
            "ERROR" => LOG_ERROR,
            "WARN" => LOG_WARN,
            "INFO" => LOG_INFO,
            "DEBUG" => LOG_DEBUG,
            "TRACE" => LOG_TRACE,
            "OFF" => LOG_OFF,
            _ => LOG_INFO,  // 默认值
        };
    }
}

// 获取当前日志等级
pub fn get_log_level() -> u8 {
    unsafe { LOG_LEVEL }
}
```

**为什么用 `unsafe`？**
- 裸机环境中没有可用的同步原语
- 单核环境下是安全的
- 在 main 函数开始时设置，之后不再修改

##### 步骤 3：修改日志宏，添加等级检查

```rust
// 辅助宏：检查是否应该输出
macro_rules! should_log {
    ($level:expr) => {
        $crate::console::get_log_level() >= $level
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        if $crate::console::should_log!($crate::console::LOG_ERROR) {
            $crate::console::print_color(
                $crate::console::ANSI_COLOR_RED,
                format_args!(concat!("[ERROR] ", $fmt, "\n") $(, $($arg)+)?)
            );
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        if $crate::console::should_log!($crate::console::LOG_WARN) {
            $crate::console::print_color(
                $crate::console::ANSI_COLOR_YELLOW,
                format_args!(concat!("[WARN] ", $fmt, "\n") $(, $($arg)+)?)
            );
        }
    };
}

#[macro_export]
macro_rules! info {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        if $crate::console::should_log!($crate::console::LOG_INFO) {
            $crate::console::print_color(
                $crate::console::ANSI_COLOR_BLUE,
                format_args!(concat!("[INFO] ", $fmt, "\n") $(, $($arg)+)?)
            );
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        if $crate::console::should_log!($crate::console::LOG_DEBUG) {
            $crate::console::print_color(
                $crate::console::ANSI_COLOR_GREEN,
                format_args!(concat!("[DEBUG] ", $fmt, "\n") $(, $($arg)+)?)
            );
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        if $crate::console::should_log!($crate::console::LOG_TRACE) {
            $crate::console::print_color(
                $crate::console::ANSI_COLOR_GRAY,
                format_args!(concat!("[TRACE] ", $fmt, "\n") $(, $($arg)+)?)
            );
        }
    };
}
```

##### 步骤 4：在 main.rs 中初始化日志等级

```rust
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    console::set_log_level("INFO");  // 设置日志等级
    println!("Hello, world!");
    print_memory_layout();
    panic!("Shutdown machine!");
}
```

##### 步骤 5：修改 Makefile 传递 LOG 参数

```makefile
.PHONY: run build clean

APP=target/riscv64gc-unknown-none-elf/release/os
BOOTLOADER=../bootloader/rustsbi-qemu.bin
LOG?=INFO  # 默认 INFO 级别

build:
	cargo build --release

run: build
	@echo "Running with LOG=$(LOG)..."
	qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -kernel $(APP)

# 直接在 Makefile 中修改代码（简单方法）
set-log:
	@sed -i 's/console::set_log_level("[^"]*");/console::set_log_level("$(LOG)");/' src/main.rs

clean:
	cargo clean
```

**使用方法**：
```bash
make run LOG=ERROR   # 只显示 ERROR
make run LOG=INFO    # 显示 INFO, WARN, ERROR
make run LOG=DEBUG   # 显示 DEBUG 及以上
```

**注意**：这个简单方案需要修改源代码。更高级的方案可以在编译时通过环境变量传递。

#### 5.4 实现方案 B：条件编译（高级，零开销）

如果追求极致性能，可以使用条件编译，在编译时完全删除不需要的日志代码。

##### 步骤 1：修改 Makefile

```makefile
LOG?=INFO
RUSTFLAGS?=--cfg log_level="$(LOG)"

build:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release
```

##### 步骤 2：使用 cfg 属性

```rust
// 在 console.rs 中
#[cfg(log_level = "ERROR")]
const CURRENT_LOG_LEVEL: u8 = 1;

#[cfg(log_level = "WARN")]
const CURRENT_LOG_LEVEL: u8 = 2;

#[cfg(log_level = "INFO")]
const CURRENT_LOG_LEVEL: u8 = 3;

#[cfg(log_level = "DEBUG")]
const CURRENT_LOG_LEVEL: u8 = 4;

#[cfg(log_level = "TRACE")]
const CURRENT_LOG_LEVEL: u8 = 5;

// 默认值
#[cfg(not(any(
    log_level = "ERROR",
    log_level = "WARN",
    log_level = "INFO",
    log_level = "DEBUG",
    log_level = "TRACE"
)))]
const CURRENT_LOG_LEVEL: u8 = 3;
```

**优点**：编译时就知道日志等级，不满足条件的日志代码会被完全删除。

**缺点**：修改日志等级需要重新编译。

#### 5.5 日志等级使用示例

```rust
fn main() {
    // 设置为 ERROR 级别
    console::set_log_level("ERROR");

    error!("This will print");    // ✓ 输出
    warn!("This won't print");    // ✗ 不输出
    info!("This won't print");    // ✗ 不输出

    // 修改为 DEBUG 级别
    console::set_log_level("DEBUG");

    error!("This will print");    // ✓ 输出
    warn!("This will print");     // ✓ 输出
    info!("This will print");     // ✓ 输出
    debug!("This will print");    // ✓ 输出
    trace!("This won't print");   // ✗ 不输出
}
```

#### 5.6 日志等级过滤规则

| 设置等级 | ERROR | WARN | INFO | DEBUG | TRACE |
|----------|-------|------|------|-------|-------|
| **ERROR** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **WARN** | ✅ | ✅ | ❌ | ❌ | ❌ |
| **INFO** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **DEBUG** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **TRACE** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **OFF** | ❌ | ❌ | ❌ | ❌ | ❌ |

**规则**：只输出**等级大于或等于**当前设置等级的日志。

#### 5.7 性能分析

**运行时检查方案**：
- 每次日志调用：1 次比较 + 1 次分支
- 开销：< 10 纳秒（现代 CPU）
- 对于操作系统内核来说，这个开销可以忽略不计

**条件编译方案**：
- 不满足条件的日志：0 开销（代码被删除）
- 适合对性能极其敏感的场景

#### 5.8 完整示例代码

查看 `console.rs` 的完整实现：

```rust
// console.rs 完整示例
use core::fmt;

const ANSI_COLOR_RED: &str = "\x1b[31m";
const ANSI_COLOR_GREEN: &str = "\x1b[32m";
const ANSI_COLOR_YELLOW: &str = "\x1b[33m";
const ANSI_COLOR_BLUE: &str = "\x1b[34m";
const ANSI_COLOR_GRAY: &str = "\x1b[90m";
const ANSI_COLOR_RESET: &str = "\x1b[0m";

const LOG_ERROR: u8 = 1;
const LOG_WARN: u8 = 2;
const LOG_INFO: u8 = 3;
const LOG_DEBUG: u8 = 4;
const LOG_TRACE: u8 = 5;

static mut LOG_LEVEL: u8 = LOG_INFO;

pub fn set_log_level(level: &str) {
    unsafe {
        LOG_LEVEL = match level {
            "ERROR" => LOG_ERROR,
            "WARN" => LOG_WARN,
            "INFO" => LOG_INFO,
            "DEBUG" => LOG_DEBUG,
            "TRACE" => LOG_TRACE,
            _ => LOG_INFO,
        };
    }
}

pub fn get_log_level() -> u8 {
    unsafe { LOG_LEVEL }
}

pub fn print_color(color: &str, args: fmt::Arguments) {
    print!("{}{}{}", color, args, ANSI_COLOR_RESET);
}

// 宏定义...
```

#### 5.9 测试日志等级功能

```bash
# 测试 ERROR 级别
cd os
sed -i 's/console::set_log_level("[^"]*");/console::set_log_level("ERROR");/' src/main.rs
make run
# 预期：只看到 ERROR 日志

# 测试 INFO 级别
sed -i 's/console::set_log_level("[^"]*");/console::set_log_level("INFO");/' src/main.rs
make run
# 预期：看到 INFO, WARN, ERROR 日志

# 测试 TRACE 级别
sed -i 's/console::set_log_level("[^"]*");/console::set_log_level("TRACE");/' src/main.rs
make run
# 预期：看到所有日志
```

#### 5.10 常见问题

**Q：为什么日志等级是 u8 而不是枚举？**
A：`u8` 更轻量，比较操作更快。在裸机环境中，性能优先。

**Q：运行时检查会影响性能吗？**
A：影响极小（< 10ns）。如果你确实需要零开销，可以使用条件编译方案。

**Q：如何动态调整日志等级？**
A：可以通过 SBI 调用传递参数，或者在调试时通过修改代码。

**Q：日志等级会影响最终二进制大小吗？**
A：运行时检查不会（所有日志代码都在）。条件编译会（低级日志被删除）。

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
