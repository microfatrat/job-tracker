//! 仪表盘页面。

use gpui::{Context, Div, FontWeight, IntoElement, SharedString, div, prelude::*, px};

use crate::{
    model::{self},
    stats::Analytics,
    ui::{app::RootView, components::*, theme},
};

impl RootView {
    pub fn render_dashboard(&self, analytics: &Analytics, cx: &Context<Self>) -> impl IntoElement {
        let today = model::today();
        let overview = &analytics.overview;
        let follow_ups = analytics.follow_ups(&self.store, today, 6);
        let activities = analytics.recent_activity(&self.store, 8);
        let max_applied = analytics
            .months
            .iter()
            .map(|month| month.applied)
            .max()
            .unwrap_or(1)
            .max(1);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text())
                                    .child("求职进展总览"),
                            )
                            .child(div().text_sm().text_color(theme::muted()).child(format!(
                                "{} · 共 {} 条投递，最近 30 天新增 {} 条",
                                today.format("%Y年%m月%d日"),
                                overview.total,
                                overview.applied_last_30_days
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_full()
                            .bg(if overview.due_followups > 0 {
                                theme::warning_soft()
                            } else {
                                theme::success_soft()
                            })
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if overview.due_followups > 0 {
                                theme::warning()
                            } else {
                                theme::success()
                            })
                            .child(if overview.due_followups > 0 {
                                format!("今日待跟进 {} 条", overview.due_followups)
                            } else {
                                "今日暂无待跟进".to_string()
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(div().flex_1().child(kpi_card(
                        "总投递",
                        overview.total.to_string(),
                        format!("本月新增 {}", overview.applied_this_month),
                        theme::accent(),
                    )))
                    .child(div().flex_1().child(kpi_card(
                        "进行中",
                        overview.active.to_string(),
                        format!("其中面试中 {}", overview.interviewing),
                        theme::teal(),
                    )))
                    .child(div().flex_1().child(kpi_card(
                        "已拿 Offer",
                        overview.offers.to_string(),
                        format!("Offer 率 {:.0}%", overview.offer_rate * 100.0),
                        theme::success(),
                    )))
                    .child(div().flex_1().child(kpi_card(
                        "已拒绝 / 放弃",
                        format!("{} / {}", overview.rejected, overview.withdrawn),
                        format!("面试率 {:.0}%", overview.interview_rate * 100.0),
                        theme::danger(),
                    ))),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_start()
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            card()
                                .p_5()
                                .child(card_title("阶段漏斗", "从投递到 Offer 的逐级转化"))
                                .child(
                                    div().flex().flex_col().children(
                                        analytics
                                            .stages
                                            .iter()
                                            .map(|stat| funnel_row(stat, overview.total)),
                                    ),
                                ),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            card()
                                .p_5()
                                .child(card_title("近期待办", "未来 14 天内需要跟进的动作"))
                                .when(follow_ups.is_empty(), |el| {
                                    el.child(empty_state("暂无待办，保持节奏就好"))
                                })
                                .children(follow_ups.into_iter().map(|application| {
                                    let id = application.id;
                                    let company = application.company.clone();
                                    let action = if application.next_action.is_empty() {
                                        "待跟进".to_string()
                                    } else {
                                        application.next_action.clone()
                                    };
                                    let date = application
                                        .next_action_at
                                        .map(|date| model::format_date_short(date, today))
                                        .unwrap_or_else(|| "--".to_string());
                                    let overdue =
                                        application.next_action_at.is_some_and(|date| date < today);
                                    div()
                                        .id(SharedString::from(format!("follow-{}", id)))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap_3()
                                        .py_2()
                                        .border_b_1()
                                        .border_color(theme::border())
                                        .cursor_pointer()
                                        .hover(|style| style.bg(theme::panel_alt()))
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            this.selected = Some(id);
                                            this.tab = crate::ui::app::Tab::Applications;
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .min_w(px(0.))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(theme::text())
                                                        .truncate()
                                                        .child(company),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::muted())
                                                        .truncate()
                                                        .child(action),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_full()
                                                .bg(if overdue {
                                                    theme::danger_soft()
                                                } else {
                                                    theme::accent_soft()
                                                })
                                                .text_xs()
                                                .text_color(if overdue {
                                                    theme::danger()
                                                } else {
                                                    theme::accent()
                                                })
                                                .child(date),
                                        )
                                })),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_start()
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            card()
                                .p_5()
                                .child(card_title("近 6 个月投递趋势", "柱状图展示每月投递数量"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_end()
                                        .gap_3()
                                        .h(px(170.))
                                        .children(analytics.months.iter().map(|month| {
                                            let height = if max_applied == 0 {
                                                4.0
                                            } else {
                                                (month.applied as f32 / max_applied as f32 * 120.0)
                                                    .max(4.0)
                                            };
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .justify_end()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(theme::accent())
                                                        .child(month.applied.to_string()),
                                                )
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .h(px(height))
                                                        .bg(theme::accent())
                                                        .rounded_t_md(),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::muted())
                                                        .child(month.label.clone()),
                                                )
                                        })),
                                ),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            card()
                                .p_5()
                                .child(card_title("最近动态", "最新的阶段变更记录"))
                                .when(activities.is_empty(), |el| {
                                    el.child(empty_state("还没有阶段变更记录"))
                                })
                                .children(activities.into_iter().map(|activity| {
                                    let id = activity.application_id;
                                    let company = activity.company.clone();
                                    let position = activity.position.clone();
                                    let stage = activity.stage;
                                    let date = model::format_date_short(activity.at, today);
                                    div()
                                        .id(SharedString::from(format!("activity-{}", id)))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .py_2()
                                        .border_b_1()
                                        .border_color(theme::border())
                                        .cursor_pointer()
                                        .hover(|style| style.bg(theme::panel_alt()))
                                        .on_click(cx.listener(move |this, _, _window, cx| {
                                            this.selected = Some(id);
                                            this.tab = crate::ui::app::Tab::Applications;
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .w(px(8.))
                                                .h(px(8.))
                                                .rounded_full()
                                                .bg(theme::stage_color(stage)),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .min_w(px(0.))
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(theme::text())
                                                        .truncate()
                                                        .child(format!("{company} · {position}")),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::muted())
                                                        .child(
                                                            if activity.note.trim().is_empty() {
                                                                activity.stage.label().to_string()
                                                            } else {
                                                                activity.note.clone()
                                                            },
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div().text_xs().text_color(theme::subtle()).child(date),
                                        )
                                })),
                        ),
                    ),
            )
    }

    /// 小工具：生成一个占位 Div，保证泛型推导正常。
    pub fn _placeholder(&self) -> Div {
        div()
    }
}
