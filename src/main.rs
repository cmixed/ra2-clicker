#![windows_subsystem = "windows"]

mod app;
mod config;
mod engine;

use app::Ra2ClickerApp;
use engine::Engine;

fn main() -> eframe::Result<()> {
    let cfg = config::Config::load();
    let _engine = Engine::start(cfg);

    let title = format!("ra2-clicker v{}", env!("CARGO_PKG_VERSION"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([340.0, 185.0])
            .with_resizable(false),
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
