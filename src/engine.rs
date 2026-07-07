use crate::config::Config;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

static SHARED: OnceLock<Arc<SharedState>> = OnceLock::new();

pub struct SharedState {
    pub config: RwLock<Config>,
    pub is_clicking: AtomicBool,
    pub hotkey_down: AtomicBool,
    pub hook_tid: AtomicU32,
}

impl SharedState {
    pub fn new(config: Config) -> Self {
        Self {
            config: RwLock::new(config),
            is_clicking: AtomicBool::new(false),
            hotkey_down: AtomicBool::new(false),
            hook_tid: AtomicU32::new(0),
        }
    }
}

#[allow(dead_code)]
pub struct Engine {
    pub shared: Arc<SharedState>,
    handle: Option<JoinHandle<()>>,
}

impl Engine {
    pub fn start(config: Config) -> Self {
        let shared = Arc::new(SharedState::new(config));
        SHARED.set(shared.clone()).ok().expect("Engine already started");

        let shared_clone = shared.clone();
        let handle = thread::Builder::new()
            .name("hook-thread".into())
            .spawn(move || unsafe { hook_thread(shared_clone) })
            .expect("Failed to spawn hook thread");

        Self {
            shared,
            handle: Some(handle),
        }
    }
}

pub fn shared() -> &'static Arc<SharedState> {
    SHARED.get().expect("Engine not initialized")
}

// ═══════════════════════════════════════════
// Windows API FFI — direct extern declarations
// ═══════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KBDLLHOOKSTRUCT {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MSLLHOOKSTRUCT {
    pt: POINT,
    mouse_data: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
struct INPUT {
    type_: u32,
    u: INPUT_0,
}

#[repr(C)]
union INPUT_0 {
    mi: MOUSEINPUT,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MOUSEINPUT {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

impl INPUT {
    fn mouse(dw_flags: u32) -> Self {
        Self {
            type_: 0,
            u: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouse_data: 0,
                    dw_flags,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }
}

#[repr(C)]
struct PROCESSENTRY32W {
    dw_size: u32,
    cnt_usage: u32,
    th32_process_id: u32,
    th32_default_heap_id: usize,
    th32_module_id: u32,
    cnt_threads: u32,
    th32_parent_process_id: u32,
    pc_pri_class_base: i32,
    dw_flags: u32,
    sz_exe_file: [u16; 260],
}

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(
        id_hook: i32,
        lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
        hmod: isize,
        dw_thread_id: u32,
    ) -> isize;
    fn UnhookWindowsHookEx(hhk: isize) -> i32;
    fn CallNextHookEx(hhk: isize, n_code: i32, w_param: usize, l_param: isize) -> isize;
    fn GetMessageW(lp_msg: *mut MSG, h_wnd: isize, msg_min: u32, msg_max: u32) -> i32;
    fn GetModuleHandleW(lp_module_name: *const u16) -> isize;
    fn SendInput(c_inputs: u32, p_inputs: *const INPUT, cb_size: i32) -> u32;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn GetCursorPos(lp_point: *mut POINT) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> isize;
    fn Process32FirstW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
    fn Process32NextW(h_snapshot: isize, lppe: *mut PROCESSENTRY32W) -> i32;
    fn CloseHandle(h_object: isize) -> i32;
    fn GetCurrentThreadId() -> u32;
}

const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_RBUTTONDOWN: u32 = 0x0204;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const TH32CS_SNAPPROCESS: u32 = 0x00000002;

// ═══════════════════════════════════════════
// Hook thread — runs message loop for LL hooks
// ═══════════════════════════════════════════

unsafe fn hook_thread(shared: Arc<SharedState>) {
    let tid = GetCurrentThreadId();
    shared.hook_tid.store(tid, Ordering::Relaxed);

    let hmod = GetModuleHandleW(std::ptr::null());
    let h_kb = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_proc, hmod, 0);
    let h_mouse = SetWindowsHookExW(WH_MOUSE_LL, mouse_proc, hmod, 0);

    let mut msg: MSG = std::mem::zeroed();
    loop {
        if GetMessageW(&mut msg, 0, 0, 0) == 0 {
            break;
        }
    }

    UnhookWindowsHookEx(h_kb);
    UnhookWindowsHookEx(h_mouse);
}

// ═══════════════════════════════════════════
// Hook callbacks
// ═══════════════════════════════════════════

unsafe extern "system" fn keyboard_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 {
        on_keyboard(w_param as u32, l_param);
    }
    CallNextHookEx(0, code, w_param, l_param)
}

unsafe extern "system" fn mouse_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 {
        on_mouse(w_param as u32, l_param);
    }
    CallNextHookEx(0, code, w_param, l_param)
}

