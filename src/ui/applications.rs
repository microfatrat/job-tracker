//! 投递管理页面：列表、筛选、详情与阶段操作。

use gpui::{Context, FontWeight, IntoElement, SharedString, div, prelude::*, px};

use crate::{
    model::{self, Stage},
    ui::{app::RootView, components::*, theme},
};

impl RootView {
    pub fn render_applications(&self, cx: &Context<Self>) -> impl IntoElement {
        let visible = self.visible_ids(cx);
        let total = self.store.len();
        let selected = self.selected.and_then(|id| self.store.get(id).cloned());
        let today = model::today();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .w_full()
            .h_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
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
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme::text())
                                            .child("投递管理"),
                                    )
                                    .child(div().text_sm().text_color(theme::muted()).child(
                                        format!(
                                            "共 {} 条记录，当前筛选显示 {} 条",
                                            total,
                                            visible.len()
                                        ),
                                    )),
                            )
                            .child(div().flex().flex_row().gap_2().child(
                                secondary_button("add-application", "＋ 新增投递").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.open_form(None, window, cx)
                                    }),
                                ),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                stage_chip(None, self.stage_filter.is_none(), "filter-all")
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.set_stage_filter(None, cx)
                                    })),
                            )
                            .children(Stage::ALL.iter().map(|stage| {
                                let stage = *stage;
                                stage_chip(
                                    Some(stage),
                                    self.stage_filter == Some(stage),
                                    format!("filter-{:?}", stage),
                                )
                                .on_click(cx.listener(
                                    move |this, _, _window, cx| {
                                        this.set_stage_filter(Some(stage), cx)
                                    },
                                ))
                            })),
                    )
                    .when(!self.store.all_tags().is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::subtle())
                                        .mr_1()
                                        .child("标签："),
                                )
                                .child(
                                    filter_chip(
                                        "全部标签",
                                        self.tag_filter.is_none(),
                                        "tag-filter-all",
                                    )
                                    .on_click(cx.listener(
                                        |this, _, _window, cx| this.set_tag_filter(None, cx),
                                    )),
                                )
                                .children(self.store.all_tags().into_iter().map(|tag| {
                                    let tag_for_click = tag.clone();
                                    filter_chip(
                                        tag.clone(),
                                        self.tag_filter.as_deref().is_some_and(|current| {
                                            model::tags_equal(current, &tag)
                                        }),
                                        SharedString::from(format!("tag-filter-{}", tag)),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _window, cx| {
                                            this.set_tag_filter(Some(tag_for_click.clone()), cx)
                                        },
                                    ))
                                })),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_start()
                    .flex_1()
                    .min_h(px(0.))
                    .child(
                        div().flex().flex_col().flex_1().min_w(px(0.)).child(
                            card()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .px_3()
                                        .py_2p5()
                                        .bg(theme::panel_alt())
                                        .border_b_1()
                                        .border_color(theme::border())
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme::muted())
                                        .child(div().flex_1().child("公司 / 岗位"))
                                        .child(div().w(px(90.)).child("当前阶段"))
                                        .child(div().w(px(48.)).child("投递")),
                                )
                                .when(visible.is_empty(), |el| {
                                    el.child(empty_state(
                                        "没有匹配的投递记录，试试清空搜索或切换筛选",
                                    ))
                                })
                                .children(visible.into_iter().filter_map(|id| {
                                    let application = self.store.get(id)?;
                                    let due = application.follow_up_due(today);
                                    let is_selected = self.selected == Some(id);
                                    Some(
                                        application_row(
                                            application,
                                            is_selected,
                                            due,
                                            format!("application-row-{}", id),
                                        )
                                        .on_click(
                                            cx.listener(move |this, _, _window, cx| {
                                                this.select(id, cx)
                                            }),
                                        ),
                                    )
                                })),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(380.))
                            .flex_shrink_0()
                            .child(self.render_detail_panel(selected.as_ref(), cx)),
                    ),
            )
    }

    fn render_detail_panel(
        &self,
        application: Option<&crate::model::JobApplication>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let Some(application) = application else {
            return card()
                .p_5()
                .child(empty_state("从左侧选择一条投递记录查看详情"));
        };

        let id = application.id;
        let stage = application.stage;
        let next_stage = stage.next_progress();
        let can_advance = next_stage.is_some();
        let history = application.history.clone();

        card()
            .flex()
            .flex_col()
            .gap_3()
            .p_5()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .min_w(px(0.))
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme::text())
                                            .truncate()
                                            .child(application.company.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::muted())
                                            .truncate()
                                            .child(application.position.clone()),
                                    ),
                            )
                            .child(stage_badge(stage)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(secondary_button("edit-application", "编辑").on_click(
                                cx.listener(move |this, _, window, cx| {
                                    this.open_form(Some(id), window, cx)
                                }),
                            ))
                            .child(danger_button("delete-application", "删除").on_click(
                                cx.listener(|this, _, _window, cx| this.delete_selected(cx)),
                            )),
                    ),
            )
            .child(divider())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::muted())
                            .child("阶段流转"),
                    )
                    .child(div().flex().flex_row().flex_wrap().gap_2().children(
                        Stage::PROGRESS.iter().map(|progress| {
                            let progress = *progress;
                            stage_chip(
                                Some(progress),
                                stage == progress,
                                format!("detail-stage-{:?}", progress),
                            )
                            .on_click(cx.listener(
                                move |this, _, _window, cx| this.set_selected_stage(progress, cx),
                            ))
                        }),
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(
                                primary_button(
                                    "advance-stage",
                                    next_stage
                                        .map(|stage| format!("推进到 {}", stage.label()))
                                        .unwrap_or_else(|| "已是 Offer".to_string()),
                                )
                                .when(!can_advance, |el| el.opacity(0.5))
                                .when(can_advance, |el| {
                                    el.on_click(
                                        cx.listener(|this, _, _window, cx| {
                                            this.advance_selected(cx)
                                        }),
                                    )
                                }),
                            )
                            .child(secondary_button("mark-rejected", "标记拒绝").on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.set_selected_stage(Stage::Rejected, cx)
                                }),
                            ))
                            .child(secondary_button("mark-withdrawn", "标记放弃").on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.set_selected_stage(Stage::Withdrawn, cx)
                                }),
                            )),
                    ),
            )
            .child(divider())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(field_row("投递渠道", empty_dash(&application.channel)))
                    .child(field_row("城市", empty_dash(&application.location)))
                    .child(field_row("薪资范围", empty_dash(&application.salary)))
                    .child(field_row("投递日期", application.applied_at.to_string()))
                    .child(field_row("最近更新", application.updated_at.to_string()))
                    .child(field_row(
                        "下一步动作",
                        empty_dash(&application.next_action),
                    ))
                    .child(field_row(
                        "下一步日期",
                        application
                            .next_action_at
                            .map(|date| date.to_string())
                            .unwrap_or_else(|| "--".to_string()),
                    ))
                    .child(field_row(
                        "面试等待",
                        application
                            .days_to_first_interview()
                            .map(|days| format!("{} 天", days))
                            .unwrap_or_else(|| "--".to_string()),
                    ))
                    .child(field_row(
                        "Offer 周期",
                        application
                            .days_to_offer()
                            .map(|days| format!("{} 天", days))
                            .unwrap_or_else(|| "--".to_string()),
                    )),
            )
            .when(!application.tags.is_empty(), |el| {
                el.child(divider()).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::muted())
                                .child("标签"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .gap_2()
                                .children(application.tags.iter().map(|tag| tag_badge(tag))),
                        ),
                )
            })
            .when(!application.notes.trim().is_empty(), |el| {
                el.child(divider()).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::muted())
                                .child("备注"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::text())
                                .child(application.notes.clone()),
                        ),
                )
            })
            .child(divider())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::muted())
                            .child(format!("阶段历史 · {} 条", history.len())),
                    )
                    .children(history.into_iter().rev().map(|event| {
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(8.))
                                    .h(px(8.))
                                    .mt_1p5()
                                    .rounded_full()
                                    .bg(theme::stage_color(event.stage)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
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
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(theme::text())
                                                    .child(event.stage.label().to_string()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::subtle())
                                                    .child(event.at.to_string()),
                                            ),
                                    )
                                    .when(!event.note.trim().is_empty(), |el| {
                                        el.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::muted())
                                                .child(event.note.clone()),
                                        )
                                    }),
                            )
                    })),
            )
    }
}

fn empty_dash(value: &str) -> String {
    if value.trim().is_empty() {
        "--".to_string()
    } else {
        value.to_string()
    }
}
