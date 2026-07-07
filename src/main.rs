#![windows_subsystem = "windows"]

mod app;
mod config;
mod engine;

use app::Ra2ClickerApp;
use engine::Engine;

#[link(name = "kernel32")]
extern "system" {
    fn FreeConsole() -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn GetConsoleWindow() -> isize;
    fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
}

const SW_HIDE: i32 = 0;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;

fn main() -> eframe::Result<()> {
    unsafe {
        FreeConsole();
        ShowWindow(GetConsoleWindow(), SW_HIDE);
    }

    let cfg = config::Config::load();
    let _engine = Engine::start(cfg);

    let title = format!("ra2-clicker v{}", env!("CARGO_PKG_VERSION"));

    let (ww, wh) = (340.0f32, 185.0f32);

    let (init_x, init_y) = {
        let cfg = _engine.shared.config.read().unwrap();
        let px = cfg.window_pos_x.min(100).max(0) as f32;
        let py = cfg.window_pos_y.min(100).max(0) as f32;
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) as f32 };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) as f32 };
        let x = ((sw - ww) * px / 100.0).max(0.0);
        let y = ((sh - wh) * py / 100.0).max(0.0);
        (x, y)
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([ww, wh])
            .with_resizable(false)
            .with_position([init_x, init_y]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(Ra2ClickerApp::default()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for (path, name) in [
        (r"C:\Windows\Fonts\msyh.ttc", "chinese"),
        (r"C:\Windows\Fonts\seguisym.ttf", "symbol"),
        (r"C:\Windows\Fonts\seguiemj.ttf", "emoji"),
    ] {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_owned(data).into());
        }
    }
    for family in &[egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let list = fonts.families.entry(family.clone()).or_default();
        for name in ["emoji", "symbol", "chinese"] {
            if fonts.font_data.contains_key(name) && !list.contains(&name.to_string()) {
                list.insert(0, name.to_string());
            }
        }
    }
    ctx.set_fonts(fonts);
}
