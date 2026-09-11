//! 阶段统计页面：漏斗、转化率、月度趋势与渠道效果。

use gpui::{Context, FontWeight, IntoElement, SharedString, div, prelude::*, px};

use crate::{
    model::{self, Stage},
    stats::Analytics,
    ui::{app::RootView, components::*, theme},
};

impl RootView {
    pub fn render_analytics(&self, analytics: &Analytics, _cx: &Context<Self>) -> impl IntoElement {
        let today = model::today();
        let overview = &analytics.overview;
        let max_applied = analytics
            .months
            .iter()
            .map(|month| month.applied)
            .max()
            .unwrap_or(1)
            .max(1);
        let stage_counts = self.store.stage_counts();
        let tag_counts = self.store.tag_counts();
        let data_warnings: Vec<String> = [
            (overview.chronology_issues, "条记录存在早于投递日期的阶段事件，已排除在平均周期之外"),
            (overview.future_applied, "条记录的投递日期晚于今天，未计入最近 30 天与月度趋势"),
        ]
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .map(|(count, text)| format!("{count} {text}"))
        .collect();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .w_full()
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
                            .child("面试阶段统计"),
                    )
                    .child(div().text_sm().text_color(theme::muted()).child(format!(
                        "基于 {} 条投递记录计算 · 统计日期 {}",
                        overview.total, today
                    )))
                    .children(data_warnings.into_iter().map(|warning| {
                        div()
                            .text_xs()
                            .text_color(theme::warning())
                            .child(format!("⚠ {warning}"))
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(div().flex_1().child(kpi_card(
                        "有回复率",
                        format!("{:.0}%", overview.response_rate * 100.0),
                        format!("{} / {} 条有进展", overview.responded, overview.total),
                        theme::accent(),
                    )))
                    .child(div().flex_1().child(kpi_card(
                        "面试率",
                        format!("{:.0}%", overview.interview_rate * 100.0),
                        format!("{} 条进入面试", overview.interviewed),
                        theme::teal(),
                    )))
                    .child(div().flex_1().child(kpi_card(
                        "Offer 率",
                        format!("{:.0}%", overview.offer_rate * 100.0),
                        format!("{} 条曾拿到 Offer（含之后放弃）", overview.offers),
                        theme::success(),
                    )))
                    .child(
                        div().flex_1().child(kpi_card(
                            "平均面试等待",
                            overview
                                .avg_days_to_interview
                                .map(|days| format!("{:.0} 天", days))
                                .unwrap_or_else(|| "--".to_string()),
                            "从投递到第一次面试（仅统计已面试的记录）",
                            theme::purple(),
                        )),
                    )
                    .child(
                        div().flex_1().child(kpi_card(
                            "平均 Offer 周期",
                            overview
                                .avg_days_to_offer
                                .map(|days| format!("{:.0} 天", days))
                                .unwrap_or_else(|| "--".to_string()),
                            "从投递到拿到 Offer（仅统计已拿 Offer 的记录）",
                            theme::orange(),
                        )),
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
                                .child(card_title(
                                    "阶段漏斗与转化率",
                                    "到达率 = 到达该阶段的人数 / 总投递数",
                                ))
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
                                .child(card_title("关键转化指标", "每个环节相对上一环节的留存"))
                                .children(analytics.stages.iter().skip(1).map(|stat| {
                                    let color = theme::rate_color(stat.step_conversion);
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .py_2()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(theme::text())
                                                        .child(format!(
                                                            "{} → {}",
                                                            previous_stage_label(stat.stage),
                                                            stat.stage.label()
                                                        )),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(color)
                                                        .child(format!(
                                                            "{:.0}%",
                                                            stat.step_conversion * 100.0
                                                        )),
                                                ),
                                        )
                                        .child(progress_bar(
                                            SharedString::from(format!(
                                                "funnel-detail-{:?}",
                                                stat.stage
                                            )),
                                            stat.step_conversion,
                                            color,
                                        ))
                                        .child({
                                            let previous = stat.previous_reached;
                                            let lost = previous.saturating_sub(stat.reached);
                                            div().text_xs().text_color(theme::subtle()).child(
                                                format!(
                                                    "上一环节 {} 人，本环节 {} 人，流失 {} 人",
                                                    previous, stat.reached, lost
                                                ),
                                            )
                                        })
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
                                .child(card_title(
                                    "近 6 个月趋势",
                                    "投递 / 面试 / Offer 的月度变化",
                                ))
                                .child(div().flex().flex_col().gap_2().children(
                                    analytics.months.iter().map(|month| {
                                        let applied_width = if max_applied == 0 {
                                            0.0
                                        } else {
                                            month.applied as f32 / max_applied as f32
                                        };
                                        let interview_width = if max_applied == 0 {
                                            0.0
                                        } else {
                                            month.interviews as f32 / max_applied as f32
                                        };
                                        let offer_width = if max_applied == 0 {
                                            0.0
                                        } else {
                                            month.offers as f32 / max_applied as f32
                                        };
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .py_1()
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(theme::text())
                                                            .child(month.label.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme::muted())
                                                            .child(format!(
                                                                "投递 {} · 面试 {} · Offer {}",
                                                                month.applied,
                                                                month.interviews,
                                                                month.offers
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .w(px(40.))
                                                                    .text_xs()
                                                                    .text_color(theme::subtle())
                                                                    .child("投递"),
                                                            )
                                                            .child(div().flex_1().child(
                                                                progress_bar(
                                                                    "trend-applied",
                                                                    applied_width,
                                                                    theme::accent(),
                                                                ),
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .w(px(40.))
                                                                    .text_xs()
                                                                    .text_color(theme::subtle())
                                                                    .child("面试"),
                                                            )
                                                            .child(div().flex_1().child(
                                                                progress_bar(
                                                                    "trend-interview",
                                                                    interview_width,
                                                                    theme::teal(),
                                                                ),
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .w(px(40.))
                                                                    .text_xs()
                                                                    .text_color(theme::subtle())
                                                                    .child("Offer"),
                                                            )
                                                            .child(div().flex_1().child(
                                                                progress_bar(
                                                                    "trend-offer",
                                                                    offer_width,
                                                                    theme::success(),
                                                                ),
                                                            )),
                                                    ),
                                            )
                                    }),
                                )),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            card()
                                .p_5()
                                .child(card_title(
                                    "投递渠道效果",
                                    "不同渠道的回复、面试与 Offer 表现",
                                ))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .pb_2()
                                        .border_b_1()
                                        .border_color(theme::border())
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme::muted())
                                        .child(div().flex_1().child("渠道"))
                                        .child(div().w(px(40.)).text_right().child("投递"))
                                        .child(div().w(px(40.)).text_right().child("面试"))
                                        .child(div().w(px(40.)).text_right().child("Offer"))
                                        .child(div().w(px(56.)).text_right().child("Offer 率")),
                                )
                                .when(analytics.channels.is_empty(), |el| {
                                    el.child(empty_state("暂无渠道数据"))
                                })
                                .children(analytics.channels.iter().map(|channel| {
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .py_2()
                                        .border_b_1()
                                        .border_color(theme::border())
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(0.))
                                                .text_sm()
                                                .text_color(theme::text())
                                                .truncate()
                                                .child(channel.channel.clone()),
                                        )
                                        .child(
                                            div()
                                                .w(px(40.))
                                                .text_right()
                                                .text_sm()
                                                .text_color(theme::text())
                                                .child(channel.total.to_string()),
                                        )
                                        .child(
                                            div()
                                                .w(px(40.))
                                                .text_right()
                                                .text_sm()
                                                .text_color(theme::teal())
                                                .child(channel.interviews.to_string()),
                                        )
                                        .child(
                                            div()
                                                .w(px(40.))
                                                .text_right()
                                                .text_sm()
                                                .text_color(theme::success())
                                                .child(channel.offers.to_string()),
                                        )
                                        .child(
                                            div()
                                                .w(px(56.))
                                                .text_right()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme::rate_color(channel.offer_rate))
                                                .child(format!(
                                                    "{:.0}%",
                                                    channel.offer_rate * 100.0
                                                )),
                                        )
                                })),
                        ),
                    ),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("当前阶段分布", "每条投递目前停留在哪个阶段"))
                    .child(div().flex().flex_row().flex_wrap().gap_3().children(
                        stage_counts.into_iter().map(|(stage, count)| {
                            let total = overview.total.max(1);
                            let width = count as f32 / total as f32;
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .w(px(150.))
                                .p_3()
                                .rounded_md()
                                .bg(theme::stage_soft(stage))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(theme::stage_color(stage))
                                                .child(stage.label()),
                                        )
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::stage_color(stage))
                                                .child(count.to_string()),
                                        ),
                                )
                                .child(progress_bar(
                                    SharedString::from(format!("dist-{:?}", stage)),
                                    width,
                                    theme::stage_color(stage),
                                ))
                        }),
                    )),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("标签分布", "所有投递记录使用的标签"))
                    .when(tag_counts.is_empty(), |el| {
                        el.child(empty_state("暂无标签"))
                    })
                    .children(tag_counts.into_iter().map(|(tag, count)| {
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme::border())
                            .child(div().text_sm().text_color(theme::text()).child(tag))
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(theme::accent_soft())
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::accent())
                                    .child(count.to_string()),
                            )
                    })),
            )
    }
}

fn previous_stage_label(stage: Stage) -> &'static str {
    let index = stage.progress_index().unwrap_or(0);
    if index == 0 {
        "投递"
    } else {
        Stage::PROGRESS[index - 1].label()
    }
}
