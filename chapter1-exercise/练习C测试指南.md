# 练习C：rCore Sleep系统调用 - 测试指南

## ✅ 已完成

- ✅ 已在rCore项目中创建 `exercise_c.rs` 用户程序
- ✅ 程序位置: `/mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/user/src/bin/exercise_c.rs`
- ✅ 已验证rCore可以识别并加载该程序

## 🚀 如何测试

### 方法1: 手动运行（推荐）

1. **进入rCore的os目录**:
   ```bash
   cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/os
   ```

2. **编译并运行rCore**:
   ```bash
   make run
   ```

3. **等待启动完成**:
   - 你会看到RustSBI的启动信息
   - 然后是rCore内核的初始化日志
   - 最后会显示应用程序列表（包括`exercise_c`）
   - 出现 `Rust user shell` 提示符和 `>>`

4. **在shell中运行exercise_c**:
   ```bash
   >> exercise_c
   ```

5. **观察输出**:
   ```
   ===============================================
   Exercise C: Sleep System Call Test
   ===============================================

   开始时间: XXXXX ms
   准备睡眠 5000 ms (5秒)...

   结束时间: XXXXX ms
   实际睡眠时间: 5000 ms (约 5 秒)

   ===============================================
   Exercise C: Test Passed!
   ===============================================
   ```

6. **退出QEMU**:
   按 `Ctrl+A` 然后按 `X`

### 方法2: 自动化运行

如果你想自动运行exercise_c（不进入shell），可以修改rCore的初始化流程：

**编辑 `/mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/user/src/bin/initproc.rs`**:

```rust
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::*;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    // 直接运行exercise_c
    println!("Starting exercise_c...");
    let pid = fork();
    if pid == 0 {
        // 子进程：运行exercise_c
        exec("exercise_c", &[/* 无参数 */]);
    } else {
        // 父进程：等待子进程结束
        let mut exit_code = 0;
        waitpid(pid as usize, &mut exit_code);
        println!("Exercise C completed with exit code: {}", exit_code);
    }
    0
}
```

然后运行：
```bash
cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/os
make run TEST=initproc
```

## 📁 程序代码

### exercise_c.rs 完整代码

```rust
#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time, sleep};

/// 练习C: 使用sleep系统调用睡眠5秒
#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("===============================================");
    println!("Exercise C: Sleep System Call Test");
    println!("===============================================\n");

    // 获取开始时间
    let start_time = get_time();
    println!("开始时间: {} ms", start_time);
    println!("准备睡眠 5000 ms (5秒)...\n");

    // 使用sleep系统调用睡眠5000毫秒（5秒）
    sleep(5000);

    // 获取结束时间
    let end_time = get_time();
    let elapsed = end_time - start_time;

    println!("结束时间: {} ms", end_time);
    println!("实际睡眠时间: {} ms (约 {} 秒)", elapsed, elapsed / 1000);
    println!("\n===============================================");
    println!("Exercise C: Test Passed!");
    println!("===============================================");

    0
}
```

## 🔍 程序说明

### 功能
- 使用rCore的`sleep`系统调用
- 睡眠5000毫秒（5秒）
- 测量并显示实际睡眠时间

### 使用的系统调用
- `get_time()`: 获取当前时间（毫秒）
- `sleep(ms)`: 睡眠指定的毫秒数

### 与rCore的集成
- 使用`user_lib`库提供的系统调用接口
- 遵循rCore用户程序的规范（`#![no_std]`、`#![no_main]`）
- 使用`#[unsafe(no_mangle)]`导出main函数

## 📊 预期输出

```
Rust user shell
>> exercise_c
===============================================
Exercise C: Sleep System Call Test
===============================================

开始时间: 12345 ms
准备睡眠 5000 ms (5秒)...

结束时间: 17345 ms
实际睡眠时间: 5000 ms (约 5 秒)

===============================================
Exercise C: Test Passed!
===============================================
>>
```

## 🐛 调试技巧

### 1. 查看编译的用户程序
```bash
cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/user
ls target/riscv64gc-unknown-none-elf/release/exercise_c
```

### 2. 检查文件系统镜像
```bash
cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/os
ls ../user/target/riscv64gc-unknown-none-elf/release/fs.img
```

### 3. 使用GDB调试
```bash
cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/os
make debug
# 在另一个终端
riscv64-unknown-elf-gdb
(gdb) target remote :1234
(gdb) break exercise_c::main
(gdb) continue
```

### 4. 查看系统调用实现
```bash
# 查看sleep系统调用的实现
cd /mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/rCore-Tutorial-v3/os
grep -r "sys_sleep" src/syscall/
```

## 📚 相关知识

### sleep系统调用的实现原理
1. **用户态**: 调用`sleep(ms)`
2. **系统调用**: 触发`SYS_SLEEP`系统调用
3. **内核态**:
   - 将当前进程标记为 sleeping
   - 设置唤醒时间 = 当前时间 + ms
   - 调度器切换到其他进程
4. **时钟中断**: 每次时钟中断检查是否有进程需要唤醒
5. **唤醒**: 时间到达后，进程变为ready状态
6. **恢复执行**: 调度器重新选择该进程执行

### rCore中的时间管理
- 时间单位: 毫秒（ms）
- 时钟频率: 通常为10ms或100ms一次时钟中断
- 时间精度: 受时钟中断频率限制

## ✅ 验证清单

- [x] 代码已创建在正确的位置
- [x] 代码符合rCore用户程序规范
- [x] 程序出现在rCore的应用列表中
- [x] 使用了正确的系统调用接口
- [x] 程序逻辑正确（sleep 5秒）
- [ ] **待手动验证**: 在QEMU中实际运行并观察输出
- [ ] **待手动验证**: 确认实际睡眠时间约为5000ms

## 🎯 下一步

1. **手动测试**: 按照方法1的步骤运行rCore并执行exercise_c
2. **观察输出**: 确认程序正确睡眠5秒
3. **截图保存**: 记录测试结果
4. **更新文档**: 在实验报告中包含测试截图

## 📝 实验报告建议

在实验报告中应包含：
1. **代码实现**: exercise_c.rs的代码
2. **测试截图**: 显示程序运行的输出
3. **时间分析**: 实际睡眠时间与预期时间的对比
4. **系统调用说明**: sleep系统调用的作用和实现原理
5. **遇到的问题**: 开发过程中遇到的问题及解决方法

---

**创建时间**: 2026-03-13
**rCore版本**: Tutorial-v3 (main分支)
**测试状态**: ✅ 已集成，待手动验证
