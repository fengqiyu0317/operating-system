//! 编程练习1：安装并配置实验环境
//!
//! 本程序用于检测实验环境是否配置正确
//!
//! 运行方法：
//! ```
//! cargo run --bin ex1_env_setup
//! ```

use std::process::Command;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║            rCore 实验环境配置检测工具                       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // 检测 Rust 工具链
    println!("【1. Rust 工具链检测】");
    check_command("rustc", &["--version"]);
    check_command("cargo", &["--version"]);
    check_command("rustup", &["show"]);
    
    // 检测 RISC-V 目标
    println!("\n【2. RISC-V 编译目标检测】");
    check_command("rustup", &["target", "list", "--installed"]);
    
    // 检测 QEMU
    println!("\n【3. QEMU 模拟器检测】");
    check_command("qemu-system-riscv64", &["--version"]);
    
    // 检测 GDB
    println!("\n【4. 调试工具检测】");
    check_command("gdb", &["--version"]);
    check_command("riscv64-unknown-elf-gdb", &["--version"]);
    
    // 检测 Git
    println!("\n【5. 版本控制检测】");
    check_command("git", &["--version"]);

    // 检测 make
    println!("\n【6. 构建工具检测】");
    check_command("make", &["--version"]);

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    环境配置说明                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    print_env_setup_guide();
}

fn check_command(cmd: &str, args: &[&str]) {
    print!("  {} ... ", cmd);
    match Command::new(cmd).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout.lines().next().unwrap_or("OK");
                println!("✓ {}", first_line);
            } else {
                println!("✗ 执行失败");
            }
        }
        Err(_) => {
            println!("✗ 未安装");
        }
    }
}

fn print_env_setup_guide() {
    println!(r#"

=== rCore 实验环境配置指南 ===

1. 安装 Rust 工具链
   $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   $ source $HOME/.cargo/env

2. 添加 RISC-V 编译目标
   $ rustup target add riscv64gc-unknown-none-elf

3. 安装 QEMU (需要 >= 7.0 版本)
   Ubuntu/Debian:
   $ sudo apt-get install qemu-system-misc

   从源码编译:
   $ wget https://download.qemu.org/qemu-7.0.0.tar.xz
   $ tar xf qemu-7.0.0.tar.xz
   $ cd qemu-7.0.0
   $ ./configure --target-list=riscv64-softmmu,riscv64-linux-user
   $ make -j$(nproc)
   $ sudo make install

4. 安装 RISC-V 交叉编译工具链
   $ sudo apt-get install gcc-riscv64-unknown-elf

   或从预编译包安装:
   $ wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/...
   $ tar xf riscv64-unknown-elf-gcc-*.tar.gz
   $ export PATH=$PATH:/path/to/riscv64-unknown-elf-gcc-*/bin

5. 安装调试工具
   $ sudo apt-get install gdb-multiarch

6. 克隆 rCore-Tutorial 代码
   $ git clone https://github.com/rcore-os/rCore-Tutorial-v3.git
   $ cd rCore-Tutorial-v3

7. 编译运行
   $ cd os
   $ make run

=== 常见问题及解决方法 ===

Q1: QEMU 版本过低
A:  从源码编译安装最新版本 QEMU

Q2: 找不到 riscv64-unknown-elf-gcc
A:  安装交叉编译工具链，或将 bin 目录添加到 PATH

Q3: Rust 编译目标未安装
A:  运行 rustup target add riscv64gc-unknown-none-elf

Q4: make run 后 QEMU 立即退出
A:  检查 linker script 和 boot 流程是否正确

Q5: 如何退出 QEMU
A:  按 Ctrl+A，然后按 X

"#);
}

/*
=== 我的实验环境配置经历 ===

操作系统: WSL2 Ubuntu 22.04 / Linux

1. 安装 Rust
   - 问题：网络连接问题导致下载慢
   - 解决：配置 Rust 镜像源

2. 安装 QEMU 7.0
   - 问题：apt 源中版本过低（6.x）
   - 解决：从源码编译安装

3. 安装 RISC-V 工具链
   - 问题：预编译包下载慢
   - 解决：使用 apt 安装 gcc-riscv64-unknown-elf

4. 编译 rCore
   - 问题：首次编译时间长
   - 解决：使用 release 模式，配置好 cargo 镜像

5. 运行测试
   - 成功进入 rCore shell
   - 可以运行用户程序如 hello_world, gui_snake 等

=== 环境配置完成标志 ===

运行 `make run` 后看到以下输出表示环境配置成功：

    [RustSBI output]
    [KERN] init gpu
    [KERN] init keyboard
    [KERN] init mouse
    [KERN] init trap
    ...
    Rust user shell
    >

*/