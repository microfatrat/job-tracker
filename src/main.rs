//! 求职简历投递与面试阶段统计系统（GPUI 桌面应用）。
//!
//! 用法：
//!   cargo run                启动图形界面
//!   cargo run -- --report    只输出文本统计报告（适合无图形环境）
//!   cargo run -- --seed      写入演示数据后退出
//!   cargo run -- --help      查看帮助
//!
//! 数据默认保存在用户数据目录，也可以用环境变量 JOB_TRACKER_DATA 指定文件。

// 可选：Windows release 构建使用 GUI 子系统，避免在图形界面后面弹出控制台窗口。
// 需要保留 --report/--seed 的控制台输出时，不要启用 windows-gui feature。
#![cfg_attr(
    all(target_os = "windows", feature = "windows-gui", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod demo;
mod model;
mod stats;
mod storage;
mod ui;

use gpui::{
    App, Application, Bounds, Focusable, KeyBinding, WindowBounds, WindowOptions, prelude::*, px,
    size,
};

use crate::{
    model::Store,
    stats::Analytics,
    ui::{
        RootView,
        app::{AddTag, CloseForm, FocusSearch, NewApplication, QuitApp, SaveForm},
        text_input::{
            InputBackspace, InputCopy, InputCut, InputDelete, InputEnd, InputHome, InputLeft,
            InputPaste, InputRight, InputSelectAll, InputSelectLeft, InputSelectRight,
        },
    },
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("job-tracker {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.iter().any(|arg| arg == "--seed") {
        seed_demo();
        return;
    }
    if args
        .iter()
        .any(|arg| arg == "--clear" || arg == "--reset-empty")
    {
        clear_data();
        return;
    }
    if args.iter().any(|arg| arg == "--report" || arg == "--stats") {
        print_report();
        return;
    }

    Application::new().run(|cx: &mut App| {
        bind_keys(cx);

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if gpui::guess_compositor() == "Headless" {
            eprintln!("未检测到图形后端（DISPLAY / WAYLAND_DISPLAY），改为输出文本报告。");
            eprintln!("提示：图形模式请在有桌面环境的机器上运行 cargo run。");
            print_report();
            cx.quit();
            return;
        }

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
            |_, cx| cx.new(|cx| RootView::new(cx)),
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

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
                cx.activate(true);
            })
            .ok();
    });
}

fn bind_keys(cx: &mut App) {
    // secondary- 在 macOS 上映射为 Command，在 Windows/Linux 上映射为 Ctrl。
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewApplication, None),
        KeyBinding::new("secondary-s", SaveForm, None),
        KeyBinding::new("secondary-f", FocusSearch, None),
        KeyBinding::new("secondary-q", QuitApp, None),
        KeyBinding::new("enter", AddTag, None),
        KeyBinding::new("escape", CloseForm, None),
        KeyBinding::new("backspace", InputBackspace, Some("TextInput")),
        KeyBinding::new("delete", InputDelete, Some("TextInput")),
        KeyBinding::new("left", InputLeft, Some("TextInput")),
        KeyBinding::new("right", InputRight, Some("TextInput")),
        KeyBinding::new("shift-left", InputSelectLeft, Some("TextInput")),
        KeyBinding::new("shift-right", InputSelectRight, Some("TextInput")),
        KeyBinding::new("secondary-a", InputSelectAll, Some("TextInput")),
        KeyBinding::new("secondary-c", InputCopy, Some("TextInput")),
        KeyBinding::new("secondary-x", InputCut, Some("TextInput")),
        KeyBinding::new("secondary-v", InputPaste, Some("TextInput")),
        KeyBinding::new("home", InputHome, Some("TextInput")),
        KeyBinding::new("end", InputEnd, Some("TextInput")),
    ]);

    cx.on_action(|_: &QuitApp, cx| cx.quit());
}

fn seed_demo() {
    let path = storage::data_path();
    let store = demo::seed_demo(model::today());
    match storage::save(&path, &store) {
        Ok(()) => {
            println!("已写入 {} 条演示数据到 {}", store.len(), path.display());
            println!("运行 cargo run 启动图形界面，或 cargo run -- --report 查看统计。");
        }
        Err(error) => eprintln!("写入失败：{error}"),
    }
}

fn clear_data() {
    let path = storage::data_path();
    let store = Store::default();
    match storage::save(&path, &store) {
        Ok(()) => {
            println!("已清空数据文件：{}", path.display());
            println!("下次启动将显示空数据。如需演示数据，运行 cargo run -- --seed。");
        }
        Err(error) => eprintln!("清空失败：{error}"),
    }
}

fn print_help() {
    println!(
        r#"job-tracker {} - 求职简历投递与面试阶段统计系统

用法：
  cargo run                  启动 GPUI 图形界面
  cargo run -- --report      输出文本统计报告
  cargo run -- --seed        写入演示数据后退出
  cargo run -- --clear       清空所有数据（写入空数据文件）
  cargo run -- --help        显示帮助
  cargo run -- --version     显示版本

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
    let store: Store = storage::load(&path);
    if store.is_empty() {
        println!("暂无投递数据。运行 cargo run -- --seed 可写入演示数据。");
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

    println!("\n阶段漏斗：");
    for stat in &analytics.stages {
        println!(
            "  {:<10} 到达 {:>3} 人  到达率 {:>5.1}%  环节留存 {:>5.1}%  当前 {:>2} 人",
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
