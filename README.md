# ra2-clicker

专为《命令与征服：红色警戒 2》设计的鼠标连点器，纯 Rust 实现。

## 设计目标

- **不需要弹出 UAC 认证** — 纯用户态全局钩子，无需管理员权限
- **深色模式适配** — 深色/浅色主题一键切换，Windows 标题栏同步跟随
- **占用不到 4MB 运行内存** — Slint 轻量软件渲染 + mimalloc 分配器

## 功能

- 连点开关、左键/右键独立控制
- 连点次数 0~30，点击间隔 1~200ms 可调
- 热键：Shift / Ctrl / Alt 三选一，支持关闭热键
- OL 风格：按住热键 + 鼠标点击触发
- RA2 模式：仅在建造栏区域连点，建造栏宽度可调
- 自动检测游戏进程（gamemd.exe 等），检测到后自动启动连点
- 全局键盘/鼠标低级钩子，游戏内外均可使用
- 配置文件热重载：点击"编辑配置文件"→ 保存后自动生效
- 退出时自动记录窗口位置（可在更多设置中关闭）

## 下载

从 [Releases](https://github.com/cmixed/ra2-clicker/releases) 下载 `ra2-clicker.exe`，建议单独创建文件夹存放（程序会在同目录生成配置文件 `ra2-clicker.toml`）。

## 构建

依赖 Rust 工具链，在项目目录下执行：

```bash
cargo build --release
```

输出 `target/release/ra2-clicker.exe`，单文件便携，无需任何运行时依赖。

## 配置

首次运行自动生成 `ra2-clicker.toml`（位于 exe 同目录），包含完整中文注释。在软件中点击"编辑配置文件"后修改并保存即可自动重载。

## 许可

MIT

## 作者

cmixed · cmixed@foxmail.com · [Gitee](https://gitee.com/cmixed/ra2-clicker) · [GitHub](https://github.com/cmixed/ra2-clicker)
