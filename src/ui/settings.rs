//! 设置页面：数据管理、快捷键与版本信息。

use gpui::{Context, IntoElement, div, prelude::*};
use gpui_component::alert::Alert;

use crate::ui::{app::RootView, components::*, theme};

impl RootView {
    pub fn render_settings(&self, cx: &Context<Self>) -> impl IntoElement {
        let path = self.data_path.display().to_string();
        let file_size = std::fs::metadata(&self.data_path)
            .map(|metadata| metadata.len())
            .ok();
        let file_size_text = file_size
            .map(format_bytes)
            .unwrap_or_else(|| "文件尚未创建".to_string());
        let tags = self.store.all_tags();

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
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text())
                            .child("设置"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::muted())
                            .child("数据管理、快捷键与构建信息"),
                    ),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("数据管理", "查看当前数据文件并重置数据"))
                    .child(field_row("数据文件", path))
                    .child(field_row(
                        "投递记录",
                        format!("{} 条", self.store.len()),
                    ))
                    .child(field_row("标签数量", format!("{} 个", tags.len())))
                    .child(field_row("文件大小", file_size_text))
                    .child(divider())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .mt_2()
                            .child(
                                secondary_button("settings-reset-demo", if self.confirm_reset {
                                    "再次点击确认重置"
                                } else {
                                    "重置为演示数据"
                                })
                                .on_click(cx.listener(
                                    |this, _, _window, cx| this.request_reset_demo(cx),
                                )),
                            )
                            .child(if self.confirm_clear {
                                danger_button("settings-clear-all", "再次点击确认清空").on_click(
                                    cx.listener(|this, _, _window, cx| {
                                        this.request_clear_all(cx)
                                    }),
                                )
                            } else {
                                danger_button("settings-clear-all", "清空所有数据").on_click(
                                    cx.listener(|this, _, _window, cx| {
                                        this.request_clear_all(cx)
                                    }),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::muted())
                            .mt_2()
                            .child(
                                "「重置为演示数据」和「清空所有数据」都需要点击两次确认，并且会先把当前数据备份成同目录下的 .reset-/.clear- 文件。",
                            ),
                    ),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("数据体检", "本次启动时发现的问题"))
                    .children(if self.data_damaged {
                        vec![
                            Alert::error(
                                "data-damaged",
                                "数据文件损坏：原始内容已备份，未被自动覆盖，请从备份文件人工修复。",
                            )
                            .title("数据文件上次读取失败")
                            .into_any_element(),
                        ]
                    } else if self.load_notices.is_empty() {
                        vec![
                            Alert::success("data-ok", "本次读取未发现异常").into_any_element(),
                        ]
                    } else {
                        Vec::new()
                    })
                    .children(
                        self.load_notices
                            .iter()
                            .map(|notice| {
                                div()
                                    .text_xs()
                                    .text_color(theme::muted())
                                    .mt_1()
                                    .child(notice.clone())
                            })
                            .collect::<Vec<_>>(),
                    )
                    .child(field_row(
                        "跳过的坏记录",
                        format!("{} 条", self.load_skipped),
                    ))
                    .child(field_row(
                        "备份文件",
                        self.load_backup
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "无".to_string()),
                    ))
            )
            .child(
                card()
                    .p_5()
                    .child(card_title(
                        "快捷键",
                        "secondary = Windows/Linux 的 Ctrl，macOS 的 Command",
                    ))
                    .child(field_row("secondary + N", "新增投递"))
                    .child(field_row("secondary + S", "保存表单"))
                    .child(field_row("secondary + F", "聚焦搜索框"))
                    .child(field_row("secondary + Q", "退出"))
                    .child(field_row("Esc", "关闭表单"))
                    .child(field_row("Enter（标签输入框）", "添加标签")),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("关于", "构建信息"))
                    .child(field_row("应用版本", env!("CARGO_PKG_VERSION")))
                    .child(field_row("GPUI", "0.2.2"))
                    .child(field_row(
                        "构建类型",
                        if cfg!(debug_assertions) {
                            "debug"
                        } else {
                            "release"
                        },
                    )),
            )
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
