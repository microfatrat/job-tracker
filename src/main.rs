//! 求职简历投递与面试阶段统计系统（GPUI 桌面应用）。
//!
//! 用法：
//!   cargo run                启动图形界面
//!   cargo run -- --report    只输出文本统计报告（适合无图形环境）
//!   cargo run -- --seed      写入演示数据后退出
//!   cargo run -- --help      查看帮助
//!
//! 数据默认保存在用户数据目录，也可以用环境变量 JOB_TRACKER_DATA 指定文件。

// Windows：release 构建默认使用 GUI 子系统，双击运行时不会弹出黑色控制台窗口；
// 从终端执行 --report/--seed 时，程序会自己附加回父控制台（见 win_console 模块）。
// `windows-gui` feature 用于让 debug 构建也隐藏控制台。
#![cfg_attr(
    all(
        target_os = "windows",
        any(feature = "windows-gui", not(debug_assertions))
    ),
    windows_subsystem = "windows"
)]
// 无头构建（--no-default-features）里没有 ui 模块，model/stats 的一部分
// 公开方法只被界面使用，此时会报 dead_code，这里统一放行。
#![cfg_attr(not(feature = "gui"), allow(dead_code))]

mod demo;
mod model;
mod stats;
mod storage;
#[cfg(feature = "gui")]
mod ui;
#[cfg(target_os = "windows")]
mod win_console;

#[cfg(feature = "gui")]
use gpui::{
    App, Bounds, Focusable, KeyBinding, WindowBounds, WindowOptions, prelude::*, px, size,
};

use crate::{model::Store, stats::Analytics};

#[cfg(feature = "gui")]
use crate::ui::{
    AppShell, RootView,
    app::{AddTag, CloseForm, FocusSearch, NewApplication, QuitApp, SaveForm},
};

fn main() {
    // Windows 的 GUI 子系统构建没有自己的控制台；如果是从终端启动的，
    // 这里把它接回父控制台，保证 --report/--seed 的输出仍能看到。
    #[cfg(target_os = "windows")]
    win_console::attach_parent_console();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let force = args.iter().any(|arg| arg == "--force" || arg == "-f");

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("job-tracker {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.iter().any(|arg| arg == "--seed") {
        if !seed_demo(force) {
            std::process::exit(1);
        }
        return;
    }
    if args
        .iter()
        .any(|arg| arg == "--clear" || arg == "--reset-empty")
    {
        if !clear_data(force) {
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|arg| arg == "--report" || arg == "--stats") {
        print_report();
        return;
    }

    #[cfg(feature = "gui")]
    run_gui();

    #[cfg(not(feature = "gui"))]
    {
        eprintln!("本次构建未启用 gui feature（无头构建），改为输出文本报告。");
        eprintln!("需要图形界面请使用 cargo run（默认启用 gui）。");
        print_report();
    }
}

/// 启动 GPUI 图形界面。
#[cfg(feature = "gui")]
fn run_gui() {
    // gpui-kit 的入口：拿到带平台后端的 Application
    gpui_kit::application()
        // 组件库的图标是 SVG，需要注册 AssetSource 才能渲染。
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx: &mut App| {
        // 组件库自带的界面文案（日历的星期/月份、日期选择器等）用中文。
        gpui_kit::component::set_locale("zh-CN");
        // 组件库初始化：主题、组件全局状态与默认快捷键。
        gpui_kit::init(cx);
        // 把本项目配色灌进组件库主题，避免两套皮肤混用。
        crate::ui::theme::install_component_theme(cx);
        bind_keys(cx);

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if gpui::guess_compositor() == "Headless" {
            eprintln!("未检测到图形后端（DISPLAY / WAYLAND_DISPLAY），改为输出文本报告。");
            eprintln!("提示：图形模式请在有桌面环境的机器上运行 cargo run。");
            print_report();
            cx.quit();
            return;
        }

        // 开窗回调里才能建视图（InputState 需要 window），
        // 这里先把句柄存下来，开窗后用来聚焦根视图。
        let root_view_handle: std::rc::Rc<std::cell::RefCell<Option<gpui::Entity<RootView>>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let bounds = Bounds::centered(None, size(px(1280.), px(820.)), cx);
        let window = match cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("JobFlow - 求职投递与面试阶段统计".into()),
                    ..Default::default()
                }),
                app_id: Some("job-tracker".to_string()),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| RootView::new(window, cx));
                // 记下句柄，开窗后用它设置初始焦点。
                *root_view_handle.borrow_mut() = Some(view.clone());
                // 组件库的 Dialog / Notification / Sheet 都挂在 Root 上，
                // 因此窗口的根视图必须是 gpui_component::Root，
                // 里面再包一层负责渲染浮层的 AppShell。
                let shell = cx.new(|_| AppShell::new(view));
                cx.new(|cx| gpui_kit::component::Root::new(shell, window, cx))
            },
        ) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("创建窗口失败：{error}");
                #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                eprintln!("请确认系统已安装 X11/Wayland 与 Vulkan 运行库。");
                #[cfg(target_os = "windows")]
                eprintln!("请确认当前是交互式桌面会话，并已安装 Microsoft Visual C++ 运行库。");
                eprintln!("也可以使用 cargo run -- --report 直接查看文本报告。");
                cx.quit();
                return;
            }
        };

        let view = root_view_handle.borrow().clone();
        window
            .update(cx, |_root, window, cx| {
                if let Some(view) = &view {
                    let handle = view.read(cx).focus_handle(cx);
                    window.focus(&handle, cx);
                }
                cx.activate(true);
            })
            .ok();
    });
}

