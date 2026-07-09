use crate::config::Config;
use crate::engine::{self, is_game_running};
use crate::AppWindow;
use slint::ComponentHandle;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const u32,
        cb_attribute: u32,
    ) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn FindWindowW(
        lp_class_name: *const u16,
        lp_window_name: *const u16,
    ) -> isize;
}

extern "system" {
    fn GetSystemMetrics(n_index: i32) -> i32;
}

fn set_title_bar_dark_mode(dark: bool) {
    let title: Vec<u16> = OsStr::new(&format!("ra2-clicker v{}", env!("CARGO_PKG_VERSION")))
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd == 0 {
        return;
    }
    let mode: u32 = if dark { 1 } else { 0 };
    unsafe {
        DwmSetWindowAttribute(hwnd, 20, &mode as *const u32, std::mem::size_of::<u32>() as u32);
    }
}

pub fn setup(ui: &AppWindow, engine: &engine::Engine) {
    let shared = &engine.shared;

    sync_from_config(ui, shared);

    ui.set_app_version(env!("CARGO_PKG_VERSION").into());
    ui.set_is_clicking(shared.is_clicking.load(Ordering::Relaxed));

    connect_callbacks(ui, shared);
    start_poll_timer(ui, shared);

    let dark = shared.config.read().unwrap().dark_mode;
    let timer = Box::new(slint::Timer::default());
    timer.start(
        slint::TimerMode::SingleShot,
        Duration::from_millis(50),
        move || set_title_bar_dark_mode(dark),
    );
    std::mem::forget(timer);
}

pub fn position_window(ui: &AppWindow, engine: &engine::Engine) {
    let cfg = engine.shared.config.read().unwrap();
    let px = cfg.window_pos_x;
    let py = cfg.window_pos_y;
    drop(cfg);

    let sw = unsafe { GetSystemMetrics(0) };
    let sh = unsafe { GetSystemMetrics(1) };
    let w = 340i32;
    let h = 380i32;

    let max_x = (sw - w).max(0);
    let max_y = (sh - h).max(0);

    let x = if px == u32::MAX { max_x / 2 } else { (px as i32).clamp(0, max_x) };
    let y = if py == u32::MAX { max_y / 2 } else { (py as i32).clamp(0, max_y) };

    let _ = ui.window().set_position(slint::PhysicalPosition { x, y });
}

