use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

// ANSI 颜色码定义
pub const ANSI_COLOR_RED: &str = "\x1b[31m";
pub const ANSI_COLOR_GREEN: &str = "\x1b[32m";
pub const ANSI_COLOR_YELLOW: &str = "\x1b[33m";
pub const ANSI_COLOR_BLUE: &str = "\x1b[34m";
pub const ANSI_COLOR_RESET: &str = "\x1b[0m";

// 彩色打印函数
pub fn print_color(color: &str, args: fmt::Arguments) {
    print!("{}{}{}", color, args, ANSI_COLOR_RESET);
}

// 彩色日志宏
#[macro_export]
macro_rules! error {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_RED,
            format_args!(concat!("[ERROR] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_YELLOW,
            format_args!(concat!("[WARN] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! info {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_BLUE,
            format_args!(concat!("[INFO] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt:literal $(, $($arg:tt)+)?) => {
        $crate::console::print_color(
            $crate::console::ANSI_COLOR_GREEN,
            format_args!(concat!("[DEBUG] ", $fmt, "\n") $(, $($arg)+)?)
        );
    }
}
