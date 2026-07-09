#![windows_subsystem = "windows"]

mod app;
mod config;
mod engine;

slint::include_modules!();

use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::Ordering;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_ELEVATION: u32 = 20;
const SW_SHOWNORMAL: i32 = 1;

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_cmd: i32,
    ) -> isize;
}

#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(
        process_handle: isize,
        desired_access: u32,
        token_handle: *mut isize,
    ) -> i32;
    fn GetTokenInformation(
        token_handle: isize,
        token_information_class: u32,
        token_information: *mut std::ffi::c_void,
        token_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn CloseHandle(h_object: isize) -> i32;
}

fn is_admin() -> bool {
    unsafe {
        let mut token: isize = 0;
        if OpenProcessToken(-1isize, TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation: u32 = 0;
        let mut size: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TOKEN_ELEVATION,
            &mut elevation as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
            &mut size,
        );
        let _ = CloseHandle(token);
        ok != 0 && elevation != 0
    }
}

fn elevate() {
    let exe = std::env::current_exe().expect("failed to get exe path");
    let wide: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        ShellExecuteW(
            0,
            "runas\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            wide.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

fn main() {
    if !is_admin() {
        elevate();
        return;
    }

    let cfg = config::Config::load();
    let engine = engine::Engine::start(cfg);

    let ui = AppWindow::new().expect("Failed to create UI");
    app::setup(&ui, &engine);
    app::position_window(&ui, &engine);
    ui.run().expect("Failed to run UI");

    if engine.shared.config.read().unwrap().remember_position {
        let mut cfg = engine.shared.config.write().unwrap();
        cfg.window_pos_x = engine.shared.window_x.load(Ordering::Relaxed).max(0) as u32;
        cfg.window_pos_y = engine.shared.window_y.load(Ordering::Relaxed).max(0) as u32;
        cfg.save().ok();
    }
}
