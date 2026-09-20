# JobFlow · 求职简历投递与面试阶段统计系统

一个用 **Rust + GPUI** 编写的桌面应用，用来管理求职投递记录、跟踪面试阶段流转，并实时统计漏斗转化率、面试率、Offer 率、月度趋势与投递渠道效果。

> 界面语言为中文，数据保存在本地 JSON 文件中，不依赖任何在线服务。

---

## 功能特性

### 1. 投递管理

- 记录公司、岗位、投递渠道、城市、薪资范围、投递日期、备注、下一步动作与日期。
- **投递日期 / 下一步日期用日历选择**（组件库 `DatePicker`，中文月份与星期）；
  投递日期只能选到今天及以前，下一步日期可以留空、也可以选未来。
- 阶段流转：已投递 → 简历筛选 → 笔试/测评 → 一面 → 二面 → 三面/终面 → HR 面 → Offer。
- 支持终止状态：已拒绝、已放弃；终止后仍保留此前到达过的阶段历史。
- 每次阶段变更都会写入历史事件，用于精确计算“到达过哪些阶段”和周期天数。
- **阶段变更可以指定日期**：详情面板「阶段流转」里有一个「阶段日期」日历（默认今天），
  推进或切换阶段时按这一天写入历史，方便补录“面试其实是上周三”的记录；日期不是今天时旁边会高亮提醒。
- **阶段历史支持逐条改日期**：历史里每条非投递事件右侧都有「改日期」按钮，
  弹窗里用日历改到正确的日子（可选范围是投递日期 ~ 今天，且不会把历史顺序弄乱）。
- 阶段日期有三条约束：不能早于投递日期、不能早于上一条事件（新事件）、不能晚于今天，
  违反时会直接提示并拒绝写入，避免出现“面试日期早于投递日期”这类负数周期。
- 新增记录时如果直接选了后面的阶段，阶段事件按**投递日期**写入（补录不会把周期算成 0 天）。
- 误点阶段可以**撤销上次阶段变更**（详情面板按钮），撤销后当前阶段回到上一条事件、`Offer 率`等口径同步回落。
- 删除记录需要点击两次确认，删除后自动选中相邻记录。
- 标签分类：支持自定义标签，也可以从已有标签中选择添加；支持按标签筛选。
- 支持按公司/岗位/渠道/城市/备注/下一步动作/标签搜索，按阶段和标签筛选；筛选变化时会自动校正选中项，不会出现“详情面板里是被筛掉的记录”。
- 支持新增、编辑、删除、推进阶段、手动切换阶段、清空数据、重置演示数据。

### 2. 仪表盘

- KPI 卡片：总投递、进行中、已拿 Offer、已拒绝/放弃。
- 阶段漏斗：每个阶段的到达投递数、到达率、环节留存率、环节流失率。
- 近期待办：未来 14 天内需要跟进的动作，过期项标红。
- 最近动态：最新的阶段变更记录。
- 近 6 个月投递趋势柱状图。

### 3. 阶段统计

- 有回复率、面试率、Offer 率。
- 平均面试等待天数（投递 → 第一次面试）。
- 平均 Offer 周期（投递 → 拿到 Offer）。
- 逐级转化漏斗与环节流失分析。
- 近 6 个月投递/面试/Offer 趋势。
- 投递渠道效果对比：各渠道投递数、面试数、Offer 数、Offer 率。
- 当前阶段分布。
- 标签分布：所有投递记录使用的标签及出现次数。

### 4. 数据持久化

- 默认路径：
  - Windows：`%APPDATA%\job-tracker\applications.json`（`APPDATA` 缺失时回退到 `%LOCALAPPDATA%`）
  - Linux：`~/.local/share/job-tracker/applications.json`
  - macOS：`~/Library/Application Support/job-tracker/applications.json`
