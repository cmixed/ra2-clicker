# ra2-clicker

专为《命令与征服：红色警戒 2》设计的鼠标连点器，纯 Rust 实现，借鉴 [ra2-mouse-click](https://github.com/cmixed/ra2-mouse-click)（C# WinForms）。

## 设计目标

- **不需要弹出 UAC 认证** — 纯用户态全局钩子，无需管理员权限
- **适配深色模式** — 深色/浅色主题一键切换，Windows 标题栏跟随
- **占用小运行内存 3.几 MB** — Slint 轻量软件渲染 + mimalloc 分配器

## 功能

- 连点开关、左键/右键连点
- 连点次数 0~30，点击间隔 1~200ms
- 热键：Shift / Ctrl / Alt 三选一，支持"无"热键
- OL 风格：按住热键 + 鼠标点击触发
- RA2 模式：仅在建造栏区域连点，建造栏宽度可调
- 自动检测游戏进程（gamemd.exe 等）并自动启停连点
- 全局键盘/鼠标钩子，游戏内外均可使用
- 深色/浅色主题一键切换
- 配置文件即时热重载（点击"编辑配置文件" → 保存后自动生效）
- 退出时自动记录窗口位置（可关闭）

## 下载

从 [Releases](https://github.com/cmixed/ra2-clicker/releases) 下载最新版本的 `ra2-clicker.exe`，直接运行即可。

## 构建

```bash
cargo build --release
```

输出 `target/release/ra2-clicker.exe`，单文件便携，无需任何运行时依赖。

## 配置

首次运行自动生成 `ra2-clicker.toml`（位于 exe 同目录），中文注释，编辑保存后自动热重载。

## 许可

MIT

## 作者

cmixed · cmixed@foxmail.com · [Gitee](https://gitee.com/cmixed/ra2-clicker) · [GitHub](https://github.com/cmixed/ra2-clicker)
