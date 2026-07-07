use crate::engine::{self, is_game_running, screen_size};
use eframe::egui;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

pub struct Ra2ClickerApp {
    last_auto_detect: Instant,
    game_detected: bool,
    show_advanced: bool,
    current_h: f32,
    show_about: bool,
}

impl Default for Ra2ClickerApp {
    fn default() -> Self {
        Self {
            last_auto_detect: Instant::now(),
            game_detected: false,
            show_advanced: false,
            current_h: 185.0,
            show_about: false,
        }
    }
}

impl eframe::App for Ra2ClickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let shared = engine::shared();
        self.update_auto_detect(shared);

        let mut cfg = shared.config.read().unwrap().clone();

        if cfg.dark_mode {
            let mut v = egui::Visuals::dark();
            v.selection.bg_fill = egui::Color32::from_rgb(55, 145, 70);
            ctx.set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            v.selection.bg_fill = egui::Color32::from_rgb(35, 125, 50);
            ctx.set_visuals(v);
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if cfg.dark_mode {
            egui::SystemTheme::Dark
        } else {
            egui::SystemTheme::Light
        }));

        ctx.style_mut(|s| {
            let fs = cfg.font_size.clamp(10, 24) as f32;
            s.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(fs, egui::FontFamily::Proportional),
            );
            s.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(fs, egui::FontFamily::Proportional),
            );
            s.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(fs + 4.0, egui::FontFamily::Proportional),
            );
            s.spacing.item_spacing = egui::vec2(12.0, 3.0);
            s.spacing.button_padding = egui::vec2(8.0, 3.0);
        });

        let target_h = if self.show_advanced { 330.0 } else { 185.0 };
        if (self.current_h - target_h).abs() > 0.5 {
            self.current_h = target_h;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                340.0, target_h,
            )));
        }

        let bg = ctx.style().visuals.window_fill();
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: bg,
                inner_margin: egui::Margin::symmetric(10, 5),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let dim = dim_color(ctx.style().visuals.text_color(), 0.5);

                ui.horizontal(|ui| {
                    let clicking = shared.is_clicking.load(Ordering::Relaxed);
                    let (color, text) = if clicking {
                        (egui::Color32::YELLOW, "\u{25CF} 连点中")
                    } else if cfg.click_on {
                        (egui::Color32::GREEN, "\u{25CF} 已就绪")
                    } else {
                        (dim, "\u{25CB} 已暂停")
                    };
                    ui.colored_label(color, text);

                    if cfg.auto_detect_mode {
                        if self.game_detected {
                            ui.colored_label(egui::Color32::GREEN, "\u{25CF} 运行中");
                        } else {
                            ui.colored_label(dim, "\u{25CB} 未检测到");
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon = if cfg.dark_mode { "\u{2600}" } else { "\u{263E}" };
                        let resp = ui
                            .add(
                                egui::Label::new(egui::RichText::new(icon).size(18.0))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text(if cfg.dark_mode {
                                "浅色模式"
                            } else {
                                "深色模式"
                            });
                        if resp.clicked() {
                            cfg.dark_mode = !cfg.dark_mode;
                        }
                        let about = ui
                            .add(
                                egui::Label::new(egui::RichText::new("\u{2139}").size(18.0))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text("关于");
                        if about.clicked() {
                            self.show_about = true;
                        }
                    });
                });

                ui.add_space(5.0);

                toggle_row(ui, &mut cfg.auto_detect_mode, "自动检测进程", &mut cfg.click_on, "连点开关");

                ui.add_space(2.0);

                ui.add(
                    egui::Slider::new(&mut cfg.click_counts, 0..=30)
                        .text("连点次数")
                        .clamping(egui::SliderClamping::Always),
                );

                ui.add_space(2.0);

                toggle_row(ui, &mut cfg.left_click, "左键连点", &mut cfg.right_click, "右键连点");

                ui.add_space(2.0);

                radio_row(ui, &mut cfg.hotkey_code);

                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    if ui.button("编辑配置文件").clicked() {
                        if let Ok(p) = config_path() {
                            let _ = Command::new("notepad").arg(&p).spawn();
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        if ui.button("更多设置").clicked() {
                            self.show_advanced = !self.show_advanced;
                        }
                    });
                });

                if self.show_advanced {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.label(egui::RichText::new("高级设置").strong().size(13.0));
                    ui.add(
                        egui::Slider::new(&mut cfg.click_interval_ms, 1..=200)
                            .text("点击间隔 (ms)")
                            .clamping(egui::SliderClamping::Always),
                    );
                    if ui
                        .radio(
                            cfg.use_ra2_ol_style,
                            "OL 风格 (按住热键 + 点击触发)",
                        )
                        .clicked()
                    {
                        cfg.use_ra2_ol_style = !cfg.use_ra2_ol_style;
                    }
                    if ui
                        .radio(
                            cfg.enable_ra2_mode,
                            "RA2 模式 (仅在建造栏区域连点)",
                        )
                        .clicked()
                    {
                        cfg.enable_ra2_mode = !cfg.enable_ra2_mode;
                    }
                    if cfg.enable_ra2_mode {
                        ui.add(
                            egui::Slider::new(&mut cfg.construction_bar_width, 50..=500)
                                .text("建造栏宽度 (px)")
                                .clamping(egui::SliderClamping::Always),
                        );
                    }
                }
            });

        if self.show_about {
            let fs = cfg.font_size + 2;
            egui::Window::new("关于")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("ra2-clicker").strong().size(fs as f32));
                        ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).size(fs as f32));
                    });
                    ui.label(egui::RichText::new("借鉴 ra2-mouse-click 的 Rust 重写版").size(fs as f32));
                    ui.label(egui::RichText::new("作者: cmixed  邮箱: cmixed@foxmail.com").size(fs as f32));
                    ui.hyperlink_to("https://github.com/cmixed/ra2-clicker", "https://github.com/cmixed/ra2-clicker");
                    ui.add_space(4.0);
                    if ui.button("关闭").clicked() {
                        self.show_about = false;
                    }
                });
        }

        if cfg.remember_position {
            if let Some(outer) = ctx.input(|i| i.viewport().outer_rect) {
                let (sw, sh) = screen_size();
                let sw = sw as f32;
                let sh = sh as f32;
                let ow = outer.width();
                let oh = outer.height();
                if sw > ow && sh > oh {
                    cfg.window_pos_x = (outer.min.x / (sw - ow) * 100.0).round().clamp(0.0, 100.0) as u32;
                    cfg.window_pos_y = (outer.min.y / (sh - oh) * 100.0).round().clamp(0.0, 100.0) as u32;
                }
            }
        }

        let _ = cfg.save();
        *shared.config.write().unwrap() = cfg;
    }
}

