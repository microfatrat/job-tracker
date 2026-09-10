//! 统一的颜色与视觉风格。

use gpui::{Hsla, rgb};

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
