# 操作系统实验 Lab1 报告

## 实验内容总结

本次实验主要完成了裸机操作系统的初始化和彩色日志输出功能的实现：

1. **裸机 Hello World 输出**：实现从 RustSBI 启动后的第一个用户程序输出
2. **彩色日志宏实现**：基于 ANSI 转义序列实现了 `error!`, `warn!`, `info!`, `debug!` 四个彩色输出宏
3. **内存布局输出**：通过链接器符号获取并显示内核各内存段（`.text`, `.rodata`, `.data`, `.bss`）的地址范围

## 运行结果

### 编译与运行

```bash
$ cd os
$ make build
$ make run
```

### 输出示例

```
[rustsbi] RustSBI version 0.3.1, adapting to RISC-V SBI v1.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.2
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 1
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87000000..0x87000ef2
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
[rustsbi] pmp04: 0x88000000..0x00000000 (-wr)
Hello, world!
[INFO] .text   [0x80200000, 0x80202000)
[DEBUG] .rodata [0x80202000, 0x80203000)
[WARN] .data   [0x80203000, 0x80203000)
[ERROR] .bss    [0x80213000, 0x80213000)
Panicked at src/main.rs:17 Shutdown machine!
```

### 内存布局说明

| 段名 | 起始地址 | 结束地址 | 大小 | 用途 | 颜色 |
|------|----------|----------|------|------|------|
| `.text` | 0x80200000 | 0x80202000 | 8 KB | 代码段 | 蓝色 |
| `.rodata` | 0x80202000 | 0x80203000 | 4 KB | 只读数据 | 绿色 |
| `.data` | 0x80203000 | 0x80203000 | 0 B | 已初始化数据 | 黄色 |
| `.bss` | 0x80213000 | 0x80213000 | 0 B | 未初始化数据 | 红色 |

## 实现细节

### 1. ANSI 彩色输出实现

使用 ANSI 转义序列实现终端彩色输出：

```rust
// ANSI 颜色码定义
pub const ANSI_COLOR_RED: &str = "\x1b[31m";
pub const ANSI_COLOR_GREEN: &str = "\x1b[32m";
pub const ANSI_COLOR_YELLOW: &str = "\x1b[33m";
pub const ANSI_COLOR_BLUE: &str = "\x1b[34m";
pub const ANSI_COLOR_RESET: &str = "\x1b[0m";

// 彩色打印函数
pub fn print_color(color: &str, args: fmt::Arguments) {
    print!("{}{}{}", color, args, ANSI_COLOR_RESET);
}
```

### 2. 日志宏实现

使用 Rust 声明宏实现类型安全的格式化输出：

```rust
#[macro_export]
macro_rules! info {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_BLUE,
            format_args!(concat!("[INFO] ", $fmt, "\n") $(, $($arg)+)?)
        );
    };
}
```

### 3. 内存布局输出

通过 `extern "C"` 声明获取链接器定义的符号地址：

```rust
unsafe extern "C" {
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
    info!(".text   [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
    debug!(".rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
    warn!(".data   [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
    error!(".bss    [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);
}
```

### 4. 链接脚本配置

[linker.ld](../os/src/linker.ld) 定义了内核内存布局：

```ld
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS {
    . = BASE_ADDRESS;

    .text : { *(.text.entry) *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) *(.srodata .srodata.*) }
    .data : { *(.data .data.*) *(.sdata .sdata.*) }
    .bss : { *(.bss.stack) sbss = .; *(.bss .bss.*) *(.sbss .sbss.*) }
}
```

## 问答作业

### 1. GDB 调试跟踪

使用 GDB 跟踪从机器加电到跳转到 0x80200000 的过程：

```bash
# 启动 QEMU（等待 GDB 连接）
$ qemu-system-riscv64 -machine virt -nographic -bios rustsbi-qemu.bin -kernel os -s -S

# 另一个终端启动 GDB
$ riscv64-unknown-elf-gdb os
(gdb) target remote :1234
(gdb) break *0x80200000
(gdb) continue
```

**关键跳转过程：**

1. **0x1000** - QEMU 启动入口，加载 RustSBI
2. **0x80000000** - RustSBI 固件执行，初始化硬件
3. **0x80200000** - 跳转到内核入口点（_start）

### 2. GDB 常用命令

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

## 实验环境

- **开发语言**：Rust (nightly)
- **目标架构**：RISC-V 64位 (rv64gc)
- **模拟器**：QEMU 7.0.0
- **Bootloader**：RustSBI 0.3.1
- **工具链**：riscv64-unknown-elf-gcc 8.3.0

## 项目结构

```
.
├── os/                        # 内核实现
│   ├── .cargo/
│   │   └── config             # Rust 工具链配置
│   ├── Cargo.toml             # 项目依赖配置
│   ├── Makefile               # 构建脚本
│   └── src/
│       ├── console.rs         # 彩色输出实现
│       ├── entry.asm          # 汇编入口
│       ├── lang_items.rs      # 语言项处理
│       ├── linker.ld          # 链接脚本
│       ├── main.rs            # 主函数
│       └── sbi.rs             # SBI 调用封装
├── bootloader/                # RustSBI 固件
├── reports/                   # 实验报告
│   └── lab1.md               # 本报告
├── docs/                      # 教程文档
│   └── lab1-tutorial.md      # Lab1 教程
└── README.md                  # 项目说明

```

## 实验心得

（可选部分，可在此处填写实验心得体会）