impl Ra2ClickerApp {
    fn update_auto_detect(&mut self, shared: &engine::SharedState) {
        let interval = {
            let c = shared.config.read().unwrap();
            if !c.auto_detect_mode {
                return;
            }
            c.auto_detect_interval_ms
        };
        if self.last_auto_detect.elapsed() >= Duration::from_millis(interval as u64) {
            self.last_auto_detect = Instant::now();
            self.game_detected = is_game_running();
            let mut c = shared.config.write().unwrap();
            if self.game_detected && !c.click_on {
                c.click_on = true;
            } else if !self.game_detected && c.click_on {
                c.click_on = false;
            }
        }
    }
}

fn toggle_row(ui: &mut egui::Ui, a: &mut bool, la: &str, b: &mut bool, lb: &str) {
    ui.horizontal(|ui| {
        let gap = ui.spacing().item_spacing.x;
        let col = (ui.available_width() - gap) / 2.0;
        let h = 22.0;
        let (r1, _) = ui.allocate_exact_size(egui::vec2(col, h), egui::Sense::hover());
        let mut c1 = ui.new_child(egui::UiBuilder::new().max_rect(r1).layout(egui::Layout::left_to_right(egui::Align::Center)));
        toggle(&mut c1, a);
        c1.label(la);
        let (r2, _) = ui.allocate_exact_size(egui::vec2(col, h), egui::Sense::hover());
        let mut c2 = ui.new_child(egui::UiBuilder::new().max_rect(r2).layout(egui::Layout::left_to_right(egui::Align::Center)));
        toggle(&mut c2, b);
        c2.label(lb);
    });
}

fn radio_row(ui: &mut egui::Ui, selected: &mut u32) {
    ui.horizontal(|ui| {
        let gap = ui.spacing().item_spacing.x;
        let col = (ui.available_width() - gap * 2.0) / 3.0;
        let h = 22.0;
        for (code, label) in [
            (0xA0u32, "Shift连点"),
            (0xA2u32, "Ctrl连点"),
            (0xA4u32, "Alt连点"),
        ] {
            let (r, _) = ui.allocate_exact_size(egui::vec2(col, h), egui::Sense::hover());
            let mut c = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
            c.radio_value(selected, code, label);
        }
    });
}

fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let size = egui::vec2(34.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
    }
    if ui.is_rect_visible(rect) {
        let rounding = egui::CornerRadius::same(9);
        if *on {
            let c = ui.style().visuals.text_color();
            let on_color = if (c.r() as u16 + c.g() as u16 + c.b() as u16) / 3 > 128 {
                egui::Color32::from_rgb(82, 94, 84)
            } else {
                egui::Color32::from_rgb(40, 110, 50)
            };
            ui.painter().rect_filled(rect, rounding, on_color);
            let r = rect.height() / 2.0 - 2.0;
            ui.painter()
                .circle_filled(egui::pos2(rect.right() - r - 2.0, rect.center().y), r, egui::Color32::WHITE);
        } else {
            let bg = ui.style().visuals.widgets.inactive.bg_fill;
            ui.painter().rect_filled(rect, rounding, bg);
            let r = rect.height() / 2.0 - 2.0;
            let knob = dim_color(ui.style().visuals.text_color(), 0.55);
            ui.painter()
                .circle_filled(egui::pos2(rect.left() + r + 2.0, rect.center().y), r, knob);
        }
    }
    response
}

fn dim_color(color: egui::Color32, factor: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        (color.r() as f32 * factor) as u8,
        (color.g() as f32 * factor) as u8,
        (color.b() as f32 * factor) as u8,
    )
}

fn config_path() -> Result<String, ()> {
    let mut p = std::env::current_exe().map_err(|_| ())?;
    p.set_extension("toml");
    Ok(p.to_string_lossy().to_string())
}
