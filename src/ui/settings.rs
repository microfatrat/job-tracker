//! 设置页面：数据管理、快捷键与版本信息。

use gpui::{Context, IntoElement, div, prelude::*};

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
                    .child(field_row(
                        "数据版本",
                        format!("version {}", self.store.version),
                    ))
                    .child(field_row("文件大小", file_size_text))
                    .child(divider())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .mt_2()
                            .child(
                                secondary_button("settings-reset-demo", "重置为演示数据")
                                    .on_click(cx.listener(
                                        |this, _, _window, cx| this.reset_demo(cx),
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
                                "「重置为演示数据」会覆盖当前数据；「清空所有数据」需要点击两次确认，清空后写入一个空的数据文件。",
                            ),
                    ),
            )
            .child(
                card()
                    .p_5()
                    .child(card_title("数据兼容", "旧版本数据可以直接使用"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::text())
                            .child("当前数据格式为 version 2。"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::muted())
                            .mt_1()
                            .child(
                                "旧数据没有 tags 字段时会自动补为空数组，加载时自动迁移到 version 2。",
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::muted())
                            .mt_1()
                            .child("标签最长 24 个字符，重复标签会自动忽略（大小写不敏感）。"),
                    ),
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
                    ))
                    .child(field_row(
                        "数据格式",
                        format!("version {}", self.store.version),
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
