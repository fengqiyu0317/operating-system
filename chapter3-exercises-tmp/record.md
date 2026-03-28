# rCore-Tutorial Chapter 3 练习记录

## 任务：编程题 #4 - 统计任务切换开销

### 目标
实现任务切换开销统计功能，测量 `__switch` 函数执行所需的 CPU cycles。

---

## 第二次尝试：不修改 TaskContext 的方案（2026-03-26）

### 设计思路

**关键教训**：修改 TaskContext 大小会破坏汇编代码中的偏移量，因此采用新方案：

1. **不修改 TaskContext 结构体**（避免破坏汇编偏移量）
2. **使用全局静态变量存储开始时间**
3. **在 Rust 代码中包裹 __switch 调用**

### 实现方案

#### 1. 新增 `os/src/task/switch_cost.rs`

```rust
/// Task switching cost statistics
pub struct SwitchCostStats {
    pub total_switches: usize,  // 总切换次数
    pub total_cost: usize,      // 总开销（cycles）
    pub max_cost: usize,        // 最大单次开销
    pub min_cost: usize,        // 最小单次开销
}

// 全局开始时间（静态变量，切换前后共享）
static mut SWITCH_START_TIME: usize = 0;

// 统计数据
lazy_static! {
    pub static ref SWITCH_COST_STATS: UPSafeCell<SwitchCostStats> = ...
}

// 在 __switch 前调用
pub fn mark_switch_start() {
    unsafe { SWITCH_START_TIME = get_time(); }
}

// 在 __switch 返回后调用
pub fn record_switch_cost() -> usize {
    let cost = get_time() - unsafe { SWITCH_START_TIME };
    SWITCH_COST_STATS.exclusive_access().record(cost);
    cost
}
```

#### 2. 修改 `os/src/task/mod.rs` 的 `run_next_task()`

```rust
fn run_next_task(&self) {
    if let Some(next) = self.find_next_task() {
        // ... 获取任务指针 ...

        mark_switch_start();           // 记录开始时间
        unsafe {
            __switch(current_task_cx_ptr, next_task_cx_ptr);
        }
        let _cost = record_switch_cost();  // 记录切换开销

    } else {
        println!("All applications completed!");
        print_switch_stats();  // 输出统计信息
        shutdown(false);
    }
}
```

#### 3. 测试程序 `user/src/bin/04switch_cost.rs`

```rust
fn main() -> i32 {
    println!("=== Task Switch Cost Test ===");
    for i in 0..100 {
        if i % 10 == 0 {
            println!("Yield iteration: {}", i);
        }
        yield_();
    }
    0
}
```

### 测试结果

```
[kernel] Task Switch Statistics:
  Total switches: 3337525
  Total cost: 2953201 cycles
  Average cost: 0 cycles
  Max cost: 2931 cycles
  Min cost: 1 cycles
```

### 结果分析

#### 问题1：切换次数异常多

**预期**：约 100 次 yield + 一些任务间切换 ≈ 200-300 次
**实际**：3337525 次

**原因分析**：
- Timer 中断（每 10ms）也会触发 `suspend_current_and_run_next()`
- 每个 timer 中断都会被记录为一次"切换"

#### 问题2：平均开销为 0

- 整数除法：`2953201 / 3337525 = 0`
- 大部分"切换"开销极小（1 cycle）

#### 关键发现

**测量的是什么**？

```
任务 A 调用 run_next_task():
  mark_switch_start()        // SWITCH_START_TIME = T1
  __switch(A, B)

任务 B 完成后调用 run_next_task():
  mark_switch_start()        // SWITCH_START_TIME = T2 (覆盖!)
  __switch(B, A)

回到任务 A:
  record_switch_cost()       // get_time() - T2 (不是 T1!)
```

- 测量的是 `T_now - T2`，即**最近一次切换的开销**
- 不包含其他任务的执行时间（因为 `SWITCH_START_TIME` 在下次切换时被覆盖）

### 当前实现的问题

1. **包含了 timer 中断触发的切换** - 这不是我们想测量的
2. **测量精度问题** - 包含了 Rust 函数调用开销
3. **无法区分切换类型** - yield、exit、timer 中断都混在一起

### 待改进方向

**方案 A**：区分切换类型，只统计 yield/exit 触发的切换

**方案 B**：在汇编代码 `switch.S` 中直接测量，排除其他开销

```assembly
__switch:
    rdtime t6          # 入口时间
    # ... 保存/恢复寄存器 ...
    rdtime t5          # 出口时间
    sub t5, t5, t6     # 计算差值
    sd t5, SWITCH_COST  # 存储
    ret
```

### 修改的文件清单

| 文件 | 操作 |
|------|------|
| `os/src/task/switch_cost.rs` | 新增：切换统计模块 |
| `os/src/task/mod.rs` | 修改：集成时间记录 |
| `user/src/bin/04switch_cost.rs` | 新增：测试程序 |

### 编译和运行

```bash
make run
```

### 下一步

需要决定采用哪种改进方案来获得更准确的测量结果。
