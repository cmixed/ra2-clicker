# ra2-clicker

**ra2-clicker** 是一款专为《命令与征服：红色警戒 2》设计的鼠标连点器。

本仓库是原 [ra2-mouse-click](https://github.com/cmixed/ra2-mouse-click)（C# WinForms）的 **Rust + egui 重写版**，感谢原作者的创意与设计。

## 与原版的区别

- ✅ 使用 **Rust** 重写，原生 Windows API，无 .NET 依赖
- ✅ **深色/浅色主题** 一键切换，跟随系统主题自动适配
- ✅ **TOML 配置文件**，带中文注释，热加载即时生效
- ✅ 现代化 UI 布局，可缩放字体，高 DPI 自适应
- ✅ 无边框窗口（系统标题栏），更简洁的外观

## 功能

- 连点开关、左键/右键连点
- 连点次数 (0~30)，点击间隔 (1~200ms)
- 热键支持：Shift / Ctrl / Alt
- OL 风格：按住热键 + 鼠标点击触发
- RA2 模式：仅在建造栏区域（屏幕右端）连点
- 自动检测游戏进程（gamemd.exe 等）
- 全局热键钩子，游戏内外均可使用

## 构建

```bash
cargo build --release
```

## 配置

首次运行后自动生成 `ra2-clicker.toml`，所有配置项均带有中文注释。

## 许可证

MIT
