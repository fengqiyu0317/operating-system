# 练习题1：实现裸机应用程序打印调用栈

## 题目要求

实现一个裸机应用程序A，能打印调用栈。

难度等级：*** (3星)

## 实现原理

### RISC-V 栈帧结构

在 RISC-V 架构中，函数调用时会创建栈帧（stack frame）。栈帧的结构遵循 RISC-V 调用约定：

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

关键寄存器：
- **fp (x8)**: 帧指针（Frame Pointer），指向当前栈帧
- **ra (x1)**: 返回地址（Return Address）

### 调用栈遍历原理

遍历调用栈的基本思路：

1. **获取当前帧指针**：通过内联汇编读取 fp 寄存器的值
2. **读取栈帧信息**：
   - 在 `fp` 位置读取上一个栈帧的 fp 值
   - 在 `fp - 8` 位置读取返回地址（ra）
3. **遍历链表**：沿着 fp 链向上遍历，直到 fp 为 0
4. **安全保护**：设置最大深度限制，防止栈损坏导致的无限循环

### 代码实现要点

#### 1. 获取当前帧指针

使用 Rust 内联汇编获取 fp 寄存器：

```rust
unsafe fn current_fp() -> FramePointer {
    let mut fp: FramePointer = 0;
    core::arch::asm!(
        "mv {}, fp",
        out(reg) fp
    );
    fp
}
```

#### 2. 从栈帧读取信息

```rust
unsafe fn from_fp(fp: FramePointer) -> Option<Self> {
    if fp == 0 {
        return None;
    }

    // 读取保存的 fp 和 ra
    let saved_fp = *(fp as *const usize);
    let saved_ra = *((fp - 8) as *const usize);

    Some(StackFrame { fp: saved_fp, ra: saved_ra })
}
```

#### 3. 遍历调用栈

```rust
loop {
    if current_fp == 0 {
        break;
    }

    let frame = StackFrame::from_fp(current_fp)?;
    // 打印帧信息
    println!("返回地址: {:#x}", frame.return_address());

    // 移动到上一个栈帧
    current_fp = frame.caller_fp();
}
```

## 文件说明

### 源代码文件

- **`user/src/bin/06stack_trace.rs`**: 调用栈打印程序
  - 实现了 `StackFrame` 结构体来表示栈帧
  - 提供了 `print_stack_trace()` 函数来打印完整调用栈
  - 包含多层递归函数调用用于测试

### 主要组件

#### StackFrame 结构体

```rust
struct StackFrame {
    fp: FramePointer,  // 上一个栈帧的指针
    ra: usize,         // 返回地址
}
```

方法：
- `from_fp()`: 从帧指针创建栈帧对象
- `return_address()`: 获取返回地址
- `caller_fp()`: 获取调用者的帧指针

#### 核心函数

- `current_fp()`: 获取当前帧指针
- `print_stack_trace()`: 打印完整调用栈
- `recursive_function()`: 递归函数，用于创建多层调用栈

## 运行和测试

### 编译程序

在 `chapter2-exercises` 目录下执行：

```bash
cd user
make build
```

### 运行程序

```bash
cd ..
make run BIN=06stack_trace
```

### 预期输出

程序应该输出类似以下内容：

```
====================================
RISC-V 调用栈打印程序
====================================

本程序演示如何打印调用栈信息
将创建多层函数调用，然后打印完整的调用栈

进入 level_1
进入 level_2
进入 level_3
递归深度: 5
递归深度: 4
递归深度: 3
递归深度: 2
递归深度: 1
到达递归底部，开始打印调用栈...

=== 调用栈追踪 ===

帧指针: 0xxxxxx

[0] 栈帧地址: 0xxxxxx
    返回地址: 0xxxxxx
    上一个帧: 0xxxxxx

[1] 栈帧地址: 0xxxxxx
    返回地址: 0xxxxxx
    上一个帧: 0xxxxxx

...

总共 XX 个栈帧

====================================
程序执行完成
====================================
```

## 技术要点

### 1. 裸机环境的特点

- **无标准库支持**：使用 `#![no_std]`，不能使用标准库的栈展开功能
- **直接访问硬件**：需要通过内联汇编直接读取寄存器
- **手动内存管理**：需要通过原始指针访问栈内存

### 2. RISC-V 调用约定

- 函数调用时，调用者需要保存 ra 和 fp
- 被调用函数在序言（prologue）中保存 fp 和 ra
- 返回地址 ra 存储在 `fp - 8` 位置
- 上一个帧指针存储在 `fp` 位置

### 3. 安全考虑

- **空指针检查**：检查 fp 是否为 0
- **深度限制**：防止栈损坏导致的无限循环
- **边界检查**：确保读取的内存地址有效

## 扩展思考

### 1. 符号解析

当前程序只能打印返回地址的数字值。在实际应用中，可以：
- 建立地址到函数名的映射表
- 实现符号解析功能
- 显示函数名而非地址

### 2. 调试信息增强

可以添加更多信息：
- 局部变量的值
- 参数信息
- 源代码行号（如果有调试信息）

### 3. 错误检测

可以检测：
- 栈溢出
- 栈损坏
- 异常的调用栈模式

## 相关概念

### 栈帧（Stack Frame）

栈帧是函数调用时在栈上分配的内存区域，用于：
- 保存返回地址
- 保存被调用者需要使用的寄存器
- 存储局部变量
- 传递参数

### 调用约定（Calling Convention）

RISC-V 调用约定定义了：
- 参数传递规则
- 返回值传递规则
- 寄存器保存规则
- 栈帧组织方式

### 链接（Linkage）

链接器将多个目标文件组合成可执行文件，解析符号引用。

## 参考资料

- [RISC-V 调用约定](https://github.com/riscv-non-isa/riscv-elf-psabi-doc)
- [rCore-Tutorial-Book-v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/)
- [RISC-V 手册](https://riscv.org/technical/specifications/)

## 总结

本练习实现了在裸机环境下打印调用栈的功能，通过：
1. 理解 RISC-V 栈帧结构
2. 使用内联汇编读取寄存器
3. 手动遍历栈帧链表
4. 安全地访问栈内存

这个程序对于调试和理解程序执行流程非常有用，是操作系统开发中的重要工具。