- 也可以用环境变量 `JOB_TRACKER_DATA` 指定文件路径。
- 首次运行会自动写入 18 条演示数据，便于直接查看统计效果。
- 保存采用“先写临时文件再替换”的方式；临时文件名带进程号与序号，多个实例同时保存不会互相截断。
- **坏数据不会被静默丢掉**：
  - 数据文件整体解析失败时，先把原始内容原样备份成 `applications.bad-<时间戳>.json`，再逐条恢复还能解析的记录，
    并在界面（toast + 设置页「数据体检」）与 `--report` 输出里说明跳过了几条坏记录；
  - 这个场景下 `--report` 不会提示“运行 --seed”，图形界面也不会自动写入演示数据，避免覆盖还能人工修复的原始文件。
- **覆盖性操作都有兜底**：重置演示数据 / 清空数据会先把当前数据备份成 `applications.reset-<时间戳>.json` /
  `applications.clear-<时间戳>.json`；命令行 `--seed` / `--clear` 在已有数据时必须显式加 `--force`，加 `--force` 时同样会先自动备份。
- **数据兼容旧版本**：数据格式为 `version 3`。加载旧文件时会自动迁移：
  - 缺少 `tags` 字段 → 补为空数组；
  - 缺少 `history` → 按投递日期补一条“创建投递记录”事件（否则漏斗第一行会凭空流失）；
  - 标签自动去首尾空白、限制 24 字符、按“忽略大小写”去重。

### 5. 设置

