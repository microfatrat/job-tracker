//! 可复用的 UI 小组件。

use gpui::{Div, FontWeight, Hsla, SharedString, div, prelude::*, px, relative};

use crate::{
    model::{JobApplication, Stage},
    stats::StageStat,
    ui::theme,
};

/// 白色卡片容器。
pub fn card() -> Div {
    div()
        .bg(theme::panel())
        .border_1()
        .border_color(theme::border())
        .rounded_lg()
        .shadow_sm()
}

/// 卡片标题。
pub fn card_title(title: impl Into<String>, subtitle: impl Into<String>) -> Div {
    let subtitle = subtitle.into();
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .mb_3()
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text())
                .child(title.into()),
        )
        .when(!subtitle.is_empty(), |el| {
            el.child(div().text_sm().text_color(theme::muted()).child(subtitle))
        })
}

/// 指标卡片。
pub fn kpi_card(
    title: impl Into<String>,
    value: impl Into<String>,
    caption: impl Into<String>,
    color: Hsla,
) -> Div {
    card()
        .flex()
        .flex_col()
        .gap_1()
        .p_4()
        .child(
            div()
                .text_sm()
                .text_color(theme::muted())
                .child(title.into()),
        )
        .child(
            div()
                .text_3xl()
                .font_weight(FontWeight::BOLD)
                .text_color(color)
                .child(value.into()),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme::subtle())
                .child(caption.into()),
        )
}

/// 阶段徽章。
pub fn stage_badge(stage: Stage) -> Div {
    div()
        .px_2()
        .py_0p5()
        .rounded_full()
        .bg(theme::stage_soft(stage))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme::stage_color(stage))
        .child(stage.label())
}

/// 进度条。
pub fn progress_bar(fraction: f32, color: Hsla) -> Div {
    let fraction = fraction.clamp(0.0, 1.0);
    div()
        .h(px(8.))
        .w_full()
        .bg(theme::slate_soft())
        .rounded_full()
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(fraction))
                .bg(color)
                .rounded_full(),
        )
}

/// 漏斗阶段行。
pub fn funnel_row(stat: &StageStat, max_reached: usize) -> Div {
    let width = if max_reached == 0 {
        0.0
    } else {
        stat.reached as f32 / max_reached as f32
    };
    let color = theme::stage_color(stat.stage);
    let rate = stat.conversion;
    let rate_text = format!("{:.0}%", rate * 100.0);

    div()
        .flex()
        .flex_col()
        .gap_1()
        .py_1p5()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .w(px(140.))
                        .child(div().w(px(10.)).h(px(10.)).rounded_full().bg(color))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text())
                                .child(stat.stage.label()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .flex_1()
                        .child(div().flex_1().child(progress_bar(width, color)))
                        .child(
                            div()
                                .w(px(44.))
                                .text_right()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text())
                                .child(format!("{}", stat.reached)),
                        )
                        .child(
                            div()
                                .w(px(52.))
                                .text_right()
                                .text_xs()
                                .text_color(theme::rate_color(rate))
                                .child(rate_text),
                        ),
                ),
        )
        .child(
            div()
                .pl(px(18.))
                .text_xs()
                .text_color(theme::subtle())
                .child(format!(
                    "当前 {} 条 · 环节留存 {:.0}% · 环节流失 {:.0}%",
                    stat.current,
                    stat.step_conversion * 100.0,
                    stat.drop_off * 100.0
                )),
        )
}

/// 一行“标签 + 值”。
pub fn field_row(label: impl Into<String>, value: impl Into<String>) -> Div {
    let value = value.into();
    div()
        .flex()
        .flex_row()
        .justify_between()
        .gap_4()
        .py_0p5()
        .child(
            div()
                .text_xs()
                .text_color(theme::muted())
                .child(label.into()),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text())
                .text_right()
                .child(value),
        )
}

/// 空状态提示。
pub fn empty_state(message: impl Into<String>) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .py_8()
        .text_sm()
        .text_color(theme::muted())
        .child(message.into())
}