unsafe fn on_keyboard(msg: u32, l_param: isize) {
    let kb = *(l_param as *const KBDLLHOOKSTRUCT);
    if kb.flags & 0x10 != 0 {
        return;
    }

    let shared = SHARED.get().unwrap();
    let cfg = shared.config.read().unwrap();
    if !cfg.click_on || kb.vk_code != cfg.hotkey_code {
        return;
    }
    let ol = cfg.use_ra2_ol_style;
    drop(cfg);

    match msg {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            shared.hotkey_down.store(true, Ordering::Relaxed);
            if !ol && !shared.is_clicking.load(Ordering::Relaxed) && !cursor_out_of_bounds(shared) {
                trigger_clicks(shared, false);
            }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            shared.hotkey_down.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
}

unsafe fn on_mouse(msg: u32, l_param: isize) {
    let is_right = match msg {
        WM_LBUTTONDOWN => false,
        WM_RBUTTONDOWN => true,
        _ => return,
    };
    let ms = *(l_param as *const MSLLHOOKSTRUCT);
    if ms.flags & 0x10 != 0 {
        return;
    }

    let shared = SHARED.get().unwrap();
    if !shared.hotkey_down.load(Ordering::Relaxed) || shared.is_clicking.load(Ordering::Relaxed) {
        return;
    }

    let cfg = shared.config.read().unwrap();
    if !cfg.click_on || !cfg.use_ra2_ol_style {
        return;
    }
    let allowed = if is_right { cfg.right_click } else { cfg.left_click };
    drop(cfg);

    if !allowed || cursor_out_of_bounds(shared) {
        return;
    }
    trigger_clicks(shared, is_right);
}

fn cursor_out_of_bounds(shared: &SharedState) -> bool {
    let cfg = shared.config.read().unwrap();
    if !cfg.enable_ra2_mode {
        return false;
    }
    let sw = if cfg.screen_width > 0 {
        cfg.screen_width as i32
    } else {
        unsafe { GetSystemMetrics(SM_CXSCREEN) }
    };
    let bw = cfg.construction_bar_width as i32;
    drop(cfg);

    let mut pos = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut pos) } == 0 {
        return true;
    }
    (sw - pos.x) > bw
}

fn trigger_clicks(shared: &Arc<SharedState>, is_right: bool) {
    if shared.is_clicking.swap(true, Ordering::Relaxed) {
        return;
    }

    let (count, interval, ol_style) = {
        let cfg = shared.config.read().unwrap();
        (cfg.click_counts, cfg.click_interval_ms, cfg.use_ra2_ol_style)
    };

    let shared = Arc::clone(shared);
    thread::Builder::new()
        .name("click-thread".into())
        .spawn(move || {
            for _ in 0..count {
                if !shared.is_clicking.load(Ordering::Relaxed) {
                    break;
                }
                unsafe { send_click(is_right); }
                thread::sleep(Duration::from_millis(interval as u64));
            }
            shared.is_clicking.store(false, Ordering::Relaxed);
            if ol_style {
                shared.hotkey_down.store(false, Ordering::Relaxed);
            }
        })
        .expect("Failed to spawn click thread");
}

unsafe fn send_click(right: bool) {
    const DOWN_L: u32 = 0x0002;
    const UP_L: u32 = 0x0004;
    const DOWN_R: u32 = 0x0008;
    const UP_R: u32 = 0x0010;

    let (down, up) = if right { (DOWN_R, UP_R) } else { (DOWN_L, UP_L) };
    let d = INPUT::mouse(down);
    let u = INPUT::mouse(up);
    SendInput(1, std::ptr::addr_of!(d), std::mem::size_of::<INPUT>() as i32);
    SendInput(1, std::ptr::addr_of!(u), std::mem::size_of::<INPUT>() as i32);
}

// ═══════════════════════════════════════════
// Process detection
// ═══════════════════════════════════════════

pub fn screen_size() -> (i32, i32) {
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

pub fn is_game_running() -> bool {
    let names = {
        let cfg = SHARED.get().unwrap().config.read().unwrap();
        cfg.game_process_list.clone()
    };

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == -1isize as isize {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dw_size = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snap, &mut entry) == 0 {
            CloseHandle(snap);
            return false;
        }

        loop {
            let name = String::from_utf16_lossy(&entry.sz_exe_file);
            let name = name.trim_end_matches('\0');
            let stem = name.rsplit('.').next().unwrap_or(name);
            if names.iter().any(|n| n == stem || n == name) {
                CloseHandle(snap);
                return true;
            }
            if Process32NextW(snap, &mut entry) == 0 {
                break;
            }
        }
        CloseHandle(snap);
        false
    }
}
