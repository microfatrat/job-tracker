# 更新日志

本项目遵循[语义化版本](https://semver.org/lang/zh-CN/)，1.0 之前次版本号可能包含不兼容变更。

## [0.3.2] - 2026-09-20

### 修复

- **转化指标的计数单位由「人」改为「条」**：阶段漏斗与「关键转化指标」统计的是
  **投递记录条数**（一条记录算一条，同一家公司多次投递算多条），原来的
  「上一环节 18 人，本环节 14 人」把记录数写成了人数，容易误解为按候选人或按公司去重：
  - 「阶段漏斗与转化率」副标题改为「到达率 = 到达该阶段的投递数 / 总投递数」；
  - 「关键转化指标」明细行改为「上一环节 X 条，本环节 Y 条，流失 Z 条」；
  - `--report` 的阶段漏斗同步改为「到达 X 条 / 当前 Y 条」。
- 设置页与 README 里遗留的「快捷键」文案清理：设置页早就没有快捷键卡片了，
  页面副标题却仍写着「数据管理、快捷键与构建信息」，现在统一为「数据管理、数据体检与构建信息」。
- 文档截图重新生成：原先 `docs/screenshots/settings.png` 与 `analytics.png` 内容重复
  （设置页截图其实是阶段统计页），现在四个页面截图各自对应正确页面。

## [0.3.1] - 2026-09-11

### 修复

- **侧边栏导航项恢复左对齐**：gpui-component 0.6 的 `Button` 内部把内容包在
  `h_flex().size_full().justify_center()` 里，外层的 `justify_start` 无效，
  导致「图标 + 文字」整体居中且四项各不对齐；激活项的背景也被叠加了透明度，
  显示成暗蓝色而不是实心主色。现在导航项改为自绘 div：图标固定 18px 列宽、
  文字左对齐、激活项实心 `#1d4ed8`。

## [0.3.0] - 2026-09-10

### 变更（依赖栈升级）

- **UI 依赖整体迁到 [gpui-kit](https://github.com/longbridge/gpui-kit) 0.6.1**：
  zed 的 gpui 从 crates.io 的 `gpui 0.2.2` 换成新一代快照 `gpui-pre 0.3.4`，
  组件库从 `gpui-component 0.5.1` 升到 `0.6.1`，资源包从 `gpui-component-assets`
  换成 `gpui-kit-assets`。依赖只剩一个入口：
  `gpui-kit = { version = "0.6.1" }`（默认带 `component` + `assets`），
  平台后端由 kit 内部的 `gpui-pre-platform` 统一打开。
- 代码侧随之调整：
  - 启动改为 `gpui_kit::application()` + `gpui_kit::init(cx)`（`Application::new()` 已移除）；
  - 图标资源改为 `gpui_kit::assets::Assets`；
  - 表单对话框用 `AlertDialog`（0.6 的 `Dialog` 只渲染自定义 footer），
    footer 由自己渲染两个按钮并直接绑回调；
  - 多行备注改用 `Textarea` / `TextareaState`（0.6 把多行输入从 `InputState` 拆出去了）；
  - `Progress::new()` 现在需要传一个稳定的 id；
  - `Window::focus(handle, cx)` 多了一个参数。
- **主题对接跟着升级**：组件取色链路变成 `colors → tokens → Base 层语义 token`，
  且按钮读的是 `colors.button_primary` 这类组件级字段。`theme::install_component_theme`
  现在会一并写入组件级颜色、重建 `ThemeTokens` 并调用 `Theme::sync_base(cx)`，
  否则按钮等控件会退回默认主题（近黑色主色）。
- 构建依赖：新依赖链里的 `yeslogic-fontconfig-sys` 需要 `fontconfig.pc`，
  Linux 上要装 `fontconfig-devel freetype-devel`（Ubuntu：`libfontconfig1-dev libfreetype6-dev`），
  CI 与 Release 工作流已补上；README 的本地无 root 前缀方案也补了对应说明。
- 设置页「关于」改为显示 `gpui-kit 0.6.1 / gpui-pre 0.3.4` 与 `gpui-component 0.6.1`。

功能与界面行为保持不变（27 个单元测试、界面交互均已回归验证）。

## [0.2.3] - 2026-09-10

### 新增

- **投递日期 / 下一步日期改为日历选择**：使用组件库的 `DatePicker`（中文月份与星期），
  投递日期只能选到今天及以前（未来日期在日历里禁用），下一步日期可留空、可选未来。
  日期不再需要手输，因此移除了 `model::parse_date` / `parse_optional_date` 两个文本解析函数。

### 修复

- 输入框边框不可见：组件库把 `theme.input` 当边框色用，之前被错误映射成白色，
  现在映射为 `border_strong`（文本输入、日期选择器、下拉框统一生效）。
- 组件库界面文案跟随系统语言设为中文（`gpui_component::set_locale("zh-CN")`）。

## [0.2.2] - 2026-09-10

### 修复

- **Windows 上不再弹出黑色控制台窗口**：release 构建改为默认使用 GUI 子系统
  （`windows_subsystem = "windows"`），双击运行只有图形界面。同时新增
  `win_console` 模块：从终端执行 `job-tracker.exe --report` / `--seed` 时
  会自动附加回父控制台，命令行输出照常显示；输出重定向到文件时保持重定向。
  `windows-gui` feature 保留，含义变为「连 debug 构建也隐藏控制台」。

### 变更

- 设置页不再展示「数据版本」「数据格式」与整张「数据兼容」卡片：
  版本号属于实现细节，普通用户不需要看到。数据迁移仍在后台自动进行。
- 设置页不再展示「快捷键」卡片（快捷键仍有效，说明保留在 README）。

## [0.2.1] - 2026-09-10

### 修复

- **打包脚本缺少可执行位**：`scripts/package-linux.sh`、`scripts/package-macos.sh`
  在 git 里是 `100644`，导致 CI 与 Release 工作流执行 `./scripts/...` 时直接
  "Permission denied" 失败（Windows 走 PowerShell 不受影响）。
  现在标记为可执行，Release 工作流可以正常产出 tar.gz / .deb / dmg / zip。

## [0.2.0] - 2026-09-10

### 新增

- **界面改用 [gpui-component](https://github.com/longbridge/gpui-component) 0.5.1**：
  对话框表单、`Input` 输入框、`Notification` 通知、`Tag` 标签、`Progress` 进度条、
  `ListItem` 列表行、`Alert` 提示与 Lucide 图标，替换掉原先自绘的控件。
- 支持**撤销上次阶段变更**，误点阶段后可以一键回退（口径同步回落）。
- 设置页新增**数据体检**：展示本次读取发现的问题、跳过的坏记录与备份文件路径。
- `--seed` / `--clear` 在已有数据时必须加 `--force`，加 `--force` 时会先自动备份。
- 搜索支持「下一步动作」字段。
- 提供 `theme::install_component_theme`，把项目配色灌进组件库主题。

### 修复

- **数据文件解析失败不再静默当空数据**：先把原始内容备份为 `名字.bad-<时间戳>.json`，
  再逐条恢复可解析的记录；`--report` 不再引导用户用 `--seed` 覆盖，界面会以错误提示告知。
- **单条坏记录不再导致整库丢弃**：改为逐条解析，坏记录跳过并计数上报。
- **重置演示数据 / 删除记录改为二次确认**，重置与清空在覆盖前自动备份。
- **并发保存不再互相截断**：临时文件名带进程号与序号，失败时清理临时文件。
- **补录不再污染周期指标**：新建记录时若直接选了后续阶段，阶段事件按投递日期写入；
  编辑时「已投递」事件跟随投递日期。
- **日期倒挂不再被 `max(0)` 掩盖**：剔除出平均值计算，并在界面与报告中单独提示。
- 数据迁移到 **version 3**：补齐缺失的阶段历史（漏斗第一行不再凭空流失）、规范化标签。
- 渠道分组忽略大小写与首尾空白；未来投递日期不计入「最近 30 天」且表单直接拒绝。
- 编辑保存只在内容真的变化时刷新 `updated_at`，不再误清「久未更新」标记。
- 筛选变化时自动校正选中项，详情面板不再展示被筛掉的记录。
- 标签去重与筛选口径统一（含非 ASCII 大小写）；搜索覆盖下一步动作。
- 统计数据按数据版本缓存，不再每帧重复计算。

### 变更

- 删除自研的 `src/ui/text_input.rs`（634 行），文本输入改用组件库实现。
- 数据格式 `version 2` → `version 3`（加载时自动迁移）。
- `ui` 模块与组件库挂在 `gui` feature 下，`--no-default-features` 的无头构建
  不再被拖入图形依赖。
- 数据文件路径不变，旧数据可直接使用。

## [0.1.0] - 2026-08

首个版本：投递管理、仪表盘、阶段统计、设置，JSON 本地持久化与演示数据。
