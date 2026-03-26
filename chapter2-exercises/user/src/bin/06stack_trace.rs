//! # Stack Trace Printing Program
//!
//! This program demonstrates how to print call stack in bare-metal environment.
//!
//! ## RISC-V Stack Frame Structure
//!
//! In RISC-V calling convention, the stack frame layout is:
//! ```text
//! High Address
//!     +---------------+
//!     | Local Vars    |
//!     +---------------+
//!     | Saved Regs    |
//!     +---------------+
//!     | Saved RA      |  <- fp - 8 (Return Address)
//!     +---------------+
//!     | Saved FP      |  <- fp - 16 (Previous Frame Pointer)
//!     +---------------+
//!     | Argument Area |
//!     +---------------+
//! Low Address
//!
//! fp points to the high boundary of current stack frame (sp before function call)
//! ```
//!
//! ## Implementation
//!
//! 1. Use inline assembly to get current frame pointer fp
//! 2. Extract return address and previous frame pointer from stack frame
//! 3. Traverse the entire call stack chain until fp is 0

#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

/// Frame pointer type
type FramePointer = usize;

/// Maximum recursion depth for verification
const MAX_RECURSION: usize = 32;

/// Record of expected frame pointers
static mut FP_RECORDS: [FramePointer; MAX_RECURSION] = [0; MAX_RECURSION];
static mut FP_COUNT: usize = 0;

/// Stack frame structure
#[derive(Debug, Clone, Copy)]
struct StackFrame {
    fp: FramePointer,
    ra: usize,
}

impl StackFrame {
    /// Create stack frame from frame pointer
    fn from_fp(fp: FramePointer) -> Option<Self> {
        unsafe {
            if fp == 0 {
                return None;
            }

            // In RISC-V calling convention:
            // - fp points to the high boundary of current stack frame
            // - fp - 8 points to the saved return address (ra)
            // - fp - 16 points to the saved previous frame pointer (fp)
            let saved_ra = *((fp - 8) as *const usize);
            let saved_fp = *((fp - 16) as *const usize);

            Some(StackFrame {
                fp: saved_fp,
                ra: saved_ra,
            })
        }
    }

    /// Get return address
    fn return_address(&self) -> usize {
        self.ra
    }

    /// Get caller's frame pointer
    fn caller_fp(&self) -> FramePointer {
        self.fp
    }
}

/// Get current frame pointer
fn current_fp() -> FramePointer {
    unsafe {
        let fp: FramePointer;
        core::arch::asm!(
            "mv {}, fp",
            out(reg) fp
        );
        fp
    }
}

/// Record current fp
fn record_fp() {
    unsafe {
        if FP_COUNT < MAX_RECURSION {
            FP_RECORDS[FP_COUNT] = current_fp();
            FP_COUNT += 1;
        }
    }
}

/// Print call stack
fn print_stack_trace() {
    record_fp();

    // 添加一个大数组，强制改变栈帧大小
    let _large_array: [u64; 16] = [0; 16];

    println!("=== Stack Trace ===");
    println!("");

    {
        let mut current_fp = current_fp();

        // First pass: collect all frame pointers from actual stack
        let mut actual_fps: [FramePointer; 64] = [0; 64];
        let mut actual_count = 0;

        loop {
            if current_fp == 0 || actual_count >= 64 {
                break;
            }
            actual_fps[actual_count] = current_fp;
            actual_count += 1;

            let frame = match StackFrame::from_fp(current_fp) {
                Some(f) => f,
                None => break,
            };
            current_fp = frame.caller_fp();
        }

        // Debug: print all recorded FPs with their indices
        println!("--- Recorded FPs ---");
        unsafe {
            for i in 0..FP_COUNT {
                println!("FP_RECORDS[{}] = {:#x}", i, FP_RECORDS[i]);
            }
        }
        println!("");

        // Second pass: print and verify
        let mut found_count = 0;
        for i in 0..actual_count {
            current_fp = actual_fps[i];

            // Check if this fp was in our records
            let mut is_recorded = false;
            let mut record_index = 0;
            unsafe {
                for j in 0..FP_COUNT {
                    if FP_RECORDS[j] == current_fp {
                        is_recorded = true;
                        record_index = j;
                        found_count += 1;
                        break;
                    }
                }
            }

            let frame = StackFrame::from_fp(current_fp);
            if let Some(f) = frame {
                print!("[{}] Frame: {:#x} ", i, current_fp);
                if is_recorded {
                    print!("[RECORDED at index {}]", record_index);
                }
                println!("");
                println!("    RA: {:#x}", f.return_address());
                println!("    Caller FP: {:#x}", f.caller_fp());
                println!("");
            }
        }

        println!("Total {} frames", actual_count);
        println!("");
        unsafe {
            let count = FP_COUNT;
            println!("Recorded {} frame pointers", count);
            println!("Found {} of {} in stack trace", found_count, count);
            if count > 0 && found_count == count {
                println!("All recorded frame pointers verified!");
            }
        }
    }
}

/// Helper: Recursive function to generate multi-level call stack
fn recursive_function(depth: u32) {
    // Record fp at entry of each recursive call
    record_fp();

    if depth > 0 {
        println!("Recursive depth: {} (fp={:#x})", depth, current_fp());
        recursive_function(depth - 1);
    } else {
        println!("Reached recursion bottom, printing stack trace...");
        println!("");
        print_stack_trace();
    }
}

/// Helper: Create multi-level function calls
fn level_3() {
    println!("Entering level_3");
    recursive_function(5);
}

fn level_2() {
    println!("Entering level_2");
    level_3();
}

fn level_1() {
    println!("Entering level_1");
    level_2();
}

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("====================================");
    println!("RISC-V Stack Trace Printer");
    println!("====================================");
    println!("");

    println!("This program demonstrates stack trace printing");
    println!("Will create multi-level function calls and print full call stack");
    println!("");

    // Create multi-level function calls
    level_1();

    println!("");
    println!("====================================");
    println!("Program completed successfully");
    println!("====================================");

    0
}
