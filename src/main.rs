#![windows_subsystem = "windows"]

mod app;
mod config;
mod engine;

use app::Ra2ClickerApp;
use engine::Engine;

#[link(name = "user32")]
extern "system" {
    fn GetSystemMetrics(nIndex: i32) -> i32;
}

const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const SM_CYCAPTION: i32 = 4;
const SM_CXFIXEDFRAME: i32 = 7;

fn main() -> eframe::Result<()> {
    let cfg = config::Config::load();
    let _engine = Engine::start(cfg);

    let title = format!("ra2-clicker v{}", env!("CARGO_PKG_VERSION"));

    let (iw, ih) = (340.0f32, 185.0f32);
    let (px, py) = {
        let c = _engine.shared.config.read().unwrap();
        (c.window_pos_x.min(100).max(0) as f32, c.window_pos_y.min(100).max(0) as f32)
    };

    let (ow, oh) = unsafe {
        let bw = GetSystemMetrics(SM_CXFIXEDFRAME) as f32;
        let cap = GetSystemMetrics(SM_CYCAPTION) as f32;
        (iw + bw * 2.0, ih + cap + bw)
    };

    let (ox, oy) = unsafe {
        let sw = GetSystemMetrics(SM_CXSCREEN) as f32;
        let sh = GetSystemMetrics(SM_CYSCREEN) as f32;
        (((sw - ow) * px / 100.0).max(0.0), ((sh - oh) * py / 100.0).max(0.0))
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([iw, ih])
            .with_resizable(false)
            .with_position([ox, oy]),
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
