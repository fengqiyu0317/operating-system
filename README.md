# Operating System Course Materials

操作系统课程学习资料

## 课程信息

- 课程名称：操作系统
- 学期：大二春季学期

## 目录结构

```
.
├── docs/                           # 文档目录
│   ├── rCore开发环境配置指南.md     # rCore-Tutorial 开发环境配置详细步骤
│   └── lab1-tutorial.md            # Lab1 实验教程
├── os/                             # 内核实现目录
│   ├── Cargo.toml                  # 项目配置文件
│   ├── Makefile                    # 构建脚本
│   └── src/                        # 源代码目录
│       ├── console.rs              # 控制台输出（彩色日志宏）
│       ├── entry.asm               # 汇编入口
│       ├── lang_items.rs           # 语言项处理
│       ├── linker.ld               # 链接脚本
│       ├── main.rs                 # 主函数
│       └── sbi.rs                  # SBI 调用封装
├── reports/                        # 实验报告目录
│   └── lab1.md                     # Lab1 实验报告
├── bootloader/                     # RustSBI 固件
├── qemu-7.0.0/                     # QEMU 源码目录
├── qemu-7.0.0.tar.xz               # QEMU 源码压缩包
├── riscv64-unknown-elf-gcc-.../    # RISC-V 工具链目录
└── riscv64-unknown-elf-gcc-...tar.gz # RISC-V 工具链压缩包
```

## 快速开始

### 开发环境配置

请参阅 [rCore开发环境配置指南](./docs/rCore开发环境配置指南.md)，该指南包含：

1. **环境要求** - 支持的操作系统和硬件要求
2. **Rust 工具链安装** - rustup、RISC-V 目标、必要组件
3. **QEMU 模拟器安装** - 包管理器和源码编译两种方式
4. **RISC-V 交叉编译工具链** - gcc-riscv64-unknown-elf 安装
5. **rCore-Tutorial 代码克隆** - 获取课程代码
6. **环境验证** - 编译和运行测试
7. **常见问题** - 问题排查和解决方案

### 一键安装（Ubuntu/Debian）

详细的一键安装脚本请参考文档中的附录部分。

## 实验运行

### Lab1: 应用程序与基本执行环境

**分支**: `ch1`

**功能**:
- 裸机 Hello World 输出
- ANSI 彩色日志输出（error, warn, info, debug）
- 内存布局展示（.text, .rodata, .data, .bss）

**运行步骤**:

```bash
# 切换到 Lab1 分支
cd os
git checkout ch1

# 编译
make build

# 运行
make run
```

**预期输出**:

```
Hello, world!
[INFO] .text   [0x80200000, 0x80202000)
[DEBUG] .rodata [0x80202000, 0x80203000)
[WARN] .data   [0x80203000, 0x80203000)
[ERROR] .bss    [0x80213000, 0x80213000)
```

**实验报告**: 详见 [reports/lab1.md](./reports/lab1.md)

## 参考资料

- [rCore-Tutorial-Book-v3 官方文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [rCore-Tutorial-v3 GitHub](https://github.com/rcore-os/rCore-Tutorial-v3)
- [Rust 官方网站](https://www.rust-lang.org/)
- [QEMU 官方网站](https://www.qemu.org/)
