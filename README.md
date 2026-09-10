# JobFlow · 求职简历投递与面试阶段统计系统

一个用 **Rust + GPUI** 编写的桌面应用，用来管理求职投递记录、跟踪面试阶段流转，并实时统计漏斗转化率、面试率、Offer 率、月度趋势与投递渠道效果。

> 界面语言为中文，数据保存在本地 JSON 文件中，不依赖任何在线服务。

---

## 功能特性

### 1. 投递管理

- 记录公司、岗位、投递渠道、城市、薪资范围、投递日期、备注、下一步动作与日期。
- 阶段流转：已投递 → 简历筛选 → 笔试/测评 → 一面 → 二面 → 三面/终面 → HR 面 → Offer。
- 支持终止状态：已拒绝、已放弃；终止后仍保留此前到达过的阶段历史。
- 每次阶段变更都会写入历史事件，用于精确计算“到达过哪些阶段”和周期天数。
- 标签分类：支持自定义标签，也可以从已有标签中选择添加；支持按标签筛选。
- 支持按公司/岗位/渠道/备注/标签搜索，按阶段和标签筛选。
- 支持新增、编辑、删除、推进阶段、手动切换阶段、清空数据、重置演示数据。

### 2. 仪表盘

- KPI 卡片：总投递、进行中、已拿 Offer、已拒绝/放弃。
- 阶段漏斗：每个阶段的到达人数、到达率、环节留存率、环节流失率。
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
- 保存采用“先写临时文件再替换”的方式，尽量避免写入一半导致文件损坏。
- **数据兼容旧版本**：旧数据文件没有 `tags` 字段时会自动补为空数组，并把 `version` 迁移到 2；旧的 v1 数据可以直接被新版读取。

### 5. 设置

- 数据管理：查看数据文件路径、投递记录数、标签数、数据版本和文件大小；
- **重置为演示数据** 和 **清空所有数据** 两个操作集中在设置页；
- 数据兼容说明、快捷键列表和构建信息。

---

## 界面截图

仓库的 `docs/screenshots/` 目录保存了 Linux release 构建的实际运行截图（1280×820）：

| 页面 | 截图 |
| --- | --- |
| 仪表盘 | [dashboard.png](docs/screenshots/dashboard.png) |
| 投递管理 | [applications.png](docs/screenshots/applications.png) |
| 阶段统计 | [analytics.png](docs/screenshots/analytics.png) |
| 新增投递表单（含标签分类） | [form.png](docs/screenshots/form.png) |
| 添加自定义标签后的表单 | [form-tags.png](docs/screenshots/form-tags.png) |
| 表单输入 | [form-typed.png](docs/screenshots/form-typed.png) |
| 设置（数据管理、快捷键、关于） | [settings.png](docs/screenshots/settings.png) |

> 截图来自 release 构建（`opt-level=2 + thin LTO + strip`），Linux 二进制约 18 MB，压缩后的 tar.gz 约 6.7 MB。

---

## 技术栈

| 组件 | 说明 |
| --- | --- |
| Rust 2024 | `edition = "2024"`，需要 Rust 1.85+ |
| GPUI 0.2.2 | Zed 的 GPU 加速 UI 框架（crates.io 版本） |
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
        ├── app.rs           # RootView：侧边栏、顶部栏、表单、事件处理
        ├── dashboard.rs     # 仪表盘页面
        ├── applications.rs  # 投递管理页面
        ├── analytics.rs     # 阶段统计页面
        ├── components.rs    # 卡片、徽章、进度条、按钮等组件
        ├── text_input.rs    # 支持 IME 中文输入的文本输入框
        └── theme.rs         # 颜色与阶段配色
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

这会写入一个空的 JSON 数据文件，下次启动显示空数据。也可以使用界面侧边栏底部的 **“清空所有数据”** 按钮（需要点击两次确认）。

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
.\scripts\package-windows.ps1 -WindowsGui  # 隐藏 release 控制台窗口
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

推送形如 `v0.1.0` 的 tag 后，`.github/workflows/release.yml` 会在 Windows、Linux 和 macOS 上自动构建，并把 ZIP / tar.gz / .deb / dmg / SHA256 上传到 GitHub Release：

```bash
git tag v0.1.0
git push origin v0.1.0
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
- 默认 release 构建保留控制台，`--report` / `--seed` 正常输出。
- 如果希望发布版双击运行时没有控制台窗口，可以启用 `windows-gui` feature：

  ```powershell
  cargo build --release --features windows-gui
  ```

  注意：启用后 release 版的 `--report` / `--seed` 控制台输出不可见；查看报告请使用 debug 构建（`cargo run -- --report`）。
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
  fontconfig freetype google-noto-sans-cjk-fonts \
  libxcb libxcb-devel
```

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
  libxkbcommon-dev libxkbcommon-x11-dev \
  libvulkan-dev vulkan-tools mesa-vulkan-drivers \
  libfontconfig1 libfreetype6 fonts-noto-cjk \
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

---

## 统计口径

- **有回复率** = 到达“简历筛选”及之后阶段的投递数 / 总投递数。
- **面试率** = 到达“一面”及之后阶段的投递数 / 总投递数。
- **Offer 率** = 拿到过 Offer 的投递数 / 总投递数。
- **到达人数**：历史事件中出现过该阶段，或当前阶段在该阶段及之后，即算“到达”。
- **环节留存** = 本阶段到达人数 / 上一阶段到达人数。
- **环节流失** = 1 - 环节留存。
- **平均面试等待** = 所有“第一次进入面试”事件的（面试日期 - 投递日期）平均值。
- **平均 Offer 周期** = 所有“第一次拿到 Offer”事件的（Offer 日期 - 投递日期）平均值。
- **月度趋势**：按投递日期统计每月新增投递数；面试/Offer 按对应事件日期统计。
- **渠道效果**：按投递渠道分组统计投递数、面试数、Offer 数与 Offer 率。

---

## 数据文件示例

```json
{
  "version": 1,
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

当前包含 12 个单元测试：

- 总览指标计数正确；
- 漏斗到达人数单调不增；
- 月度趋势固定返回 6 个月；
- 平均面试等待天数计算正确；
- 演示数据一致性（含标签非空）；
- 存储读写往返；
- 空数据文件往返（`--clear` 场景）；
- 缺失文件加载为空数据；
- 旧版本数据（没有 `tags` 字段）自动迁移到 version 2；
- 标签收集与计数（大小写去重）；
- 按标签筛选（大小写不敏感）；
- 标签规范化（去空白、长度限制）。

---

## Git 工作流

项目已经初始化为 Git 仓库，默认分支 `main`，当前版本 tag 为 `v0.1.0`。

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
git push origin v0.1.0
```

推送形如 `v0.1.0` 的 tag 会触发 `.github/workflows/release.yml`，自动在 Windows/Linux 上构建并创建 GitHub Release：

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
- GPUI 仍在快速迭代，如果升级 gpui 版本，可能需要同步调整少量 API（本项目固定在 0.2.2）。

---

## 许可

MIT
