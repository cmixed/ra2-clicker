use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HEADER: &str = r#"# ra2-clicker 配置文件
# 保存后即刻生效，无需重启程序
#
# 连点设置
# click_on            = 连点开关 (true=开, false=关)
# click_counts        = 每次触发的连点次数 (0~30)
# click_interval_ms   = 每次点击间隔，单位毫秒 (1~200)
# left_click          = 启用左键连点 (true=启用)
# right_click         = 启用右键连点 (true=启用)
# use_ra2_ol_style    = OL 风格: 按住热键+鼠标点击才触发 (true=启用)
#
# 热键设置
# hotkey_code         = 连点触发热键
#   160 = Left Shift
#   162 = Left Ctrl
#   164 = Left Alt
#
# RA2 模式
# enable_ra2_mode     = 仅在建造栏区域连点 (true=启用)
# construction_bar_width = 建造栏宽度，单位像素 (50~500)
# screen_width        = 屏幕宽度，0 表示自动检测
#
# 自动检测游戏进程
# auto_detect_mode    = 自动检测游戏进程并开关连点 (true=启用)
# auto_detect_interval_ms = 检测间隔，单位毫秒
# game_process_list   = 需检测的游戏进程名列表
#
# 外观
# dark_mode           = 深色模式 (true=深色, false=浅色)
# font_size           = 字体大小 (10~24)

"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub click_on: bool,
    pub click_counts: u32,
    pub click_interval_ms: u32,
    pub use_ra2_ol_style: bool,
    pub left_click: bool,
    pub right_click: bool,
    pub hotkey_code: u32,
    pub enable_ra2_mode: bool,
    pub construction_bar_width: u32,
    pub screen_width: u32,
    pub auto_detect_mode: bool,
    pub auto_detect_interval_ms: u32,
    pub game_process_list: Vec<String>,
    pub dark_mode: bool,
    pub font_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            click_on: false,
            click_counts: 5,
            click_interval_ms: 10,
            use_ra2_ol_style: true,
            left_click: true,
            right_click: false,
            hotkey_code: 0xA0,
            enable_ra2_mode: true,
            construction_bar_width: 160,
            screen_width: 0,
            auto_detect_mode: true,
            auto_detect_interval_ms: 5000,
            game_process_list: vec![
                "gamemd".into(),
                "game".into(),
                "gameares".into(),
                "gamemd-spawn".into(),
                "gamemd-ares".into(),
            ],
            dark_mode: true,
            font_size: 15,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let toml_part = if let Some(pos) = content.find(|c: char| c != '#' && c != ' ' && c != '\n' && c != '\r') {
                    &content[pos..]
                } else {
                    &content
                };
                if let Ok(config) = toml::from_str(toml_part) {
                    return config;
                }
            }
        }
        let mut config = Config::default();
        config.dark_mode = system_is_dark_mode();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let body = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(), format!("{}{}", HEADER, body)).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn path() -> PathBuf {
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("ra2-clicker"));
        path.set_extension("toml");
        path
    }
}

fn system_is_dark_mode() -> bool {
    let out = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output();
    match out {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout);
            !s.contains("0x1")
        }
        _ => true,
    }
}
