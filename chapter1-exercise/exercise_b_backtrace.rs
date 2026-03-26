// 练习B: 打印调用栈信息
// 编译方法1: rustc exercise_b_backtrace.rs -o exercise_b_backtrace
// 编译方法2: 创建Cargo项目使用backtrace crate

fn main() {
    println!("调用栈信息演示：\n");

    // 调用一系列函数以创建调用栈
    first_function();

    // 方法3: 演示如何通过panic获取backtrace
    // println!("\n提示: 设置 RUST_BACKTRACE=1 环境变量可以查看panic时的完整调用栈");
}

fn first_function() {
    println!("在 first_function() 中");
    second_function();
}

fn second_function() {
    println!("在 second_function() 中");
    third_function();
}

fn third_function() {
    println!("在 third_function() 中");
    println!("\n=== 手动追踪调用链 ===");
    println!("当前调用链:");
    println!("  main()");
    println!("    └─> first_function()");
    println!("        └─> second_function()");
    println!("            └─> third_function() [当前]");

    // 提示使用GDB或其他方法查看真实调用栈
    print_debug_tips();
}

fn print_debug_tips() {
    println!("\n=== 如何查看真实的调用栈 ===");
    println!("方法1: 使用GDB");
    println!("  $ gdb ./exercise_b_backtrace");
    println!("  (gdb) break third_function");
    println!("  (gdb) run");
    println!("  (gdb) backtrace");

    println!("\n方法2: 使用rust-lldb");
    println!("  $ rust-lldb ./exercise_b_backtrace");
    println!("  (lldb) breakpoint set --name third_function");
    println!("  (lldb) run");
    println!("  (lldb) bt");

    println!("\n方法3: 使用Cargo项目 + backtrace crate");
    println!("  在Cargo.toml中添加:");
    println!("    [dependencies]");
    println!("    backtrace = \"0.3\"");
    println!("  然后在代码中使用:");
    println!("    let bt = backtrace::Backtrace::new();");
    println!("    println!(\"{{:?}}\", bt);");
}