/// 只注册应用级快捷键；文本输入的编辑键由组件库的 Input 自行处理。
#[cfg(feature = "gui")]
fn bind_keys(cx: &mut App) {
    // secondary- 在 macOS 上映射为 Command，在 Windows/Linux 上映射为 Ctrl。
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewApplication, None),
        KeyBinding::new("secondary-s", SaveForm, None),
        KeyBinding::new("secondary-f", FocusSearch, None),
        KeyBinding::new("secondary-q", QuitApp, None),
        KeyBinding::new("enter", AddTag, None),
        KeyBinding::new("escape", CloseForm, None),
    ]);

    cx.on_action(|_: &QuitApp, cx| cx.quit());
}

/// 覆盖性操作前的安全闸门：数据非空时必须显式 `--force`，并自动备份。
///
/// 返回 `Ok(Some(备份路径))` 表示已经备份，`Ok(None)` 表示无需备份（数据为空），
/// `Err(())` 表示用户没有加 `--force` 而现有数据非空，调用方应当中止。
fn guard_overwrite(
    path: &std::path::Path,
    force: bool,
    action: &str,
) -> Result<Option<std::path::PathBuf>, ()> {
    let outcome = storage::load(path);
    if outcome.damaged {
        eprintln!("注意：现有数据文件无法解析，原始内容会先备份。");
        if let Some(backup) = &outcome.backup {
            eprintln!("已备份到：{}", backup.display());
        }
    }

    if outcome.store.is_empty() {
        return Ok(None);
    }

    if !force {
        eprintln!(
            "数据文件里已有 {} 条记录：{}",
            outcome.store.len(),
            path.display()
        );
        eprintln!("{action} 会覆盖这些数据。确认请加 --force，例如：");
        eprintln!("  cargo run -- {action} --force");
        return Err(());
    }

    match storage::backup(path, &outcome.store, "backup") {
        Ok(backup) => {
            println!("已备份现有数据到：{}", backup.display());
            Ok(Some(backup))
        }
        Err(error) => {
            eprintln!("备份现有数据失败：{error}，已中止以免丢失数据。");
            Err(())
        }
    }
}

fn seed_demo(force: bool) -> bool {
    let path = storage::data_path();
    if guard_overwrite(&path, force, "--seed").is_err() {
        return false;
    }
    let store = demo::seed_demo(model::today());
    match storage::save(&path, &store) {
        Ok(()) => {
            println!("已写入 {} 条演示数据到 {}", store.len(), path.display());
            println!("运行 cargo run 启动图形界面，或 cargo run -- --report 查看统计。");
            true
        }
        Err(error) => {
            eprintln!("写入失败：{error}");
            false
        }
    }
}

fn clear_data(force: bool) -> bool {
    let path = storage::data_path();
    if guard_overwrite(&path, force, "--clear").is_err() {
        return false;
    }
    let store = Store::default();
    match storage::save(&path, &store) {
        Ok(()) => {
            println!("已清空数据文件：{}", path.display());
            println!("下次启动将显示空数据。如需演示数据，运行 cargo run -- --seed。");
            true
        }
        Err(error) => {
            eprintln!("清空失败：{error}");
            false
        }
    }
}

