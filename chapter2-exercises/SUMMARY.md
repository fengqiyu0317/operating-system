# Chapter 2 练习题1 - 调用栈打印程序总结

## 完成情况

✅ 已完成：实现一个裸机应用程序，能打印调用栈

## 题目难度

*** (3星)

## 实现内容

### 1. 核心文件

- **源代码**: [`user/src/bin/06stack_trace.rs`](user/src/bin/06stack_trace.rs)
- **详细文档**: [`README_stack_trace.md`](README_stack_trace.md)

### 2. 关键技术点

#### RISC-V 栈帧结构
```
高地址
    +------------------+
    |   局部变量        |
    +------------------+
    |   保存的寄存器    |
    +------------------+
    |   保存的 RA       |  <- fp - 8 (返回地址)
    +------------------+
    |   保存的 FP       |  <- fp (上一个帧指针)
    +------------------+
    |   参数区域        |
    +------------------+
低地址
```

#### 实现方法
1. 使用内联汇编读取当前帧指针 fp (x8 寄存器)
2. 从栈帧中读取返回地址 (位于 fp - 8)
3. 读取上一个帧指针 (位于 fp)
4. 沿着 fp 链遍历直到 fp 为 0
5. 设置最大深度限制防止无限循环

### 3. 核心代码结构

```rust
// 栈帧结构体
struct StackFrame {
    fp: FramePointer,  // 上一个栈帧指针
    ra: usize,         // 返回地址
}

// 获取当前帧指针
fn current_fp() -> FramePointer {
    unsafe {
        let fp: FramePointer;
        core::arch::asm!("mv {}, fp", out(reg) fp);
        fp
    }
}

// 打印调用栈
fn print_stack_trace() {
    let mut current_fp = current_fp();
    loop {
        if current_fp == 0 { break; }
        let frame = StackFrame::from_fp(current_fp)?;
        println!("返回地址: {:#x}", frame.return_address());
        current_fp = frame.caller_fp();
    }
}
```

## 运行测试

### 编译
```bash
cd chapter2-exercises/user
make build
```

### 运行
```bash
cd ../os
make run
```

### 测试结果

程序成功运行，输出包括：
- ✅ 多层函数调用 (level_1 → level_2 → level_3)
- ✅ 递归深度追踪 (5 → 4 → 3 → 2 → 1)
- ✅ 调用栈信息打印（栈帧地址、返回地址、上一个帧）
- ✅ 栈帧计数
- ✅ 正常退出

## 技术亮点

1. **裸机环境实现**: 无需标准库支持，直接操作硬件
2. **内联汇编**: 使用 Rust 的 `core::arch::asm!` 读取寄存器
3. **内存安全**: 使用 unsafe 块封装原始指针操作
4. **递归测试**: 通过多层递归验证调用栈完整性
5. **错误保护**: 设置最大深度限制防止栈损坏

## 应用场景

- **调试**: 快速定位程序崩溃位置
- **性能分析**: 了解函数调用链
- **学习**: 理解 RISC-V 调用约定和栈帧布局
- **开发工具**: 为操作系统提供调试支持

## 相关文件

- 源代码: `chapter2-exercises/user/src/bin/06stack_trace.rs`
- 详细文档: `chapter2-exercises/README_stack_trace.md`
- 链接脚本: `chapter2-exercises/os/src/link_app.S` (已更新)

## 总结

本练习成功实现了在裸机环境下打印调用栈的功能，深入理解了：
- RISC-V 架构的栈帧结构
- 函数调用约定
- 寄存器使用规范
- 裸机编程技巧

这是一个重要的调试工具，为后续的操作系统开发和调试奠定了基础。
