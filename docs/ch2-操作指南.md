# rCore-Tutorial-v3 Chapter 2 批处理系统运行指南

## 概述

本章实现了一个支持批处理的"邓式鱼"操作系统，能够连续运行多个应用程序，并在程序出错时保护系统自身。

## 环境要求

- QEMU 模拟器
- Git
- Rust 工具链
- make 工具

## 操作步骤

### 1. 获取代码

```bash
# 克隆仓库
git clone https://github.com/rcore-os/rCore-Tutorial-v3.git

# 进入项目目录
cd rCore-Tutorial-v3

# 切换到第二章分支
git checkout ch2
```

### 2. 运行批处理系统

```bash
# 在 QEMU 模拟器上运行
make run
```

### 3. 预期输出结果

如果一切顺利，您将看到批处理系统自动加载并运行所有程序：

```
[RustSBI output]
[kernel] Hello, world!
[kernel] num_app = 5
[kernel] app_0 [0x8020a038, 0x8020af90)
[kernel] app_1 [0x8020af90, 0x8020bf80)
[kernel] app_2 [0x8020bf80, 0x8020d108)
[kernel] app_3 [0x8020d108, 0x8020e0e0)
[kernel] app_4 [0x8020e0e0, 0x8020f0b8)
[kernel] Loading app_0
Hello, world!
[kernel] Application exited with code 0
[kernel] Loading app_1
Into Test store_fault, we will insert an invalid store operation...
Kernel should kill this application!
[kernel] PageFault in application, kernel killed it.
[kernel] Loading app_2
3^10000=5079(MOD 10007)
3^20000=8202(MOD 10007)
3^30000=8824(MOD 10007)
3^40000=5750(MOD 10007)
3^50000=3824(MOD 10007)
3^60000=8516(MOD 10007)
3^70000=2510(MOD 10007)
3^80000=9379(MOD 10007)
3^90000=2621(MOD 10007)
3^100000=2749(MOD 10007)
Test power OK!
[kernel] Application exited with code 0
[kernel] Loading app_3
Try to execute privileged instruction in U Mode
Kernel should kill this application!
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Loading app_4
Try to access privileged CSR in U Mode
Kernel should kill this application!
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Panicked at src/batch.rs:58 All applications completed!
```

## 测试程序说明

本章包含 5 个测试程序，位于 `user/src/bin` 目录下：

| 程序 | 功能 | 预期结果 |
|------|------|----------|
| 00hello_world.rs | 输出 "Hello, world!" | 正常退出(code 0) |
| 01store_fault.rs | 故意执行非法内存写入 | 触发 PageFault，被内核终止 |
| 02power.rs | 计算 3 的幂次模 10007 | 正常退出(code 0) |
| 03priv_inst.rs | 尝试在用户态执行特权指令 | 触发 IllegalInstruction，被内核终止 |
| 04priv_csr.rs | 尝试在用户态访问特权 CSR | 触发 IllegalInstruction，被内核终止 |

## 核心功能演示

- ✅ 批处理系统自动加载并运行多个应用程序
- ✅ 特权级保护机制正常工作
- ✅ 应用程序错误不会影响操作系统稳定性
- ✅ 跨特权级的系统调用功能正常

## 注意事项

1. 确保已安装完整的 RISC-V 开发环境
2. 首次运行可能需要下载依赖，请耐心等待
3. 如遇编译错误，请检查 Rust 工具链版本是否正确

## 参考资料

- [rCore-Tutorial-Book-v3 第二章](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/0intro.html)
