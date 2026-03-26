// 练习B: 打印调用栈信息（Cargo版本，使用backtrace crate）
// 编译和运行: cargo run --release

use backtrace::Backtrace;

fn main() {
    println!("调用栈信息演示（使用backtrace crate）：\n");

    // 调用一系列函数以创建调用栈
    first_function();

    println!("\n=== 程序正常结束 ===");
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
    println!("在 third_function() 中\n");

    // 使用backtrace crate获取调用栈
    println!("=== 当前调用栈 ===");
    let bt = Backtrace::new();
    println!("{:?}", bt);

    println!("\n=== 格式化的调用链 ===");
    print_backtrace_summary(&bt);
}

fn print_backtrace_summary(bt: &Backtrace) {
    let frames = bt.frames();
    println!("调用链（从底到顶）：");

    for (index, frame) in frames.iter().enumerate() {
        // 打印帧号
        print!("  #{}", index);

        // 尝试获取符号名称
        if let Some(symbol) = frame.symbols().first() {
            if let Some(name) = symbol.name() {
                print!(" - {}", name);
            }
            if let Some(location) = symbol.filename() {
                print!(" ({:?}:{:?})", location, symbol.lineno());
            }
        }

        println!();
    }
}
