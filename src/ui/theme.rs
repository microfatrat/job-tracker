//! 统一的颜色与视觉风格。

use gpui::{App, Hsla, px, rgb};

use crate::model::Stage;

pub fn app_bg() -> Hsla {
    rgb(0xf1f5f9).into()
}

pub fn sidebar_bg() -> Hsla {
    rgb(0x0f172a).into()
}

pub fn sidebar_hover() -> Hsla {
    rgb(0x1e293b).into()
}

pub fn sidebar_active() -> Hsla {
    rgb(0x1d4ed8).into()
}

pub fn panel() -> Hsla {
    rgb(0xffffff).into()
}

pub fn panel_alt() -> Hsla {
    rgb(0xf8fafc).into()
}

pub fn border() -> Hsla {
    rgb(0xe2e8f0).into()
}

pub fn border_strong() -> Hsla {
    rgb(0xcbd5e1).into()
}

pub fn text() -> Hsla {
    rgb(0x0f172a).into()
}

pub fn text_on_dark() -> Hsla {
    rgb(0xe2e8f0).into()
}

pub fn muted() -> Hsla {
    rgb(0x64748b).into()
}

pub fn subtle() -> Hsla {
    rgb(0x94a3b8).into()
}

pub fn accent() -> Hsla {
    rgb(0x2563eb).into()
}

pub fn accent_soft() -> Hsla {
    rgb(0xdbeafe).into()
}

pub fn success() -> Hsla {
    rgb(0x16a34a).into()
}

pub fn success_soft() -> Hsla {
    rgb(0xdcfce7).into()
}

pub fn warning() -> Hsla {
    rgb(0xd97706).into()
}

pub fn warning_soft() -> Hsla {
    rgb(0xfef3c7).into()
}

pub fn danger() -> Hsla {
    rgb(0xdc2626).into()
}

pub fn danger_soft() -> Hsla {
    rgb(0xfee2e2).into()
}

pub fn purple() -> Hsla {
    rgb(0x7c3aed).into()
}

pub fn purple_soft() -> Hsla {
    rgb(0xede9fe).into()
}

pub fn teal() -> Hsla {
    rgb(0x0d9488).into()
}

pub fn teal_soft() -> Hsla {
    rgb(0xccfbf1).into()
}

pub fn orange() -> Hsla {
    rgb(0xea580c).into()
}

pub fn orange_soft() -> Hsla {
    rgb(0xffedd5).into()
}

pub fn indigo() -> Hsla {
    rgb(0x4f46e5).into()
}

pub fn indigo_soft() -> Hsla {
    rgb(0xe0e7ff).into()
}

pub fn cyan() -> Hsla {
    rgb(0x0891b2).into()
}

pub fn cyan_soft() -> Hsla {
    rgb(0xcffafe).into()
}

pub fn slate() -> Hsla {
    rgb(0x475569).into()
}

pub fn slate_soft() -> Hsla {
    rgb(0xe2e8f0).into()
}

/// 阶段主色。
pub fn stage_color(stage: Stage) -> Hsla {
    match stage {
        Stage::Applied => slate(),
        Stage::ResumeScreening => accent(),
        Stage::WrittenTest => purple(),
        Stage::Interview1 => teal(),
        Stage::Interview2 => cyan(),
        Stage::Interview3 => indigo(),
        Stage::HrInterview => orange(),
        Stage::Offer => success(),
        Stage::Rejected => danger(),
        Stage::Withdrawn => rgb(0x6b7280).into(),
    }
}

/// 阶段浅色背景。
pub fn stage_soft(stage: Stage) -> Hsla {
    match stage {
        Stage::Applied => slate_soft(),
        Stage::ResumeScreening => accent_soft(),
        Stage::WrittenTest => purple_soft(),
        Stage::Interview1 => teal_soft(),
        Stage::Interview2 => cyan_soft(),
        Stage::Interview3 => indigo_soft(),
        Stage::HrInterview => orange_soft(),
        Stage::Offer => success_soft(),
        Stage::Rejected => danger_soft(),
        Stage::Withdrawn => rgb(0xe5e7eb).into(),
    }
}

/// 根据转化率选择颜色。
pub fn rate_color(rate: f32) -> Hsla {
    if rate >= 0.6 {
        success()
    } else if rate >= 0.3 {
        warning()
    } else {
        danger()
    }
}

/// 把本项目的配色灌进 `gpui-component` 的主题。
///
/// 组件库自带的是 shadcn 中性色（主色接近纯黑），直接混用会和本项目的
/// 石板灰 + 蓝色主色打架。这里把两边的语义色对齐，之后组件库的
/// Button / Tag / Input / ListItem / Progress 与自绘部分就是同一套皮肤。
pub fn install_component_theme(cx: &mut App) {
    use gpui_component::Theme;

    let theme = Theme::global_mut(cx);

    // 基础排版：桌面端数据密集型界面，比默认的 16px 更紧凑一些。
    theme.font_size = px(14.);
    theme.radius = px(6.);
    theme.radius_lg = px(10.);
    theme.shadow = true;

    let c = &mut theme.colors;
    // 组件库里的 background 指的是「窗口/卡片底色」（shadcn 默认是纯白），
    // 不是我们页面用的浅灰底；弄反了对话框会变成灰底。
    c.background = panel();
    c.foreground = text();
    c.border = border();
    c.input = panel();
    c.caret = text();
    c.ring = accent();
    c.selection = accent_soft();
    c.popover = panel();
    c.popover_foreground = text();

    c.primary = accent();
    c.primary_foreground = rgb(0xffffff).into();
    c.primary_hover = rgb(0x1d4ed8).into();
    c.primary_active = rgb(0x1e40af).into();

    c.secondary = panel();
    c.secondary_foreground = text();
    c.secondary_hover = panel_alt();
    c.secondary_active = slate_soft();

    c.accent = accent_soft();
    c.accent_foreground = accent();

    c.muted = panel_alt();
    c.muted_foreground = muted();

    c.danger = danger();
    c.danger_foreground = rgb(0xffffff).into();
    c.danger_hover = rgb(0xb91c1c).into();
    c.danger_active = rgb(0x991b1b).into();

    c.success = success();
    c.success_foreground = rgb(0xffffff).into();
    c.warning = warning();
    c.warning_foreground = rgb(0xffffff).into();
    c.info = accent();
    c.info_foreground = rgb(0xffffff).into();

    c.list = panel();
    c.list_hover = panel_alt();
    c.list_active = accent_soft();
    c.list_active_border = accent();
    c.list_even = panel_alt();
    c.list_head = panel_alt();

    c.sidebar = sidebar_bg();
    c.sidebar_foreground = text_on_dark();
    c.sidebar_accent = sidebar_active();
    c.sidebar_accent_foreground = rgb(0xffffff).into();
    c.sidebar_border = sidebar_hover();

    c.group_box = panel_alt();
    c.group_box_foreground = text();
    c.progress_bar = accent();
    c.scrollbar = panel_alt();
    c.scrollbar_thumb = border_strong();
    c.scrollbar_thumb_hover = muted();
}
