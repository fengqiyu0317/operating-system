// 练习A: 显示当前目录下的文件名
// 使用Rust实现类似ls的功能

use std::fs;
use std::path::Path;

fn main() {
    // 获取当前目录路径
    let current_dir = Path::new(".");

    println!("当前目录下的文件：");

    // 读取并遍历当前目录
    match fs::read_dir(current_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        // 获取文件名并打印
                        let file_name = entry.file_name();
                        println!("{}", file_name.to_string_lossy());
                    }
                    Err(e) => {
                        eprintln!("读取条目失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("无法读取当前目录: {}", e);
        }
    }
}
