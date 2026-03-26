# rCore-Tutorial 学习记录

## 2025-03-21: 报告编写

### 任务背景
用户正在学习 rCore-Tutorial-v3 教程，已完成前六章的学习，需要写一份报告。

### 问题发现
1. 用户原有报告中 Ch5 和 Ch6 内容搞反了
2. 正确顺序：Ch5=进程，Ch6=文件系统

### 解决方案
1. 从官方网站获取各章节的正确信息：https://rcore-os.cn/rCore-Tutorial-Book-v3/
2. 按用户要求的结构重写报告：
   - 每章核心知识点概述
   - 核心实现内容（关键 struct + 基本接口）

### 报告结构

**每章包含：**
1. 核心知识点概述
2. 核心实现内容（关键数据结构 + 核心接口）

**各章主题：**
- Ch1: 应用程序与基本执行环境（SBI 调用、裸机启动）
- Ch2: 批处理系统（Trap 机制、特权级切换、系统调用）
- Ch3: 多道程序与分时多任务（任务切换、调度）
- Ch4: 地址空间（虚拟内存、SV39 页表）
- Ch5: 进程（fork/exec/waitpid、进程控制块）
- Ch6: 文件系统（easy-fs、inode 抽象）

### 输出文件
`/mnt/d/Tomato_Fish/豫文化课/新时代/大二春/操作系统/reports/rCore-Tutorial-v3-报告.md`

### 关键数据结构汇总

| 章节 | 关键 Struct |
|------|-------------|
| Ch2 | `TrapContext` |
| Ch3 | `TaskContext`, `TaskControlBlock`, `TaskStatus` |
| Ch4 | `PageTableEntry`, `MapArea`, `MemorySet` |
| Ch5 | `ProcessControlBlock`, `TaskUserRes` |
| Ch6 | `File` trait, `OSInode`, `UserBuffer` |

### 参考资料
- https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter*/index.html