/// 主按钮。
pub fn primary_button(
    id: impl Into<SharedString>,
    label: impl Into<String>,
) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(id)
        .px_3()
        .py_1p5()
        .rounded_md()
        .bg(theme::accent())
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(gpui::white())
        .cursor_pointer()
        .hover(|style| style.bg(gpui::rgb(0x1d4ed8)))
        .child(label.into())
}

/// 次要按钮。
pub fn secondary_button(
    id: impl Into<SharedString>,
    label: impl Into<String>,
) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(id)
        .px_3()
        .py_1p5()
        .rounded_md()
        .bg(theme::panel_alt())
        .border_1()
        .border_color(theme::border_strong())
        .text_sm()
        .text_color(theme::text())
        .cursor_pointer()
        .hover(|style| style.bg(theme::slate_soft()))
        .child(label.into())
}

/// 危险操作按钮。
pub fn danger_button(id: impl Into<SharedString>, label: impl Into<String>) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(id)
        .px_3()
        .py_1p5()
        .rounded_md()
        .bg(theme::danger_soft())
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme::danger())
        .cursor_pointer()
        .hover(|style| style.bg(gpui::rgb(0xfecaca)))
        .child(label.into())
}

/// 阶段筛选按钮。
pub fn stage_chip(
    stage: Option<Stage>,
    active: bool,
    id: impl Into<SharedString>,
) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    let label = stage.map_or("全部".to_string(), |stage| stage.label().to_string());
    let color = stage.map(theme::stage_color).unwrap_or(theme::accent());
    let soft = stage.map(theme::stage_soft).unwrap_or(theme::accent_soft());
    div()
        .id(id)
        .px_2p5()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(if active { color } else { theme::border() })
        .bg(if active { soft } else { theme::panel() })
        .text_xs()
        .font_weight(if active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(if active { color } else { theme::muted() })
        .cursor_pointer()
        .hover(move |style| style.border_color(color))
        .child(label)
}

/// 标签徽章。
pub fn tag_badge(tag: &str) -> Div {
    div()
        .px_1p5()
        .py_0p5()
        .rounded_full()
        .bg(theme::accent_soft())
        .text_xs()
        .text_color(theme::accent())
        .child(tag.to_string())
}

/// 通用筛选 chip（用于标签筛选）。
pub fn filter_chip(
    label: impl Into<String>,
    active: bool,
    id: impl Into<SharedString>,
) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    div()
        .id(id)
        .px_2p5()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(if active {
            theme::accent()
        } else {
            theme::border()
        })
        .bg(if active {
            theme::accent_soft()
        } else {
            theme::panel()
        })
        .text_xs()
        .font_weight(if active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(if active {
            theme::accent()
        } else {
            theme::muted()
        })
        .cursor_pointer()
        .hover(|style| style.border_color(theme::accent()))
        .child(label.into())
}

/// 列表行：公司 + 岗位 + 阶段 + 日期。
pub fn application_row(
    application: &JobApplication,
    selected: bool,
    due: bool,
    id: impl Into<SharedString>,
) -> gpui::Stateful<Div> {
    let id: SharedString = id.into();
    let stage = application.stage;
    let date = application.applied_at.format("%m-%d").to_string();
    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2p5()
        .border_b_1()
        .border_color(theme::border())
        .bg(if selected {
            theme::accent_soft()
        } else {
            theme::panel()
        })
        .cursor_pointer()
        .hover(|style| style.bg(theme::panel_alt()))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_0p5()
                .min_w(px(0.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text())
                                .truncate()
                                .child(application.company.clone()),
                        )
                        .when(due, |el| {
                            el.child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(theme::warning_soft())
                                    .text_xs()
                                    .text_color(theme::warning())
                                    .child("待跟进"),
                            )
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::muted())
                        .truncate()
                        .child(application.position.clone()),
                )
                .when(!application.tags.is_empty(), |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap_1()
                            .children(application.tags.iter().take(3).map(|tag| tag_badge(tag))),
                    )
                }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_end()
                .gap_1()
                .child(stage_badge(stage))
                .child(div().text_xs().text_color(theme::subtle()).child(date)),
        )
}

/// 水平分隔线。
pub fn divider() -> Div {
    div().h(px(1.)).w_full().bg(theme::border())
}
