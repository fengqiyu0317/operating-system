//! App management syscalls
use crate::batch::{run_next_app, get_current_task_info, get_current_app_id, print_syscall_stats, record_app_end_time, get_app_exec_time};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    // 获取当前应用 ID
    let app_id = get_current_app_id();
    // 记录应用结束时间
    record_app_end_time(app_id);
    // 获取执行时间
    let exec_cycles = get_app_exec_time(app_id);
    // 转换为微秒（假设 CPU 频率为 10MHz，需要后续测试调整）
    let exec_time_us = exec_cycles / 10;
    println!("[kernel] Application exited with code {}", exit_code);
    println!("[kernel] Execution time: {} us ({} cycles)", exec_time_us, exec_cycles);
    // 输出系统调用统计信息
    print_syscall_stats(app_id);
    run_next_app()
}

/// get current task info: task id and task name
/// buf: pointer to buffer to store task name
/// max_len: maximum length of buffer
/// returns task id on success, or negative error code
pub fn sys_get_taskinfo(buf: *mut u8, max_len: usize) -> isize {
    if buf.is_null() || max_len == 0 {
        return -1;
    }
    let (task_id, task_name) = get_current_task_info();
    let name_bytes = task_name.as_bytes();
    let copy_len = core::cmp::min(name_bytes.len(), max_len - 1);

    unsafe {
        // 复制任务名称到用户缓冲区
        core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), buf, copy_len);
        // 添加 null 终止符
        *buf.add(copy_len) = 0;
    }

    task_id as isize
}