- 数据管理：查看数据文件路径、投递记录数、标签数与文件大小；
- **重置为演示数据** 和 **清空所有数据** 两个操作集中在设置页，都需要点击两次确认，并会先自动备份现有数据；
- **数据体检**：展示本次启动读取数据时发现的问题（文件损坏、跳过的坏记录、备份文件路径）；
- 数据体检结果、构建信息与版本号（全局快捷键见下方[「快捷键」](#快捷键)一节）。

---

## 界面截图

仓库的 `docs/screenshots/` 目录保存了实际运行截图（Linux + WSLg，1280×820）：

| 页面 | 截图 |
| --- | --- |
| 仪表盘 | [dashboard.png](docs/screenshots/dashboard.png) |
| 投递管理 | [applications.png](docs/screenshots/applications.png) |
| 阶段统计 | [analytics.png](docs/screenshots/analytics.png) |
| 新增投递（gpui-component 对话框） | [form.png](docs/screenshots/form.png) |
| 投递日期日历选择 | [form-date.png](docs/screenshots/form-date.png) |
| 编辑投递（含标签与阶段选择） | [form-tags.png](docs/screenshots/form-tags.png) |
| 设置（数据管理、数据体检） | [settings.png](docs/screenshots/settings.png) |

> 四个页面截图取自 v0.3.2（Linux + WSLg，1280×820），使用 gpui-component 组件库渲染；
> 表单相关的三张取自 v0.3.0（对话框自 v0.3.0 起没有改动）。
> release 构建（`opt-level=2 + thin LTO + strip`）的 Linux 二进制约 34 MB（gpui-kit 0.6 比旧栈大一些），压缩后的 tar.gz 约 10 MB。

---

## 技术栈

| 组件 | 说明 |
| --- | --- |
| Rust 2024 | `edition = "2024"`，需要 Rust 1.85+ |
| gpui-kit 0.6.1 | longbridge 的 GPUI 全家桶单一入口：框架 + 平台后端 + 基础层 + 组件 + 资源 |
| gpui-pre 0.3.4 | 框架层：Zed 的 gpui 快照（zed@6916400），替代早期的 `gpui 0.2.2` |
| gpui-component 0.6.1 | 组件库（60+ 桌面组件）：按钮、输入框、日历、对话框、通知、标签、进度条、图标…… |
| Windows 后端 | GPUI 内置 DirectX 12 渲染 + DirectWrite 文本 |
| Linux 后端 | Vulkan 渲染 + X11 / Wayland 窗口 |
| macOS 后端 | Metal 渲染 + CoreText 文本 |
| serde / serde_json | 数据序列化与 JSON 持久化 |
| chrono | 日期与月份计算 |
| uuid | 投递记录唯一 ID |
| unicode-segmentation | 文本输入框的光标/选区按字素处理 |

---

## 目录结构

```text
job-tracker/
├── Cargo.toml
├── README.md
├── LICENSE
├── .github/workflows/
│   ├── ci.yml               # Windows + Linux 持续集成
│   └── release.yml          # 打 tag 后自动发布
├── packaging/
│   ├── README.md            # 打包与发布详细指南
│   ├── linux/job-tracker.desktop
│   ├── macos/Info.plist     # macOS .app 的 Info.plist 模板
│   └── windows/job-tracker.iss
├── scripts/
│   ├── package-linux.sh     # Linux tar.gz / .deb
│   ├── package-macos.sh     # macOS .app / zip / dmg
│   └── package-windows.ps1  # Windows 便携 ZIP / 安装程序
└── src/
    ├── main.rs              # 入口：CLI 参数、GPUI 启动、文本报告、快捷键绑定
    ├── model.rs             # 阶段枚举、投递记录、阶段事件、Store
    ├── stats.rs             # 漏斗/转化率/月度趋势/渠道统计
    ├── storage.rs           # JSON 读写与数据文件路径
    ├── demo.rs              # 演示数据
    └── ui/
        ├── mod.rs
        ├── app.rs           # RootView：侧边栏、顶部栏、对话框表单、事件处理
        │                    #   以及负责渲染浮层的 AppShell
        ├── dashboard.rs     # 仪表盘页面
        ├── applications.rs  # 投递管理页面
        ├── analytics.rs     # 阶段统计页面
        ├── settings.rs      # 设置页面（数据管理、数据体检）
        ├── components.rs    # 卡片、徽章、进度条、按钮等（基于 gpui-component 封装）
        └── theme.rs         # 颜色与阶段配色（同时把配色灌进组件库主题）
```

---

## 运行方式

### 图形界面

```bash
cargo run
```

### 文本报告（适合无图形环境 / SSH）

```bash
cargo run -- --report
```

### 写入演示数据

```bash
cargo run -- --seed
```

### 清空所有数据

```bash
cargo run -- --clear
```

这会写入一个空的 JSON 数据文件，下次启动显示空数据。也可以使用界面设置页的 **“清空所有数据”** 按钮（需要点击两次确认）。

> 两个命令在数据文件里已有记录时都会**拒绝执行**并返回退出码 1，避免脚本或手滑覆盖真实数据：

```bash
cargo run -- --seed  --force   # 先自动备份成 applications.backup-<时间戳>.json，再写入演示数据
cargo run -- --clear --force   # 先自动备份，再写入空数据文件
```

如果想手动删除数据文件：

- Linux：`rm ~/.local/share/job-tracker/applications.json`
- Windows：`Remove-Item "$env:APPDATA\job-tracker\applications.json"`
- macOS：`rm "~/Library/Application Support/job-tracker/applications.json"`

> 注意：直接删除文件后，下次启动会因为“首次运行”而重新写入演示数据；如果希望保持空数据，请使用 `--clear`。

### 查看帮助 / 版本

```bash
cargo run -- --help
cargo run -- --version
```

### 指定数据文件

```bash
JOB_TRACKER_DATA=/tmp/my-applications.json cargo run
```

### 无头编译 / 测试

如果当前机器没有 X11/Wayland/Vulkan 开发与运行环境，可以关闭默认的 `gui` feature，只编译核心逻辑与无头 GPUI：

```bash
cargo check --no-default-features
cargo test  --no-default-features
cargo run   --no-default-features -- --report
```

---

## 打包与发布

详细步骤见 [packaging/README.md](packaging/README.md)。快速命令：

Windows（PowerShell）：

```powershell
.\scripts\package-windows.ps1              # release 构建 + 便携 ZIP
.\scripts\package-windows.ps1 -Installer   # 额外生成 Inno Setup 安装程序
.\scripts\package-windows.ps1 -WindowsGui  # 连 debug 构建也隐藏控制台（release 本来就隐藏）
```

Linux：

```bash
./scripts/package-linux.sh          # release 构建 + tar.gz
./scripts/package-linux.sh --deb    # 额外生成 .deb
```

macOS：

```bash
xcode-select --install               # 首次需要
./scripts/package-macos.sh           # JobFlow.app + zip (+ dmg)
./scripts/package-macos.sh --no-dmg  # 只生成 .app 和 zip
./scripts/package-macos.sh --sign "Developer ID Application: Your Name (TEAMID)"
```

推送形如 `v0.2.0` 的 tag 后，`.github/workflows/release.yml` 会在 Windows、Linux 和 macOS 上自动构建，并把 ZIP / tar.gz / .deb / dmg / SHA256 上传到 GitHub Release：

```bash
git tag v0.2.0
git push origin v0.2.0
```

---

## Windows 运行与构建

### 运行

在 Windows 10/11 上安装 Rust MSVC 工具链（`rustup default stable-msvc`）和 **Visual Studio Build Tools**（勾选 “使用 C++ 的桌面开发”，包含 MSVC 编译器与 Windows SDK），然后：

```powershell
cargo run
```

数据默认保存在 `%APPDATA%\job-tracker\applications.json`。

### 构建依赖说明

- GPUI 的 Windows 后端使用 DirectX / DirectWrite，不需要 Vulkan、X11 或 Wayland。
- **release 构建默认使用 GUI 子系统**：双击运行时不会弹出黑色控制台窗口。
  从终端执行 `job-tracker.exe --report` / `--seed` 时，程序会自己附加回父控制台，
  输出照常显示在那个终端里；输出被重定向到文件（`> report.txt`）时也保持重定向。
- debug 构建（`cargo run`）保留控制台，方便开发时看日志；如果也想隐藏，
  加上 `windows-gui` feature：

  ```powershell
  cargo build --features windows-gui
  ```
- 默认的 `gui` feature 包含 `gpui/windows-manifest`，用于嵌入 DPI 感知清单；它需要 Windows SDK 中的 `rc.exe`（通常随 Visual Studio Build Tools 安装）。
- **release 构建**还会编译 HLSL 着色器，需要 Windows SDK 中的 `fxc.exe`；如果没装，可以先用 debug 构建（`cargo run`），或设置 `GPUI_FXC_PATH` 指向 `fxc.exe`。
- 如果 `rc.exe` 不可用，可以临时用无清单模式运行：

```powershell
cargo run --no-default-features
```

  在 Windows 上 GPUI 的 Windows 平台代码仍然会被编译，只是不嵌入 manifest；图形界面可以正常使用。

### 文本报告

Windows 上同样可以只用命令行：

```powershell
cargo run -- --report
cargo run -- --seed
```

建议使用 Windows Terminal 或 PowerShell 7 以获得正确的中文显示。

界面中的中文由 DirectWrite 自动回退到系统 CJK 字体；如果个别字符显示异常，请确认系统已安装微软雅黑（Microsoft YaHei）或中文语言包。

---

## Linux 图形依赖

GPUI 在 Linux 上使用 Vulkan 渲染，并通过 X11 或 Wayland 创建窗口。运行时需要：

- X11（`DISPLAY`）或 Wayland（`WAYLAND_DISPLAY`）会话；
- Vulkan 驱动（真实显卡驱动，或 Mesa 的软件光栅化驱动 lavapipe）；
- 字体配置：fontconfig + 至少一套包含中文字形的字体（例如 Noto Sans CJK）；
- 编译期需要 `libxkbcommon` / `libxkbcommon-x11` 的 `.so` 链接符号；
- `blade-graphics` 通过 `ash` 链接 `libvulkan`。

### Fedora

```bash
sudo dnf install -y gcc-c++ pkgconf-pkg-config \
  libxkbcommon-devel libxkbcommon-x11-devel \
  vulkan-loader-devel vulkan-loader mesa-vulkan-drivers \
  fontconfig freetype fontconfig-devel freetype-devel google-noto-sans-cjk-fonts \
  libxcb libxcb-devel
```

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
  libxkbcommon-dev libxkbcommon-x11-dev \
  libvulkan-dev vulkan-tools mesa-vulkan-drivers \
  libfontconfig1 libfreetype6 libfontconfig1-dev libfreetype6-dev fonts-noto-cjk \
  libxcb1 libxcb1-dev
```

如果 `cargo run` 启动时报 “no Vulkan device” 或窗口创建失败，可以安装 `vulkan-tools` 后运行 `vulkaninfo --summary` 检查驱动；没有独显时确保已安装 `mesa-vulkan-drivers`（lavapipe 软件渲染）。

### WSLg 提示

WSLg 自带 Wayland 与 XWayland，但发行版镜像里通常没有 Vulkan 驱动。此时可以安装 Mesa lavapipe：

```bash
sudo dnf install -y vulkan-loader mesa-vulkan-drivers
# 或者 Ubuntu
sudo apt install -y libvulkan1 mesa-vulkan-drivers
```

如果 WSLg 的 X11 socket 不在 `/tmp/.X11-unix`，可以建立软链接：

```bash
mkdir -p /tmp/.X11-unix
ln -sf /mnt/wslg/.X11-unix/X0 /tmp/.X11-unix/X0
```

若系统中文字体缺失，安装 `google-noto-sans-cjk-fonts`（Fedora）或 `fonts-noto-cjk`（Ubuntu）。

### WSLg 上必须走 X11 后端

**WSLg 的 Weston 只提供 `xdg_wm_base` v1**，而 GPUI（gpui-pre）的 Wayland 后端要求 v2..=v5，
所以在 WSLg 里直接 `cargo run` 会 panic：

```text
panicked at gpui-.../wayland/client.rs:151:
called `Result::unwrap()` on an `Err` value: UnsupportedVersion
```

把 `WAYLAND_DISPLAY` 置空即可让 GPUI 退回 X11 后端（XWayland 工作正常）：

```bash
WAYLAND_DISPLAY= cargo run
```

### 无 root（最小化 Fedora / WSL）的本地开发环境

如果这台机器没有 sudo、又缺 `libxcb` / `libxkbcommon-x11` / Vulkan / fontconfig，
可以像本仓库当前的工作副本那样用一个**本地原生库前缀**（`/.native`，已在 `.gitignore` 里）：

1. 用 `dnf download` 拉 RPM（不需要 root），再用 `rpm2archive -n <rpm> | tar -xf - -C .native/root` 解包，
   需要的包大致是：`libxcb libXau libXdmcp libxkbcommon libxkbcommon-x11 libX11 libX11-xcb libX11-common
   vulkan-loader mesa-vulkan-drivers llvm-libs libpng harfbuzz graphite2 fontconfig freetype
   libdrm libxshmfence libwayland-client libdisplay-info spirv-tools-libs dejavu-sans-fonts xwininfo xwd
   fontconfig-devel freetype-devel`（后两个是为了拿到 `fontconfig.pc`：0.6 的依赖链里
   `yeslogic-fontconfig-sys` 会在构建时用 pkg-config 查它）；
2. 在 `.native/root/usr/lib64` 里给 `-l` 用的库名建软链（`libxcb.so -> libxcb.so.1` 等）；
3. 把 `mesa` 的 ICD 清单里的 `library_path` 改成前缀内的绝对路径；
4. 由于本机没有 `/etc/fonts/fonts.conf`，自带一份 `fonts.conf` 指向 `/usr/share/fonts` 与前缀里的 DejaVu 字体；
5. 把上面这些写进本地的 `.cargo/config.toml`（同样已被 `.gitignore` 忽略）：

```toml
[build]
rustflags = ["-L", "<repo>/.native/root/usr/lib64",
             "-C", "link-arg=-Wl,-rpath,<repo>/.native/root/usr/lib64"]

[env]  # 桌面会话已有同名变量，所以要 force = true
LD_LIBRARY_PATH      = { value = ".native/root/usr/lib64", relative = true, force = true }
VK_ICD_FILENAMES     = { value = ".native/root/usr/share/vulkan/icd.d/lvp_icd.x86_64.json", relative = true, force = true }
FONTCONFIG_FILE      = { value = ".native/root/etc-fonts.conf", relative = true, force = true }
# yeslogic-fontconfig-sys 构建时要 pkg-config 找 fontconfig（前缀里放一份精简 .pc）
PKG_CONFIG_PATH      = { value = ".native/root/usr/lib64/pkgconfig", relative = true, force = true }
XLOCALEDIR           = { value = ".native/root/usr/share/X11/locale", relative = true, force = true }
MESA_SHADER_CACHE_DIR = { value = "/tmp/mesa-cache", force = true }
WAYLAND_DISPLAY      = { value = "", force = true }   # 见上一节
```

这样 `cargo run` 就能直接开窗口（软件渲染；若 `/dev/dxg` 可用，把 ICD 换成 `dzn_icd.x86_64.json` 即可走 GPU）。
有 sudo 的机器上按上面的 `dnf install` 装系统包即可，不需要 `.native`，删掉 `.cargo/config.toml` 也不影响。

---

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| `Ctrl + N`（macOS：`Cmd + N`） | 新增投递 |
| `Ctrl + S`（macOS：`Cmd + S`） | 保存表单 |
| `Ctrl + F`（macOS：`Cmd + F`） | 聚焦搜索框 |
| `Esc` | 关闭表单 |
| `Ctrl + Q`（macOS：`Cmd + Q`） | 退出 |
| `Ctrl + A`（macOS：`Cmd + A`） | 文本输入框全选 |
| `Ctrl + C / X / V`（macOS：`Cmd + ...`） | 文本输入框复制/剪切/粘贴 |

> 快捷键使用 GPUI 的 `secondary` 修饰键：Windows/Linux 上为 Ctrl，macOS 上为 Command。
> 文本输入框内部的编辑快捷键（全选/复制/剪切/粘贴/撤销/按词移动等）由 gpui-component 的
> `Input` 组件自己处理，应用只注册上面这些全局快捷键。

---

## UI 组件库（gpui-kit / gpui-component）

界面基于 [gpui-kit](https://github.com/longbridge/gpui-kit) 0.6.1 构建。0.6 起 zed 的 gpui
以 `gpui-pre` 快照发布，longbridge 把它们收进一个入口 crate：

| 依赖 | 作用 |
| --- | --- |
| `gpui-kit` | 唯一入口：`gpui_kit::*` 就是 GPUI 框架，另有 `::platform` / `::base` / `::component` / `::assets` 子层 |
| `gpui`（`package = "gpui-pre"`） | 与 kit 内部同一个 crate，显式列出只为在 Windows 上嵌入 DPI 清单（kit 未暴露该 feature） |

| 界面元素 | 使用的组件 |
| --- | --- |
| 新增/编辑投递表单 | `AlertDialog`（遮罩、ESC 关闭、焦点管理、底部按钮）+ `Input` |
| 投递日期 / 下一步日期 | `DatePicker`（中文日历，投递日期禁选未来） |
| 备注 | `Textarea`（多行、自动增高） |
| 顶部搜索框 | `Input`（前置搜索图标、一键清空） |
| 所有按钮 | `Button`（primary / danger / ghost / 尺寸变体） |
| 阶段与标签 | `Tag` + `Button` |
| 记录列表 | `ListItem` |
| 漏斗与趋势条 | `Progress` |
| 操作反馈 | `Notification`（右上角浮层、自动消失） |
| 数据体检提示 | `Alert` |
| 图标 | `Icon` + `IconName`（Lucide，`gpui_kit::assets`） |

集成时踩到的几个点，已经在代码里处理：

1. **依赖只留一个入口**：`gpui-kit = { version = "0.6.1" }`（默认带 `component` + `assets`），
   平台后端（X11/Wayland/macOS/Windows）由 kit 内部的 `gpui-pre-platform` 统一打开，
   不再需要手写 `gpui/wayland`、`gpui/x11` 之类的 feature。整个 UI 栈挂在 `gui` feature 上，
   `--no-default-features` 的无头构建不会把它拖进来。
2. **启动方式变了**：`Application::new()` 已不存在，改用 `gpui_kit::application()`
   （带平台后端的 Application），初始化用 `gpui_kit::init(cx)`。
3. **图标资源**：需要 `gpui_kit::application().with_assets(gpui_kit::assets::Assets)`，
   否则 `Icon` 什么都不显示。
4. **浮层要自己渲染**：`Root` 只渲染内嵌视图，`Dialog`/`Sheet`/`Notification`
   需要应用在根视图外层调用 `Root::render_*_layer(window, cx)`；而表单内容又要读 `RootView`，
   直接在 `RootView::render` 里渲染会「自己读自己」而 panic，因此多包了一层 `AppShell`。
5. **带按钮的对话框是 `AlertDialog`**：0.6 的 `Dialog` 只渲染调用方给的 footer
   （`AlertDialog` 才有默认的确定/取消）。本项目用 `AlertDialog` + 自定义 footer，
   两个按钮直接绑回调，不依赖框架的动作分发。
6. **主题是「三层」的**：`colors`（原始调色板）→ `tokens`（组件级解析结果）→
   Base 层语义 token。0.6 的按钮读 `colors.button_primary` 这类**组件级字段**而不是
   `colors.primary`，而且改完 `colors` 必须手动重建 `tokens` 并调用 `Theme::sync_base(cx)`，
   否则组件仍用默认主题（近黑色主色）。这些都在 `theme::install_component_theme` 里处理了。

## 统计口径

所有指标都按**投递记录条数**统计：同一个候选人（你）投出去的每一条记录算一条，同一家公司多次投递算多条，不按人数或公司去重。

- **有回复率** = 到达“简历筛选”及之后阶段的投递数 / 总投递数。
- **面试率** = 到达“一面”及之后阶段的投递数 / 总投递数。
- **Offer 率** = **曾**拿到过 Offer 的投递数 / 总投递数（拿到后又放弃的记录仍计入）。
- **到达投递数**：历史事件中出现过该阶段，或当前阶段在该阶段及之后，即算“到达”。每个阶段统计的是**投递记录条数**，不是人数。
- **环节留存** = 本阶段到达投递数 / 上一阶段到达投递数。
- **环节流失** = 1 - 环节留存。
- **平均面试等待** = 所有“第一次进入面试”事件的（面试日期 - 投递日期）平均值，只统计已经面试过的记录。
- **平均 Offer 周期** = 所有“第一次拿到 Offer”事件的（Offer 日期 - 投递日期）平均值，只统计已经拿到 Offer 的记录。
- **日期倒挂的记录**（阶段事件早于投递日期）不参与上面两个平均值，只在界面和 `--report` 里单独提示，不会被悄悄按 0 天计入。
- **未来日期**：投递日期晚于今天的记录不计入“最近 30 天”，也不进入月度趋势（表单本身会拒绝未来日期），`--report` 会单列条数。
- **月度趋势**：按投递日期统计每月新增投递数；面试/Offer 按对应事件日期统计。
- **渠道效果**：按投递渠道分组统计投递数、面试数、Offer 数与 Offer 率；分组时忽略首尾空白与大小写，空白渠道与手填“未填写”合并。
- **性能**：统计数据在数据变化后重算一次并缓存，界面每帧复用，不再逐帧重复计算漏斗与趋势。

---

## 数据文件示例

```json
{
  "version": 3,
  "applications": [
    {
      "id": "0f1e2d3c-...",
      "company": "星海科技",
      "position": "Rust 后端工程师",
      "channel": "BOSS直聘",
      "location": "上海",
      "salary": "30-45K",
      "applied_at": "2026-06-05",
      "stage": "Offer",
      "updated_at": "2026-07-06",
      "next_action": "确认 Offer 薪资与入职时间",
      "next_action_at": "2026-09-11",
      "notes": "两轮技术面反馈很好。",
      "tags": ["Rust", "远程", "目标公司"],
      "history": [
        { "stage": "Applied", "at": "2026-06-05", "note": "创建投递记录" },
        { "stage": "ResumeScreening", "at": "2026-06-07", "note": "简历筛选：星海科技" },
        { "stage": "Interview1", "at": "2026-06-17", "note": "一面：星海科技" },
        { "stage": "Offer", "at": "2026-07-06", "note": "Offer：星海科技" }
      ]
    }
  ]
}
```

---

## 测试

```bash
cargo test --no-default-features
```

## 持续集成

`.github/workflows/ci.yml` 会在以下平台执行 `cargo check --all-targets`、`cargo test --all-targets` 和 debug 构建：

- **Windows (MSVC)**：验证 DirectX / DirectWrite 后端与 `windows-manifest` 构建；
- **Linux (X11/Wayland)**：安装 `libxkbcommon`、`libvulkan`、fontconfig 等依赖后验证图形后端，并额外执行一次无头文本报告 smoke test；
- **macOS (Metal)**：验证 Metal / CoreText 后端，并执行一次 `package-macos.sh` 打包 smoke test。

当前包含 27 个单元测试：

**统计口径**

- 总览指标计数正确；
- 漏斗到达投递数单调不增、`previous_reached` 逐级衔接；
- 月度趋势固定返回 6 个月，且跨年分桶正确（12 月 / 1 月边界）；
- 平均面试等待天数计算正确；
- 日期倒挂的记录被排除在平均值之外、同时被计数上报；
- 未来投递日期不计入最近 30 天，且"月度合计 + 未来条数 = 总数"；
- 渠道分组忽略大小写与首尾空白，空白渠道与"未填写"合并；
- 空数据下不 panic、比率与漏斗都退化为 0；
- 演示数据一致性（含标签非空）。

**数据模型与持久化**

- 存储读写往返、空数据文件往返（`--clear` 场景）、缺失文件加载为空数据且不报损坏；
- 旧版本数据（没有 `tags` / `history`）自动迁移到 version 3，history 按投递日期补齐；
- 损坏文件先备份再逐条恢复（好记录保留、坏记录计数、备份内容与原文件逐字节一致）；
- 完全无法解析的文件不会被静默覆盖；
- 四个线程并发保存同一路径不产生损坏文件、也不残留临时文件；
- 保存时自动创建多级父目录。

**模型与交互逻辑**

- 标签收集与计数（忽略大小写，含 `Ä`/`ä` 这类非 ASCII 大小写）；
- 标签规范化（去空白、24 字符上限、迁移时去重）；
- 按标签筛选与标签列表口径一致（列表里能点到的标签一定能筛出对应记录）；
- 搜索覆盖备注与"下一步动作"；
- 撤销最后一次阶段变更可以回退当前阶段与 `Offer` 口径，且不会把记录退回"没有历史"的状态。

---

## Git 工作流

项目已经初始化为 Git 仓库，默认分支 `main`，发布时使用形如 `v0.2.0` 的 tag（当前版本 `0.2.0`，见 [CHANGELOG.md](CHANGELOG.md)）。

常用命令：

```bash
git status
git add .
git commit -m "feat: ..."
git log --oneline --decorate
```

如果还没有远程仓库：

```bash
git remote add origin <your-repo-url>
git push -u origin main
git push origin v0.2.0
```

推送形如 `v0.2.0` 的 tag 会触发 `.github/workflows/release.yml`，自动在 Windows/Linux 上构建并创建 GitHub Release：

```bash
git tag v0.2.0
git push origin main --tags
```

---

## 已知限制与后续可扩展方向

- 备注目前是单行输入框；如需长文本可以改为多行编辑器。
- 暂不支持 CSV / Excel 导入导出，可以基于 `Store` 的 JSON 结构扩展。
- 暂不支持提醒通知；`next_action_at` 已经存好数据，可以接入系统通知。
- 统计目前按“投递记录”维度计算；如果需要按“岗位”或“公司”聚合，可以在 `stats.rs` 中增加分组函数。
- GPUI 仍在快速迭代：本项目跟随 `gpui-kit 0.6.1`（框架层 `gpui-pre 0.3.4`）。
  升级时要留意 `gpui_kit::application()`、`AlertDialog` 的 footer、
  以及主题 `colors → tokens → Base` 三层同步这几处。

---

## 许可

MIT