fn connect_callbacks(ui: &AppWindow, shared: &Arc<engine::SharedState>) {
    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_click_on(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.click_on = !cfg.click_on;
        let v = cfg.click_on;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_click_on(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_auto_detect(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.auto_detect_mode = !cfg.auto_detect_mode;
        let v = cfg.auto_detect_mode;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_auto_detect_mode(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_left_click(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.left_click = !cfg.left_click;
        let v = cfg.left_click;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_left_click(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_right_click(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.right_click = !cfg.right_click;
        let v = cfg.right_click;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_right_click(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_hotkey(move |code| {
        let mut cfg = sh.config.write().unwrap();
        if cfg.hotkey_code == code as u32 {
            cfg.hotkey_code = 0;
        } else {
            cfg.hotkey_code = code as u32;
        }
        cfg.sync_from_code();
        let code = cfg.hotkey_code;
        drop(cfg);
        if let Some(ui) = weak.upgrade() {
            ui.set_hotkey_code(code as i32);
        }
    });

    let weak = ui.as_weak();
    ui.on_toggle_advanced(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_show_advanced(!ui.get_show_advanced());
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_ol_style(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.use_ra2_ol_style = !cfg.use_ra2_ol_style;
        let v = cfg.use_ra2_ol_style;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_use_ra2_ol_style(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_ra2_mode(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.enable_ra2_mode = !cfg.enable_ra2_mode;
        let v = cfg.enable_ra2_mode;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_enable_ra2_mode(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_remember_position(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.remember_position = !cfg.remember_position;
        let v = cfg.remember_position;
        cfg.save().ok();
        if let Some(ui) = weak.upgrade() {
            ui.set_remember_position(v);
        }
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_edit_config(move || {
        let path = Config::path();
        let child = std::process::Command::new("notepad")
            .arg(&path)
            .spawn();
        let weak = weak.clone();
        let sh = sh.clone();
        thread::spawn(move || {
            let mut child = match child {
                Ok(c) => c,
                _ => return,
            };
            thread::sleep(Duration::from_millis(500));
            let mut initial = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());
            loop {
                match child.try_wait() {
                    Ok(Some(_)) | Err(_) => break,
                    _ => {}
                }
                thread::sleep(Duration::from_millis(500));
                let mtime = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());
                if let (Some(old), Some(new)) = (initial, mtime) {
                    if new > old {
                        initial = mtime;
                        let weak = weak.clone();
                        let sh = sh.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak.upgrade() {
                                apply_config_file(&ui, &sh);
                            }
                        });
                    }
                }
            }
        });
    });

    let weak = ui.as_weak();
    let sh = shared.clone();
    ui.on_toggle_dark_mode(move || {
        let mut cfg = sh.config.write().unwrap();
        cfg.dark_mode = !cfg.dark_mode;
        let v = cfg.dark_mode;
        cfg.save().ok();
        drop(cfg);
        set_title_bar_dark_mode(v);
        if let Some(ui) = weak.upgrade() {
            apply_theme(&ui, v);
        }
    });

    let sh = shared.clone();
    ui.on_click_counts_changed(move |value| {
        let mut cfg = sh.config.write().unwrap();
        cfg.click_counts = value as u32;
        cfg.save().ok();
    });

    let sh = shared.clone();
    ui.on_click_interval_changed(move |value| {
        let mut cfg = sh.config.write().unwrap();
        cfg.click_interval_ms = value as u32;
        cfg.save().ok();
    });

    let sh = shared.clone();
    ui.on_construction_bar_width_changed(move |value| {
        let mut cfg = sh.config.write().unwrap();
        cfg.construction_bar_width = value as u32;
        cfg.save().ok();
    });

    let weak = ui.as_weak();
    ui.on_open_about(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_show_about(true);
        }
    });

    let weak = ui.as_weak();
    ui.on_close_about(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_show_about(false);
        }
    });

    ui.on_open_homepage(move || {
        let _ = std::process::Command::new("explorer")
            .arg("https://gitee.com/cmixed/ra2-clicker")
            .spawn();
    });
}

fn start_poll_timer(ui: &AppWindow, shared: &Arc<engine::SharedState>) {
    let ui_handle = ui.as_weak();
    let shared = shared.clone();
    let mut last_detect = Instant::now();

    let timer = Box::new(slint::Timer::default());
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(200),
        move || {
            let ui = match ui_handle.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            ui.set_is_clicking(shared.is_clicking.load(Ordering::Relaxed));

            let pos = ui.window().position();
            shared.window_x.store(pos.x, Ordering::Relaxed);
            shared.window_y.store(pos.y, Ordering::Relaxed);

            let cfg = shared.config.read().unwrap();
            let auto_detect = cfg.auto_detect_mode;
            let interval = cfg.auto_detect_interval_ms;
            drop(cfg);

            if auto_detect && last_detect.elapsed() >= Duration::from_millis(interval as u64) {
                last_detect = Instant::now();
                let detected = is_game_running();
                ui.set_game_detected(detected);

                let mut cfg = shared.config.write().unwrap();
                if detected && !cfg.click_on {
                    cfg.click_on = true;
                    ui.set_click_on(true);
                } else if !detected && cfg.click_on {
                    cfg.click_on = false;
                    ui.set_click_on(false);
                }
            }
        },
    );
    std::mem::forget(timer);
}

fn apply_config_file(ui: &AppWindow, shared: &Arc<engine::SharedState>) {
    let path = Config::path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        _ => return,
    };
    let content = content.trim_start_matches('\u{FEFF}');
    let toml_part: String = content.lines()
        .skip_while(|line| {
            let t = line.trim();
            t.is_empty() || t.starts_with('#')
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut new_cfg = match toml::from_str::<Config>(&toml_part) {
        Ok(c) => c,
        _ => return,
    };
    new_cfg.sync_from_hotkey();
    let mut old = shared.config.write().unwrap();
    if new_cfg.dark_mode != old.dark_mode {
        set_title_bar_dark_mode(new_cfg.dark_mode);
        apply_theme(ui, new_cfg.dark_mode);
    }
    if new_cfg.click_on != old.click_on { ui.set_click_on(new_cfg.click_on); }
    if new_cfg.auto_detect_mode != old.auto_detect_mode { ui.set_auto_detect_mode(new_cfg.auto_detect_mode); }
    if new_cfg.left_click != old.left_click { ui.set_left_click(new_cfg.left_click); }
    if new_cfg.right_click != old.right_click { ui.set_right_click(new_cfg.right_click); }
    if new_cfg.hotkey_code != old.hotkey_code { ui.set_hotkey_code(new_cfg.hotkey_code as i32); }
    if new_cfg.use_ra2_ol_style != old.use_ra2_ol_style { ui.set_use_ra2_ol_style(new_cfg.use_ra2_ol_style); }
    if new_cfg.enable_ra2_mode != old.enable_ra2_mode { ui.set_enable_ra2_mode(new_cfg.enable_ra2_mode); }
    if new_cfg.click_counts != old.click_counts { ui.set_click_counts(new_cfg.click_counts as i32); }
    if new_cfg.click_interval_ms != old.click_interval_ms { ui.set_click_interval_ms(new_cfg.click_interval_ms as i32); }
    if new_cfg.construction_bar_width != old.construction_bar_width { ui.set_construction_bar_width(new_cfg.construction_bar_width as i32); }
    if new_cfg.remember_position != old.remember_position { ui.set_remember_position(new_cfg.remember_position); }
    *old = new_cfg;
}

fn apply_theme(ui: &AppWindow, dark: bool) {
    ui.set_dark_mode(dark);
    ui.set_bg(slint::Color::from_argb_encoded(if dark { 0xff1e1e1e } else { 0xffebecf0 }));
    ui.set_txt(slint::Color::from_argb_encoded(if dark { 0xffcccccc } else { 0xff222222 }));
}

fn sync_from_config(ui: &AppWindow, shared: &Arc<engine::SharedState>) {
    let cfg = shared.config.read().unwrap();
    ui.set_click_on(cfg.click_on);
    ui.set_auto_detect_mode(cfg.auto_detect_mode);
    ui.set_click_counts(cfg.click_counts as i32);
    ui.set_left_click(cfg.left_click);
    ui.set_right_click(cfg.right_click);
    ui.set_hotkey_code(cfg.hotkey_code as i32);
    ui.set_use_ra2_ol_style(cfg.use_ra2_ol_style);
    ui.set_enable_ra2_mode(cfg.enable_ra2_mode);
    ui.set_click_interval_ms(cfg.click_interval_ms as i32);
    ui.set_construction_bar_width(cfg.construction_bar_width as i32);
    ui.set_remember_position(cfg.remember_position);
    apply_theme(ui, cfg.dark_mode);
}
