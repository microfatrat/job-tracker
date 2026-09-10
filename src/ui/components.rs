//! 可复用的 UI 小组件（基于 `gpui-component` 组件库实现）。
//!
//! 这里保留原有的 helper 名称与用法，内部改用组件库的 Button / Tag /
//! Progress / ListItem 等，样式与交互（hover、focus ring、禁用态、键盘可达性）
//! 交给组件库统一处理。

use gpui::{App, Div, FontWeight, Hsla, ParentElement, SharedString, Styled, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Selectable as _, Sizable as _,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    list::ListItem,
    progress::Progress,
    tag::Tag,
};

use crate::{
    model::{self, JobApplication, Stage},
    stats::StageStat,
    ui::theme,
};

/// 卡片容器。
pub fn card() -> Div {
    div()
        .bg(theme::panel())
        .border_1()
        .border_color(theme::border())
        .rounded(px(10.))
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
                .text_2xl()
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

/// 阶段徽章（组件库 Tag，保留每个阶段的配色）。
pub fn stage_badge(stage: Stage) -> Tag {
    Tag::custom(
        theme::stage_soft(stage),
        theme::stage_color(stage),
        theme::stage_color(stage),
    )
    .rounded_full()
    .child(stage.label())
}

/// 进度条（组件库 Progress）。
pub fn progress_bar(fraction: f32, color: Hsla) -> Progress {
    Progress::new().value(fraction.clamp(0.0, 1.0) * 100.0).bg(color)
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
        .items_start()
        .gap_4()
        .py_0p5()
        .child(
            div()
                .flex_shrink_0()
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
    empty_state_with_icon(IconName::Inbox, message)
}

/// 带图标的空状态提示。
pub fn empty_state_with_icon(icon: IconName, message: impl Into<String>) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .py_8()
        .child(Icon::new(icon).size(px(28.)).text_color(theme::subtle()))
        .child(
            div()
                .text_sm()
                .text_color(theme::muted())
                .child(message.into()),
        )
}

/// 主按钮。
pub fn primary_button(id: impl Into<SharedString>, label: impl Into<String>) -> Button {
    Button::new(id.into()).primary().label(label.into())
}

/// 次要按钮。
pub fn secondary_button(id: impl Into<SharedString>, label: impl Into<String>) -> Button {
    Button::new(id.into()).label(label.into())
}

/// 危险操作按钮。
pub fn danger_button(id: impl Into<SharedString>, label: impl Into<String>) -> Button {
    Button::new(id.into()).danger().label(label.into())
}

/// 侧边栏导航按钮。
///
/// 侧边栏是深色底，组件库默认的按钮文字色是深色前景色，直接用会看不见，
/// 因此这里用 `custom` 变体显式指定深色底上的配色。
pub fn nav_button(
    cx: &App,
    id: impl Into<SharedString>,
    icon: IconName,
    label: impl Into<String>,
    active: bool,
) -> Button {
    let variant = if active {
        ButtonCustomVariant::new(cx)
            .color(theme::sidebar_active())
            .foreground(gpui::white())
            .hover(theme::sidebar_active())
            .active(theme::sidebar_active())
    } else {
        ButtonCustomVariant::new(cx)
            .color(theme::sidebar_bg())
            .foreground(theme::text_on_dark())
            .hover(theme::sidebar_hover())
            .active(theme::sidebar_hover())
    };

    Button::new(id.into())
        .w_full()
        .justify_start()
        .icon(Icon::new(icon))
        .label(label.into())
        .custom(variant)
}

/// 阶段筛选按钮。
pub fn stage_chip(
    stage: Option<Stage>,
    active: bool,
    id: impl Into<SharedString>,
) -> Button {
    let label = stage.map_or("全部".to_string(), |stage| stage.label().to_string());
    Button::new(id.into())
        .small()
        .label(label)
        .selected(active)
        .when(active, |button| button.primary())
}

/// 标签徽章（组件库 Tag）。
pub fn tag_badge(tag: &str) -> Tag {
    Tag::secondary()
        .rounded_full()
        .child(tag.to_string())
}

/// 通用筛选按钮（用于标签筛选）。
pub fn filter_chip(
    label: impl Into<String>,
    active: bool,
    id: impl Into<SharedString>,
) -> Button {
    Button::new(id.into())
        .small()
        .label(label.into())
        .selected(active)
        .when(active, |button| button.primary())
}

/// 列表行：公司 + 岗位 + 阶段 + 日期（组件库 ListItem，带 hover/选中样式）。
pub fn application_row(
    application: &JobApplication,
    selected: bool,
    due: bool,
    id: impl Into<SharedString>,
) -> ListItem {
    let id: SharedString = id.into();
    let stage = application.stage;
    let date = model::format_date_short(application.applied_at, model::today());
    ListItem::new(id)
        .selected(selected)
        .rounded(px(6.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .w_full()
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
                                    el.child(Tag::warning().rounded_full().child("待跟进"))
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
                                    .children(
                                        application.tags.iter().take(3).map(|tag| tag_badge(tag)),
                                    ),
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
                ),
        )
}

/// 水平分隔线。
pub fn divider() -> Div {
    div().h(px(1.)).w_full().bg(theme::border())
}
