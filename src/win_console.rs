//! Windows 控制台处理。
//!
//! Windows 的 release 构建使用 GUI 子系统（`windows_subsystem = "windows"`），
//! 双击运行时不会再弹出黑色控制台窗口；但这样从终端执行
//! `job-tracker.exe --report` 也不会有输出了。
//!
//! 这里在启动时做一次「补挂」：如果当前进程没有可用的标准输出，
//! 就尝试附加到父进程（终端）的控制台，并把 stdout/stderr 指过去，
//! 于是命令行输出仍然会打印在调用它的那个终端里。
//! 双击启动（没有父控制台）时附加会失败，什么也不做。

use std::{ffi::c_void, ptr};

/// `AttachConsole(ATTACH_PARENT_PROCESS)`
const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
/// `STD_OUTPUT_HANDLE`
const STD_OUTPUT_HANDLE: u32 = (-11i32) as u32;
/// `STD_ERROR_HANDLE`
const STD_ERROR_HANDLE: u32 = (-12i32) as u32;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
const OPEN_EXISTING: u32 = 3;
const INVALID_HANDLE_VALUE: isize = -1;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn AttachConsole(dw_process_id: u32) -> i32;
    fn GetStdHandle(n_std_handle: u32) -> *mut c_void;
    fn SetStdHandle(n_std_handle: u32, h_handle: *mut c_void) -> i32;
    fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *mut c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: *mut c_void,
    ) -> *mut c_void;
}

fn is_valid(handle: *mut c_void) -> bool {
    !handle.is_null() && handle as isize != INVALID_HANDLE_VALUE
}

/// 让命令行输出在「GUI 子系统」构建下也能显示在父终端里。
///
/// 任何一步失败都只是「看不到输出」，不会 panic，也不影响图形界面。
pub fn attach_parent_console() {
    unsafe {
        // 已经有可用的标准输出（例如 `job-tracker --report > out.txt` 被重定向到文件）
        // 就不要动它，否则会把输出抢到控制台上。
        if is_valid(GetStdHandle(STD_OUTPUT_HANDLE)) {
            return;
        }
        // 双击启动时没有父控制台，这里会失败，直接返回。
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            return;
        }

        let conout: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
        let handle = CreateFileW(
            conout.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        );
        if !is_valid(handle) {
            return;
        }

        SetStdHandle(STD_OUTPUT_HANDLE, handle);
        SetStdHandle(STD_ERROR_HANDLE, handle);
    }
}