fn print_help() {
    println!(
        r#"job-tracker {} - 求职简历投递与面试阶段统计系统

用法：
  cargo run                  启动 GPUI 图形界面
  cargo run -- --report      输出文本统计报告
  cargo run -- --seed        写入演示数据后退出（已有数据时需加 --force）
  cargo run -- --clear       清空所有数据（写入空数据文件，已有数据时需加 --force）
  cargo run -- --help        显示帮助
  cargo run -- --version     显示版本

选项：
  --force, -f                覆盖已有数据前先自动备份，然后继续执行 --seed/--clear

环境变量：
  JOB_TRACKER_DATA           自定义数据文件路径

Windows 构建提示：
  默认 gui feature 会嵌入 manifest（需要 rc.exe）；release 构建还需要 fxc.exe。
  缺少这些工具时可以用 cargo run --no-default-features 跳过 manifest。

快捷键（secondary = Windows/Linux 的 Ctrl，macOS 的 Command）：
  secondary + N              新增投递
  secondary + S              保存表单
  secondary + F              聚焦搜索框
  Esc                        关闭表单
  secondary + Q              退出"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn print_report() {
    let path = storage::data_path();
    let outcome = storage::load(&path);
    let store: Store = outcome.store;

    // 读取异常必须先说清楚，绝不误导用户去“写演示数据”覆盖可恢复的数据。
    for notice in &outcome.notices {
        eprintln!("⚠ {notice}");
    }

    if store.is_empty() {
        if outcome.damaged {
            println!("数据文件无法解析，且没有可恢复的记录。");
            println!("原始文件已备份，请先修复或从备份恢复，不要直接覆盖。");
        } else {
            println!("暂无投递数据。运行 cargo run -- --seed 可写入演示数据。");
        }
        println!("数据文件：{}", path.display());
        return;
    }

    let today = model::today();
    let analytics = Analytics::compute(&store, today);
    let overview = &analytics.overview;

    println!("============================================================");
    println!(" 求职投递与面试阶段统计报告");
    println!(" 数据文件：{}", path.display());
    println!(" 统计日期：{}", today);
    println!("============================================================");
    println!(
        "总投递 {} 条 | 进行中 {} | 面试中 {} | Offer {} | 拒绝 {} | 放弃 {}",
        overview.total,
        overview.active,
        overview.interviewing,
        overview.offers,
        overview.rejected,
        overview.withdrawn
    );
    println!(
        "有回复率 {:.1}% | 面试率 {:.1}% | Offer 率 {:.1}% | 待跟进 {} 条",
        overview.response_rate * 100.0,
        overview.interview_rate * 100.0,
        overview.offer_rate * 100.0,
        overview.due_followups
    );
    if let Some(days) = overview.avg_days_to_interview {
        println!("平均面试等待：{:.1} 天", days);
    }
    if let Some(days) = overview.avg_days_to_offer {
        println!("平均 Offer 周期：{:.1} 天", days);
    }
    if overview.future_applied > 0 || overview.chronology_issues > 0 || outcome.skipped > 0 {
        println!(
            "数据体检：投递日期在未来 {} 条 | 阶段事件早于投递日期 {} 条{}",
            overview.future_applied,
            overview.chronology_issues,
            if outcome.skipped > 0 {
                format!(" | 本次读取跳过坏记录 {} 条", outcome.skipped)
            } else {
                String::new()
            }
        );
    }

    println!("\n阶段漏斗：");
    for stat in &analytics.stages {
        println!(
            "  {:<10} 到达 {:>3} 条  到达率 {:>5.1}%  环节留存 {:>5.1}%  当前 {:>2} 条",
            stat.stage.label(),
            stat.reached,
            stat.conversion * 100.0,
            stat.step_conversion * 100.0,
            stat.current
        );
    }

    println!("\n近 6 个月趋势：");
    for month in &analytics.months {
        println!(
            "  {}  投递 {:>2}  面试 {:>2}  Offer {:>2}",
            month.key, month.applied, month.interviews, month.offers
        );
    }

    println!("\n渠道效果：");
    for channel in &analytics.channels {
        println!(
            "  {:<12} 投递 {:>3}  面试 {:>3}  Offer {:>3}  Offer 率 {:>5.1}%",
            channel.channel,
            channel.total,
            channel.interviews,
            channel.offers,
            channel.offer_rate * 100.0
        );
    }

    println!("\n当前阶段分布：");
    for (stage, count) in store.stage_counts() {
        if count > 0 {
            println!("  {:<10} {}", stage.label(), count);
        }
    }

    println!("\n最近动态：");
    for activity in analytics.recent_activity(&store, 8) {
        println!(
            "  {}  {} · {}  → {}",
            activity.at,
            activity.company,
            activity.position,
            activity.stage.label()
        );
    }
}
