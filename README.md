# ra2-clicker v1.4.1

一款专为《命令与征服：红色警戒 2》设计的鼠标连点器。

本仓库借鉴 [ra2-mouse-click](https://github.com/cmixed/ra2-mouse-click)（C# WinForms）的 **Rust + egui** 重写版，感谢原作创意。

## 特性

- 原生 Windows API，纯 Rust，无 .NET / .NET Framework 依赖
- 深色/浅色主题一键切换，首次运行自动跟随系统
- 可调字体大小 (10~24)，DPI 自适应
- 可配置窗口位置百分比 (0~100)，支持记住拖拽位置
- TOML 配置文件，中文注释，改动即时保存
- ℹ 关于窗口（作者 / 邮箱 / 仓库）

## 功能

- 滑动开关：连点开关、自动检测进程、左键/右键连点
- 连点次数 0~30，点击间隔 1~200ms
- 热键：Shift / Ctrl / Alt 三选一
- OL 风格：按住热键 + 鼠标点击触发
- RA2 模式：仅在建造栏区域连点，建造栏宽度可调
- 自动检测游戏进程（gamemd.exe 等）并自动启停连点
- 全局热键钩子，游戏内外均可使用

## 构建

```bash
cargo build --release
```

输出 `target/release/ra2-clicker.exe`。

## 配置

首次运行自动生成 `ra2-clicker.toml`，包含完成的中文注释。编辑后即时生效。

## 许可

MIT

## 鸣谢

- 原 [ra2-mouse-click](https://github.com/cmixed/ra2-mouse-click) 作者
- [DeepSeek](https://chat.deepseek.com/) 辅助开发

## 作者

cmixed · cmixed@foxmail.com · [GitHub](https://github.com/cmixed/ra2-clicker)
