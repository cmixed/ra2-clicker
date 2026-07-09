use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HEADER: &str = r#"# ra2-clicker 配置文件
# 编辑后保存即生效，无需重启
#
# ── 连点设置 ──
# click_on            = 连点开关 (true=开, false=关)
# click_counts        = 每次连点次数 (0~30)
# click_interval_ms   = 点击间隔，毫秒 (1~200)
# left_click          = 左键连点 (true=启用)
# right_click         = 右键连点 (true=启用)
# use_ra2_ol_style    = OL 风格: 按住热键+鼠标点击才触发 (true=启用)
#
# ── 热键 ──
# hotkey              = 触发键: "shift" / "ctrl" / "alt"
#
# ── RA2 模式 ──
# enable_ra2_mode     = 仅在建造栏区域连点 (true=启用)
# construction_bar_width = 建造栏宽度，像素 (50~500)
# screen_width        = 屏幕宽度，0=自动检测
#
# ── 自动检测进程 ──
# auto_detect_mode    = 自动检测游戏并开关连点 (true=启用)
# auto_detect_interval_ms = 检测间隔，毫秒
# game_process_list   = 待检测的进程名列表
#
# ── 外观 ──
# dark_mode           = 深色模式 (true=深色, false=浅色)
# font_size           = 字体大小 (10~24)
# window_pos_x        = 窗口水平位置 (像素)，留空=居中
# window_pos_y        = 窗口垂直位置 (像素)，留空=居中
# remember_position   = 退出时保存窗口位置 (true=启用)

"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ── 连点设置 ──
    pub click_on: bool,
    pub click_counts: u32,
    pub click_interval_ms: u32,
    pub use_ra2_ol_style: bool,
    pub left_click: bool,
    pub right_click: bool,

    // ── 热键 (对外为字符串，对内为虚拟键码) ──
    #[serde(
        default = "default_hotkey",
        alias = "hotkey_code",
        deserialize_with = "deser_hotkey"
    )]
    pub hotkey: String,
    #[serde(skip)]
    pub hotkey_code: u32,

    // ── RA2 模式 ──
    pub enable_ra2_mode: bool,
    pub construction_bar_width: u32,
    pub screen_width: u32,

    // ── 自动检测进程 ──
    pub auto_detect_mode: bool,
    pub auto_detect_interval_ms: u32,
    pub game_process_list: Vec<String>,

    // ── 外观 ──
    pub dark_mode: bool,
    pub font_size: u32,
    pub window_pos_x: u32,
    pub window_pos_y: u32,
    pub remember_position: bool,
}

fn default_hotkey() -> String {
    "shift".into()
}

const HOTKEY_SHIFT: u32 = 0xA0;
const HOTKEY_CTRL: u32 = 0xA2;
const HOTKEY_ALT: u32 = 0xA4;

impl Default for Config {
    fn default() -> Self {
        let mut s = Self {
            click_on: false,
            click_counts: 5,
            click_interval_ms: 10,
            use_ra2_ol_style: true,
            left_click: true,
            right_click: true,
            hotkey: "shift".into(),
            hotkey_code: 0,
            enable_ra2_mode: true,
            construction_bar_width: 160,
            screen_width: 0,
            auto_detect_mode: true,
            auto_detect_interval_ms: 3000,
            game_process_list: vec![
                "gamemd".into(),
                "game".into(),
                "gameares".into(),
                "gamemd-spawn".into(),
                "gamemd-ares".into(),
            ],
            dark_mode: true,
            font_size: 15,
            window_pos_x: u32::MAX,
            window_pos_y: u32::MAX,
            remember_position: true,
        };
        s.sync_from_hotkey();
        s
    }
}

impl Config {
    pub fn sync_from_hotkey(&mut self) {
        self.hotkey_code = match self.hotkey.as_str() {
            "none" => 0,
            "shift" => HOTKEY_SHIFT,
            "ctrl" => HOTKEY_CTRL,
            "alt" => HOTKEY_ALT,
            _ => HOTKEY_SHIFT,
        };
    }

    pub fn sync_from_code(&mut self) {
        self.hotkey = match self.hotkey_code {
            0 => "none",
            HOTKEY_SHIFT => "shift",
            HOTKEY_CTRL => "ctrl",
            HOTKEY_ALT => "alt",
            _ => "shift",
        }
        .into();
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let content = content.trim_start_matches('\u{FEFF}');
                let toml_part: String = content.lines()
                    .skip_while(|line| {
                        let t = line.trim();
                        t.is_empty() || t.starts_with('#')
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Ok(mut config) = toml::from_str::<Self>(&toml_part) {
                    config.sync_from_hotkey();
                    return config;
                }
            }
        }
        let config = Config::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let mut s = self.clone();
        s.sync_from_code();
        let body = toml::to_string_pretty(&s).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(), format!("{}{}", HEADER, body))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn path() -> PathBuf {
        let mut p = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("ra2-clicker"));
        p.set_extension("toml");
        p
    }
}



mod hotkey_deser {
    use serde::de;

    pub fn deser<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct V;
        impl<'de> de::Visitor<'de> for V {
            type Value = String;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(r#""shift", "ctrl", "alt", 160, 162, or 164"#)
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
                Ok(v.to_lowercase())
            }

                    fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
                        Ok(match v {
                            0 => "none",
                            160 => "shift",
                            162 => "ctrl",
                            164 => "alt",
                            _ => return Err(de::Error::custom("invalid hotkey code")),
                        }
                        .into())
                    }
        }
        deserializer.deserialize_any(V)
    }
}
use hotkey_deser::deser as deser_hotkey;
