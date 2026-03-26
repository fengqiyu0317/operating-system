//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;

// User memory range - app is loaded at 0x80400000
// User stack is at 0x80400000 - 0x1000
const USER_STACK_BASE: usize = 0x80400000 - 0x1000;
const USER_STACK_TOP: usize = 0x80400000;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;
const KERNEL_BASE: usize = 0x80000000;

/// Check if a buffer is within user address space
fn check_user_buffer(buf: *const u8, len: usize) -> bool {
    let addr = buf as usize;
    // Check null pointer
    if addr == 0 {
        return false;
    }
    // Check if buffer is in user stack range
    if addr >= USER_STACK_BASE && addr + len <= USER_STACK_TOP {
        return true;
    }
    // Check if buffer is in app memory range
    if addr >= APP_BASE_ADDRESS && addr + len <= APP_BASE_ADDRESS + APP_SIZE_LIMIT {
        return true;
    }
    // Kernel space is not allowed
    if addr >= KERNEL_BASE && addr < APP_BASE_ADDRESS {
        return false;
    }
    false
}

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            // Check buffer validity
            if !check_user_buffer(buf, len) {
                return -1;
            }
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            -1  // Return -1 for unsupported file descriptors
        }
    }
}
