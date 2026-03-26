# rCore-Tutorial Chapter 3 练习记录

## 任务：编程题 #4 - 统计任务切换开销

### 目标
实现任务切换开销统计功能，测量 `__switch` 函数执行所需的 CPU cycles。

### 方案概述
1. 在 `TaskContext` 中添加 `switch_temp` 字段用于存储时间戳
2. 在 `switch.S` 的 `__switch` 函数中测量切换前后的 CPU cycles
3. 使用全局变量累计总的切换开销和切换次数
4. 通过系统调用 `SYSCALL_GET_SWITCH_INFO` 向用户态暴露统计信息

### 已完成的修改

#### 1. `os/src/task/context.rs`
- 添加 `switch_temp: usize` 字段到 `TaskContext`
- 用于在 `__switch` 中临时存储开始时间

#### 2. `os/src/task/switch.S`
- 在 `__switch` 开头使用 `rdtime t2` 记录开始时间，存到 `switch_temp`
- 在切换完成后读取结束时间，计算开销，累加到全局变量
- 使用 `t5` 寄存器传递全局变量的地址

#### 3. `os/src/task/mod.rs`
- 添加全局变量：
  - `TOTAL_SWITCH_OVERHEAD: UPSafeCell<usize>` - 总切换开销
  - `TOTAL_SWITCH_COUNT: UPSafeCell<usize>` - 总切换次数
  - `SHOW_TASK_SWITCH: UPSafeCell<bool>` - 是否显示切换信息
- 修改 `run_first_task`, `run_next_task`, `exit_and_run_next` 以支持统计

#### 4. `os/src/config.rs`
- 将 `MAX_APP_NUM` 从 5 改为 16，避免索引越界

#### 5. `os/src/syscall/process.rs` 和 `os/src/syscall/mod.rs`
- 实现 `sys_get_switch_info()` 系统调用
- 返回格式：高32位为总切换次数，低32位为总切换开销

#### 6. `user/src/syscall.rs`
- 添加 `sys_get_switch_info()` 包装函数
- 解包返回的两个值

#### 7. `user/src/bin/05switch_cost.rs`
- 创建测试程序
- 执行 100 次 `yield_()` 调用
- 计算并显示平均每次切换的开销

### 当前问题：BorrowMutError

**错误信息**：
```
[ERROR] [kernel] Panicked at src/sync/up.rs:29 already borrowed: BorrowMutError
```

**问题位置**：
`os/src/task/mod.rs` 中的 `run_first_task`、`run_next_task`、`exit_and_run_next`

**问题代码**：
```rust
// 增加总切换次数
*TOTAL_SWITCH_COUNT.exclusive_access() += 1;
// 获取总切换开销变量的地址
let overhead_ptr = &mut *TOTAL_SWITCH_OVERHEAD.exclusive_access() as *mut usize as usize;
```

**根本原因**：
`UPSafeCell` 内部使用 `RefCell`，不允许同时持有多个可变借用，即使是不同的实例。

**可能的解决方案**：
1. 将 `TOTAL_SWITCH_COUNT` 和 `TOTAL_SWITCH_OVERHEAD` 合并为一个结构体
2. 使用 `static mut` 和 `unsafe` 绕过借用检查
3. 预先计算地址并存储在 `static mut` 变量中

### 待完成任务
1. 修复 BorrowMutError 问题
2. 运行测试程序验证功能
3. 确认测量结果准确
