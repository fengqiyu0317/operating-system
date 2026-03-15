# rCore-Tutorial 开发环境配置指南

> 基于 [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html) 编写

本文档详细介绍如何配置 rCore-Tutorial 的开发环境，包括 Rust 工具链、QEMU 模拟器和 RISC-V 交叉编译工具链的安装。

---

## 目录

1. [环境要求](#1-环境要求)
2. [安装 Rust 工具链](#2-安装-rust-工具链)
3. [安装 QEMU 模拟器](#3-安装-qemu-模拟器)
4. [安装 RISC-V 交叉编译工具链](#4-安装-risc-v-交叉编译工具链)
5. [克隆 rCore-Tutorial 代码](#5-克隆-rcore-tutorial-代码)
6. [验证环境配置](#6-验证环境配置)
7. [常见问题](#7-常见问题)

---

## 1. 环境要求

### 支持的操作系统

- **Linux (推荐)**: Ubuntu 18.04/20.04/22.04 或其他 Linux 发行版
- **macOS**: 10.15 (Catalina) 或更高版本
- **Windows**: 使用 WSL2 (Windows Subsystem for Linux 2)

### 硬件要求

- 至少 4GB 内存（推荐 8GB 以上）
- 至少 20GB 可用磁盘空间

---

## 2. 安装 Rust 工具链

### 2.1 安装 rustup

rustup 是 Rust 的官方安装器和管理工具。

**Linux / macOS / WSL2:**

```bash
# 下载并安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装过程中，选择默认安装（选项 1）即可。

**配置环境变量：**

```bash
# 加载 Rust 环境变量
source $HOME/.cargo/env
```

或者重新打开终端。

### 2.2 配置 Rust 工具链

```bash
# 确保 Rust 版本足够新（需要 1.66.0 或更高版本）
rustc --version

# 如果版本过低，更新 Rust 工具链
rustup update
```

### 2.3 添加 RISC-V 目标平台

rCore-Tutorial 需要 RISC-V 64 位目标平台支持：

```bash
# 添加 riscv64 目标
rustup target add riscv64gc-unknown-none-elf
```

### 2.4 安装必要的 Rust 组件

```bash
# 安装 rust-src 组件（用于编译裸机程序）
rustup component add rust-src

# 安装 llvm-tools-preview 组件（用于生成二进制文件）
rustup component add llvm-tools-preview

# （可选）安装 rust-analyzer 分析器
rustup component add rust-analyzer
```

### 2.5 安装 cargo-binutils

```bash
# 安装 cargo-binutils，提供 cargo objdump 等工具
cargo install cargo-binutils
```

---

## 3. 安装 QEMU 模拟器

QEMU 是一个通用的开源机器模拟器和虚拟化器，用于运行 RISC-V 操作系统。

### 3.1 Ubuntu/Debian 系统安装（推荐）

**方法一：使用包管理器安装（简单）**

```bash
# Ubuntu 22.04 或更高版本
sudo apt-get update
sudo apt-get install qemu-system-misc

# Ubuntu 20.04 或更低版本
# 注意：这些版本的 QEMU 可能太旧，建议使用方法二编译安装
```

**方法二：从源码编译安装（推荐用于获得最新版本）**

如果系统自带的 QEMU 版本过低（低于 7.0），建议从源码编译：

```bash
# 安装编译依赖
sudo apt-get install -y gcc libc6-dev pkg-config \
    libglib2.0-dev libpixman-1-dev ninja-build

# 下载 QEMU 7.0.0 源码（如果已有压缩包可跳过）
wget https://download.qemu.org/qemu-7.0.0.tar.xz

# 解压源码
tar -xf qemu-7.0.0.tar.xz
cd qemu-7.0.0

# 配置编译选项（只编译 RISC-V 支持，加快编译速度）
./configure --target-list=riscv64-softmmu,riscv64-linux-user

# 编译（使用多核加速，-j 后的数字根据 CPU 核心数调整）
make -j$(nproc)

# 安装到系统目录
sudo make install
```

### 3.2 macOS 系统安装

```bash
# 使用 Homebrew 安装
brew install qemu
```

### 3.3 验证 QEMU 安装

```bash
# 检查 QEMU 版本
qemu-system-riscv64 --version

# 预期输出类似：
# QEMU emulator version 7.0.0
```

---

## 4. 安装 RISC-V 交叉编译工具链

RISC-V 交叉编译工具链用于编译和链接 RISC-V 汇编代码。

### 4.1 Ubuntu/Debian 系统安装

**方法一：使用包管理器安装（Ubuntu 22.04+）**

```bash
sudo apt-get install gcc-riscv64-unknown-elf
```

**方法二：使用预编译包安装（推荐）**

如果系统包管理器没有提供该工具链，可以下载预编译版本：

```bash
# 创建安装目录
sudo mkdir -p /opt/riscv

# 下载预编译工具链
wget https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz

# 解压到 /opt/riscv
sudo tar -xzf riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz -C /opt/riscv

# 添加到 PATH 环境变量
echo 'export PATH="/opt/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 4.2 macOS 系统安装

```bash
# 使用 Homebrew 安装
brew tap riscv/riscv
brew install riscv-gnu-toolchain --with-multilib
```

### 4.3 验证工具链安装

```bash
# 检查工具链版本
riscv64-unknown-elf-gcc --version

# 预期输出类似：
# riscv64-unknown-elf-gcc (GCC) 8.3.0
```

---

## 5. 克隆 rCore-Tutorial 代码

### 5.1 克隆代码仓库

```bash
# 克隆 rCore-Tutorial-v3 主仓库
git clone https://github.com/rcore-os/rCore-Tutorial-v3.git

# 进入项目目录
cd rCore-Tutorial-v3
```

### 5.2 切换到合适的分支

```bash
# 查看所有分支
git branch -a

# 切换到 main 分支（或根据课程要求切换到特定分支）
git checkout main
```

---

## 6. 验证环境配置

### 6.1 编译测试

```bash
# 在 rCore-Tutorial-v3 目录下执行
make build

# 或者使用 cargo 直接编译
cargo build --target riscv64gc-unknown-none-elf --release
```

### 6.2 运行测试

```bash
# 在 QEMU 中运行 rCore
make run
```

如果一切正常，你将看到 rCore 操作系统启动并进入 shell 界面。

### 6.3 完整验证脚本

以下脚本可以一次性验证所有工具是否正确安装：

```bash
#!/bin/bash

echo "=== 验证 rCore-Tutorial 开发环境 ==="
echo ""

# 检查 Rust
echo "1. 检查 Rust 工具链..."
if command -v rustc &> /dev/null; then
    echo "   ✓ Rust 版本: $(rustc --version)"
else
    echo "   ✗ Rust 未安装"
fi

# 检查 RISC-V 目标
echo ""
echo "2. 检查 RISC-V 目标..."
if rustup target list --installed | grep -q "riscv64gc-unknown-none-elf"; then
    echo "   ✓ riscv64gc-unknown-none-elf 目标已安装"
else
    echo "   ✗ riscv64gc-unknown-none-elf 目标未安装"
fi

# 检查 rust-src
echo ""
echo "3. 检查 rust-src 组件..."
if rustup component list --installed | grep -q "rust-src"; then
    echo "   ✓ rust-src 组件已安装"
else
    echo "   ✗ rust-src 组件未安装"
fi

# 检查 QEMU
echo ""
echo "4. 检查 QEMU..."
if command -v qemu-system-riscv64 &> /dev/null; then
    echo "   ✓ QEMU 版本: $(qemu-system-riscv64 --version | head -1)"
else
    echo "   ✗ qemu-system-riscv64 未安装"
fi

# 检查 RISC-V 工具链
echo ""
echo "5. 检查 RISC-V 工具链..."
if command -v riscv64-unknown-elf-gcc &> /dev/null; then
    echo "   ✓ riscv64-unknown-elf-gcc 版本: $(riscv64-unknown-elf-gcc --version | head -1)"
else
    echo "   ✗ riscv64-unknown-elf-gcc 未安装"
fi

echo ""
echo "=== 验证完成 ==="
```

---

## 7. 常见问题

### Q1: `rustup` 命令找不到

**解决方案：** 确保已正确安装 rustup 并加载环境变量：

```bash
source $HOME/.cargo/env
```

或重新打开终端。

### Q2: QEMU 版本过低

**解决方案：** 从源码编译安装 QEMU 7.0 或更高版本，参见 [3.1 节](#31-ubuntudebian-系统安装推荐)。

### Q3: 编译时出现链接错误

**解决方案：** 确保已安装 `rust-src` 组件：

```bash
rustup component add rust-src
```

### Q4: 运行 `make run` 时 QEMU 无响应

**解决方案：** 
1. 检查 QEMU 是否正确安装
2. 尝试使用 `-nographic` 选项手动运行：
```bash
qemu-system-riscv64 -machine virt -nographic -bios none -kernel target/riscv64gc-unknown-none-elf/release/os
```

### Q5: Windows WSL2 中无法运行 QEMU

**解决方案：** 
1. 确保 WSL2 版本足够新
2. 可能需要安装额外的库：
```bash
sudo apt-get install libgtk-3-dev
```

---

## 附录：一键安装脚本

以下是一键安装所有依赖的脚本（适用于 Ubuntu/Debian）：

```bash
#!/bin/bash
set -e

echo "开始配置 rCore-Tutorial 开发环境..."

# 1. 安装系统依赖
echo "[1/5] 安装系统依赖..."
sudo apt-get update
sudo apt-get install -y curl gcc libc6-dev pkg-config \
    libglib2.0-dev libpixman-1-dev ninja-build git

# 2. 安装 Rust
echo "[2/5] 安装 Rust 工具链..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# 3. 配置 Rust 组件
echo "[3/5] 配置 Rust 组件..."
rustup update
rustup target add riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
cargo install cargo-binutils

# 4. 安装 QEMU
echo "[4/5] 安装 QEMU..."
if ! command -v qemu-system-riscv64 &> /dev/null; then
    sudo apt-get install -y qemu-system-misc
fi

# 5. 安装 RISC-V 工具链
echo "[5/5] 安装 RISC-V 工具链..."
if ! command -v riscv64-unknown-elf-gcc &> /dev/null; then
    sudo apt-get install -y gcc-riscv64-unknown-elf || {
        echo "包管理器安装失败，尝试手动安装..."
        sudo mkdir -p /opt/riscv
        wget -q https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz
        sudo tar -xzf riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz -C /opt/riscv
        echo 'export PATH="/opt/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin:$PATH"' >> ~/.bashrc
    }
fi

echo "安装完成！请运行以下命令加载环境变量："
echo "  source ~/.bashrc"
echo ""
echo "然后运行验证脚本检查安装是否成功。"
```

---

## 参考链接

- [rCore-Tutorial-Book-v3 官方文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [Rust 官方网站](https://www.rust-lang.org/)
- [QEMU 官方网站](https://www.qemu.org/)
- [RISC-V GNU Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)
- [rCore-Tutorial-v3 GitHub](https://github.com/rcore-os/rCore-Tutorial-v3)