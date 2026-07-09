#![windows_subsystem = "windows"]

mod app;
mod config;
mod engine;

slint::include_modules!();

use std::sync::atomic::Ordering;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
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
