//! 应用根视图：侧边栏、顶部栏、标签页切换与表单。

use std::path::PathBuf;

use chrono::NaiveDate;
use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, IntoElement, Render,
    SharedString, Stateful, Window, actions, div, prelude::*, px, relative,
};
use uuid::Uuid;

use crate::{
    demo,
    model::{self, JobApplication, Stage, Store},
    stats::Analytics,
    storage,
    ui::{components::*, text_input::TextInput, theme},
};

actions!(
    job_tracker,
    [
        NewApplication,
        SaveForm,
        CloseForm,
        FocusSearch,
        RefreshStats,
        QuitApp,
        AddTag,
    ]
);

/// 顶部导航标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Applications,
    Analytics,
    Settings,
}

impl Tab {
    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "仪表盘",
            Tab::Applications => "投递管理",
            Tab::Analytics => "阶段统计",
            Tab::Settings => "设置",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Tab::Dashboard => "一眼看清整体求职进展与待办",
            Tab::Applications => "维护每一条投递记录与阶段流转",
            Tab::Analytics => "漏斗、转化率与渠道效果分析",
            Tab::Settings => "数据管理、快捷键与版本信息",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Tab::Dashboard => "▦",
            Tab::Applications => "▤",
            Tab::Analytics => "▥",
            Tab::Settings => "⚙",
        }
    }
}

/// 轻量提示消息。
#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

/// 新增/编辑表单状态。
pub struct FormState {
    pub company: Entity<TextInput>,
    pub position: Entity<TextInput>,
    pub channel: Entity<TextInput>,
    pub location: Entity<TextInput>,
    pub salary: Entity<TextInput>,
    pub applied_at: Entity<TextInput>,
    pub next_action: Entity<TextInput>,
    pub next_action_at: Entity<TextInput>,
    pub notes: Entity<TextInput>,
    pub tag_input: Entity<TextInput>,
    pub tags: Vec<String>,
    pub stage: Stage,
}

/// 从表单解析出的数据。
pub struct FormData {
    pub company: String,
    pub position: String,
    pub channel: String,
    pub location: String,
    pub salary: String,
    pub applied_at: NaiveDate,
    pub next_action: String,
    pub next_action_at: Option<NaiveDate>,
    pub notes: String,
    pub tags: Vec<String>,
    pub stage: Stage,
}

impl FormState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            company: cx.new(|cx| TextInput::new("例如：星海科技", cx)),
            position: cx.new(|cx| TextInput::new("例如：Rust 后端工程师", cx)),
            channel: cx.new(|cx| TextInput::new("BOSS直聘 / 内推 / 官网 ...", cx)),
            location: cx.new(|cx| TextInput::new("例如：上海", cx)),
            salary: cx.new(|cx| TextInput::new("例如：30-45K", cx)),
            applied_at: cx.new(|cx| TextInput::new("YYYY-MM-DD", cx)),
            next_action: cx.new(|cx| TextInput::new("例如：准备二面", cx)),
            next_action_at: cx.new(|cx| TextInput::new("YYYY-MM-DD（可留空）", cx)),
            notes: cx.new(|cx| TextInput::new("备注：面试反馈、联系人、薪资细节 ...", cx)),
            tag_input: cx.new(|cx| TextInput::new("输入自定义标签，回车或点击添加", cx)),
            tags: Vec::new(),
            stage: Stage::Applied,
        }
    }

    /// 清空表单，准备新增。
    pub fn clear(&mut self, cx: &mut App) {
        for input in [
            &self.company,
            &self.position,
            &self.channel,
            &self.location,
            &self.salary,
            &self.applied_at,
            &self.next_action,
            &self.next_action_at,
            &self.notes,
            &self.tag_input,
        ] {
            input.update(cx, |input, cx| input.clear(cx));
        }
        self.applied_at.update(cx, |input, cx| {
            input.set_value(model::today().to_string(), cx)
        });
        self.tags.clear();
        self.stage = Stage::Applied;
    }

    /// 载入一条已有记录，准备编辑。
    pub fn load(&mut self, application: &JobApplication, cx: &mut App) {
        let pairs: [(&Entity<TextInput>, String); 9] = [
            (&self.company, application.company.clone()),
            (&self.position, application.position.clone()),
            (&self.channel, application.channel.clone()),
            (&self.location, application.location.clone()),
            (&self.salary, application.salary.clone()),
            (&self.applied_at, application.applied_at.to_string()),
            (&self.next_action, application.next_action.clone()),
            (
                &self.next_action_at,
                application
                    .next_action_at
                    .map(|date| date.to_string())
                    .unwrap_or_default(),
            ),
            (&self.notes, application.notes.clone()),
        ];
        for (input, value) in pairs {
            input.update(cx, |input, cx| input.set_value(value, cx));
        }
        self.tags = application.tags.clone();
        self.tag_input.update(cx, |input, cx| input.clear(cx));
        self.stage = application.stage;
    }

    /// 校验并解析表单。
    pub fn to_data(&self, cx: &App) -> Result<FormData, String> {
        let company = self.company.read(cx).value().trim().to_string();
        let position = self.position.read(cx).value().trim().to_string();
        if company.is_empty() {
            return Err("公司名称不能为空".to_string());
        }
        if position.is_empty() {
            return Err("岗位名称不能为空".to_string());
        }
        let applied_at = model::parse_date(&self.applied_at.read(cx).value())?;
        let next_action_at = model::parse_optional_date(&self.next_action_at.read(cx).value())?;
        Ok(FormData {
            company,
            position,
            channel: self.channel.read(cx).value().trim().to_string(),
            location: self.location.read(cx).value().trim().to_string(),
            salary: self.salary.read(cx).value().trim().to_string(),
            applied_at,
            next_action: self.next_action.read(cx).value().trim().to_string(),
            next_action_at,
            notes: self.notes.read(cx).value().trim().to_string(),
            tags: self.tags.clone(),
            stage: self.stage,
        })
    }
}

/// 应用根视图。
pub struct RootView {
    pub store: Store,
    pub data_path: PathBuf,
    pub tab: Tab,
    pub search: Entity<TextInput>,
    pub stage_filter: Option<Stage>,
    pub tag_filter: Option<String>,
    pub selected: Option<Uuid>,
    pub form: FormState,
    pub show_form: bool,
    pub editing: Option<Uuid>,
    pub toast: Option<Toast>,
    pub root_focus: FocusHandle,
    /// 清空数据需要二次确认。
    pub confirm_clear: bool,
}

impl RootView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let data_path = storage::data_path();
        let first_run = !data_path.exists();
        let mut store = storage::load(&data_path);
        let mut toast = None;

        if first_run {
            store = demo::seed_demo(model::today());
            let _ = storage::save(&data_path, &store);
            toast = Some(Toast {
                message: "已写入演示数据，可直接体验统计功能".to_string(),
                kind: ToastKind::Info,
            });
        }

        let selected = store.applications.first().map(|application| application.id);
        let search = cx.new(|cx| TextInput::new("搜索公司 / 岗位 / 渠道 / 备注", cx));
        // 搜索框内容变化时重新渲染根视图，让列表过滤即时生效。
        cx.observe(&search, |_this, _input, cx| cx.notify())
            .detach();
        let form = FormState::new(cx);

        Self {
            store,
            data_path,
            tab: Tab::Dashboard,
            search,
            stage_filter: None,
            tag_filter: None,
            selected,
            form,
            show_form: false,
            editing: None,
            toast,
            root_focus: cx.focus_handle(),
            confirm_clear: false,
        }
    }

    /// 当前搜索词。
    pub fn query(&self, cx: &App) -> String {
        self.search.read(cx).value()
    }

    /// 当前过滤后的投递 id。
    pub fn visible_ids(&self, cx: &App) -> Vec<Uuid> {
        self.store.filter_ids(
            &self.query(cx),
            self.stage_filter,
            self.tag_filter.as_deref(),
        )
    }

    pub fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        self.confirm_clear = false;
        cx.notify();
    }

    pub fn set_stage_filter(&mut self, stage: Option<Stage>, cx: &mut Context<Self>) {
        self.stage_filter = stage;
        cx.notify();
    }

    pub fn set_tag_filter(&mut self, tag: Option<String>, cx: &mut Context<Self>) {
        self.tag_filter = tag;
        cx.notify();
    }

    /// 添加标签到当前表单；空标签或重复标签会被忽略。
    pub fn add_tag(&mut self, tag: String, cx: &mut Context<Self>) {
        let Some(tag) = model::normalize_tag(&tag) else {
            self.set_toast("标签不能为空", ToastKind::Error);
            cx.notify();
            return;
        };
        if self
            .form
            .tags
            .iter()
            .any(|existing| model::tags_equal(existing, &tag))
        {
            self.set_toast(format!("标签「{tag}」已经添加过了"), ToastKind::Info);
            cx.notify();
            return;
        }
        self.form.tags.push(tag.clone());
        self.form.tag_input.update(cx, |input, cx| input.clear(cx));
        self.set_toast(format!("已添加标签「{tag}」"), ToastKind::Success);
        cx.notify();
    }

    /// 从表单输入框读取并添加标签。
    pub fn add_tag_from_input(&mut self, cx: &mut Context<Self>) {
        let value = self.form.tag_input.read(cx).value();
        self.add_tag(value, cx);
    }

    /// 从当前表单移除标签。
    pub fn remove_tag(&mut self, tag: &str, cx: &mut Context<Self>) {
        self.form
            .tags
            .retain(|existing| !model::tags_equal(existing, tag));
        cx.notify();
    }

    pub fn select(&mut self, id: Uuid, cx: &mut Context<Self>) {
        self.selected = Some(id);
        self.confirm_clear = false;
        cx.notify();
    }

    pub fn set_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toast = Some(Toast {
            message: message.into(),
            kind,
        });
    }

    pub fn save_store(&mut self, cx: &mut Context<Self>) {
        match storage::save(&self.data_path, &self.store) {
            Ok(()) => self.set_toast("数据已保存", ToastKind::Success),
            Err(error) => self.set_toast(format!("保存失败：{error}"), ToastKind::Error),
        }
        cx.notify();
    }

    pub fn open_form(
        &mut self,
        editing: Option<Uuid>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_form = true;
        self.editing = editing;
        self.confirm_clear = false;
        match editing.and_then(|id| self.store.get(id)) {
            Some(application) => {
                let application = application.clone();
                self.form.load(&application, cx);
            }
            None => self.form.clear(cx),
        }
        self.form.company.read(cx).focus_handle(cx).focus(window);
        cx.notify();
    }

    pub fn close_form(&mut self, cx: &mut Context<Self>) {
        self.show_form = false;
        self.editing = None;
        cx.notify();
    }

    pub fn submit_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let data = match self.form.to_data(cx) {
            Ok(data) => data,
            Err(error) => {
                self.set_toast(error, ToastKind::Error);
                cx.notify();
                return;
            }
        };

        let today = model::today();
        if let Some(id) = self.editing {
            if let Some(application) = self.store.get_mut(id) {
                application.company = data.company;
                application.position = data.position;
                application.channel = data.channel;
                application.location = data.location;
                application.salary = data.salary;
                application.applied_at = data.applied_at;
                application.next_action = data.next_action;
                application.next_action_at = data.next_action_at;
                application.notes = data.notes;
                application.tags = data.tags;
                if application.stage != data.stage {
                    application.set_stage(data.stage, today, "编辑表单更新阶段");
                } else {
                    application.updated_at = today;
                }
            }
            self.set_toast("投递记录已更新", ToastKind::Success);
        } else {
            let mut application = JobApplication::new(data.company, data.position, data.applied_at);
            application.channel = data.channel;
            application.location = data.location;
            application.salary = data.salary;
            application.next_action = data.next_action;
            application.next_action_at = data.next_action_at;
            application.notes = data.notes;
            application.tags = data.tags;
            if data.stage != Stage::Applied {
                application.set_stage(data.stage, today, "创建记录时设置阶段");
            }
            let id = self.store.add(application);
            self.selected = Some(id);
            self.set_toast("已新增投递记录", ToastKind::Success);
        }

        self.store.sort();
        self.show_form = false;
        self.editing = None;
        self.save_store(cx);
        window.focus(&self.focus_handle(cx));
        cx.notify();
    }

    pub fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        if self.store.remove(id) {
            self.selected = self
                .store
                .applications
                .first()
                .map(|application| application.id);
            self.set_toast("已删除该投递记录", ToastKind::Info);
            self.save_store(cx);
        }
    }

    pub fn set_selected_stage(&mut self, stage: Stage, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        if let Some(application) = self.store.get_mut(id) {
            application.set_stage(
                stage,
                model::today(),
                format!("手动切换到 {}", stage.label()),
            );
            self.set_toast(format!("已更新为 {}", stage.label()), ToastKind::Success);
            self.save_store(cx);
        }
    }

    pub fn advance_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        if let Some(application) = self.store.get_mut(id) {
            let next = application.stage.next_progress();
            if application.advance(model::today()) {
                let label = next.map(|stage| stage.label()).unwrap_or("下一阶段");
                self.set_toast(format!("已推进到 {label}"), ToastKind::Success);
                self.save_store(cx);
            } else {
                self.set_toast("当前阶段无法继续推进", ToastKind::Info);
                cx.notify();
            }
        }
    }

    pub fn reset_demo(&mut self, cx: &mut Context<Self>) {
        self.store = demo::seed_demo(model::today());
        self.selected = self
            .store
            .applications
            .first()
            .map(|application| application.id);
        self.confirm_clear = false;
        self.set_toast("已重置为演示数据", ToastKind::Info);
        self.save_store(cx);
    }

    /// 第一次点击进入确认状态，第二次点击才真正清空。
    pub fn request_clear_all(&mut self, cx: &mut Context<Self>) {
        if self.confirm_clear {
            self.clear_all(cx);
        } else {
            self.confirm_clear = true;
            self.set_toast("再次点击“清空所有数据”确认清空", ToastKind::Info);
            cx.notify();
        }
    }

    /// 清空全部投递记录，并写入空的 JSON 文件。
    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.store = Store::default();
        self.selected = None;
        self.stage_filter = None;
        self.tag_filter = None;
        self.confirm_clear = false;
        self.set_toast("已清空所有投递数据", ToastKind::Info);
        self.save_store(cx);
    }

    fn on_new_application(
        &mut self,
        _: &NewApplication,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_form(None, window, cx);
    }

    fn on_add_tag(&mut self, _: &AddTag, window: &mut Window, cx: &mut Context<Self>) {
        if self.show_form
            && self
                .form
                .tag_input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        {
            self.add_tag_from_input(cx);
        }
    }

    fn on_close_form(&mut self, _: &CloseForm, _: &mut Window, cx: &mut Context<Self>) {
        if self.show_form {
            self.close_form(cx);
        }
    }

    fn on_save_form(&mut self, _: &SaveForm, window: &mut Window, cx: &mut Context<Self>) {
        if self.show_form {
            self.submit_form(window, cx);
        }
    }

    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.search.read(cx).focus_handle(cx).focus(window);
        cx.notify();
    }

    fn nav_item(&self, tab: Tab, cx: &Context<Self>) -> Stateful<Div> {
        let active = self.tab == tab;
        let id = SharedString::from(format!("nav-{:?}", tab));
        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(if active {
                theme::sidebar_active()
            } else {
                theme::sidebar_bg()
            })
            .text_color(if active {
                gpui::white()
            } else {
                theme::text_on_dark()
            })
            .cursor_pointer()
            .hover(|style| style.bg(theme::sidebar_hover()))
            .on_click(cx.listener(move |this, _, _window, cx| this.set_tab(tab, cx)))
            .child(div().w(px(18.)).child(tab.icon()))
            .child(
                div()
                    .text_sm()
                    .font_weight(if active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .child(tab.title()),
            )
    }

    fn render_sidebar(&self, cx: &Context<Self>) -> impl IntoElement {
        let analytics = Analytics::compute(&self.store, model::today());
        let overview = &analytics.overview;
        let path = self.data_path.display().to_string();

        div()
            .flex()
            .flex_col()
            .w(px(232.))
            .h_full()
            .bg(theme::sidebar_bg())
            .text_color(theme::text_on_dark())
            .p_4()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .mb_3()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(gpui::white())
                            .child("JobFlow"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::subtle())
                            .child("求职投递与面试阶段统计"),
                    ),
            )
            .child(self.nav_item(Tab::Dashboard, cx))
            .child(self.nav_item(Tab::Applications, cx))
            .child(self.nav_item(Tab::Analytics, cx))
            .child(self.nav_item(Tab::Settings, cx))
            .child(div().my_3().h(px(1.)).w_full().bg(theme::sidebar_hover()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::subtle())
                            .child("当前进度"),
                    )
                    .child(self.sidebar_stat("总投递", overview.total, theme::accent()))
                    .child(self.sidebar_stat("面试中", overview.interviewing, theme::teal()))
                    .child(self.sidebar_stat("已拿 Offer", overview.offers, theme::success()))
                    .child(self.sidebar_stat("待跟进", overview.due_followups, theme::warning()))
                    .child(self.sidebar_stat("需唤醒", overview.stale, theme::purple())),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::subtle())
                            .child("数据文件"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_on_dark())
                            .truncate()
                            .child(path),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::subtle())
                            .mt_1()
                            .child("数据操作在「设置」中"),
                    ),
            )
    }

    fn sidebar_stat(&self, label: &str, value: usize, color: gpui::Hsla) -> Div {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_on_dark())
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child(value.to_string()),
            )
    }

    fn render_topbar(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_4()
            .h(px(72.))
            .px_6()
            .bg(theme::panel())
            .border_b_1()
            .border_color(theme::border())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text())
                            .child(self.tab.title()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::muted())
                            .child(self.tab.subtitle()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .w(px(280.))
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme::panel_alt())
                            .border_1()
                            .border_color(theme::border_strong())
                            .child(div().text_sm().text_color(theme::subtle()).child("搜索"))
                            .child(div().flex_1().min_w(px(0.)).child(self.search.clone())),
                    )
                    .child(primary_button("new-application", "＋ 新增投递").on_click(
                        cx.listener(|this, _, window, cx| this.open_form(None, window, cx)),
                    )),
            )
    }

    fn render_toast(&self, toast: &Toast) -> impl IntoElement {
        let (bg, fg) = match toast.kind {
            ToastKind::Info => (theme::accent_soft(), theme::accent()),
            ToastKind::Success => (theme::success_soft(), theme::success()),
            ToastKind::Error => (theme::danger_soft(), theme::danger()),
        };
        div()
            .absolute()
            .bottom(px(24.))
            .right(px(24.))
            .px_4()
            .py_3()
            .rounded_md()
            .bg(bg)
            .border_1()
            .border_color(fg)
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(fg)
            .child(toast.message.clone())
    }

    fn form_field(&self, label: &str, input: Entity<TextInput>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .flex_1()
            .min_w(px(0.))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::muted())
                    .child(label.to_string()),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::panel())
                    .border_1()
                    .border_color(theme::border_strong())
                    .child(input),
            )
    }

    fn render_form_overlay(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .bg(gpui::rgba(0x0f172a99))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("form-modal-scroll")
                    .flex()
                    .flex_col()
                    .w(px(760.))
                    .max_h(relative(0.88))
                    .overflow_y_scroll()
                    .bg(theme::panel())
                    .rounded_lg()
                    .shadow_2xl()
                    .p_5()
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
                                            .text_lg()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme::text())
                                            .child(if self.editing.is_some() {
                                                "编辑投递记录"
                                            } else {
                                                "新增投递记录"
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::muted())
                                            .child("阶段变更会自动写入历史，用于漏斗与周期统计"),
                                    ),
                            )
                            .child(
                                secondary_button("close-form", "关闭").on_click(
                                    cx.listener(|this, _, _window, cx| this.close_form(cx)),
                                ),
                            ),
                    )
                    .child(div().h(px(1.)).w_full().bg(theme::border()))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(self.form_field("公司 *", self.form.company.clone()))
                            .child(self.form_field("岗位 *", self.form.position.clone())),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(self.form_field("投递渠道", self.form.channel.clone()))
                            .child(self.form_field("城市", self.form.location.clone()))
                            .child(self.form_field("薪资范围", self.form.salary.clone())),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(self.form_field("投递日期 *", self.form.applied_at.clone()))
                            .child(self.form_field("下一步动作", self.form.next_action.clone()))
                            .child(self.form_field("下一步日期", self.form.next_action_at.clone())),
                    )
                    .child(self.form_field("备注", self.form.notes.clone()))
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
                                    .child("当前阶段"),
                            )
                            .child(div().flex().flex_row().flex_wrap().gap_2().children(
                                Stage::ALL.iter().map(|stage| {
                                    let stage = *stage;
                                    stage_chip(
                                        Some(stage),
                                        self.form.stage == stage,
                                        format!("form-stage-{:?}", stage),
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _window, cx| {
                                            this.form.stage = stage;
                                            cx.notify();
                                        },
                                    ))
                                }),
                            )),
                    )
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
                                    .child("标签 / 分类"),
                            )
                            .when(!self.form.tags.is_empty(), |el| {
                                el.child(div().flex().flex_row().flex_wrap().gap_2().children(
                                    self.form.tags.iter().map(|tag| {
                                        let tag = tag.clone();
                                        let label = format!("{tag} ×");
                                        let remove_tag = tag.clone();
                                        let id = SharedString::from(format!("form-tag-{}", tag));
                                        div()
                                            .id(id)
                                            .px_2()
                                            .py_1()
                                            .rounded_full()
                                            .bg(theme::accent_soft())
                                            .text_xs()
                                            .text_color(theme::accent())
                                            .cursor_pointer()
                                            .hover(|style| style.bg(theme::danger_soft()))
                                            .on_click(cx.listener(move |this, _, _window, cx| {
                                                this.remove_tag(&remove_tag, cx)
                                            }))
                                            .child(label)
                                    }),
                                ))
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::muted())
                                            .child("从已有标签中选择"),
                                    )
                                    .child(
                                        div().flex().flex_row().flex_wrap().gap_2().children(
                                            self.store
                                                .all_tags()
                                                .into_iter()
                                                .filter(|tag| {
                                                    !self.form.tags.iter().any(|existing| {
                                                        model::tags_equal(existing, tag)
                                                    })
                                                })
                                                .map(|tag| {
                                                    let tag_for_click = tag.clone();
                                                    let id = SharedString::from(format!(
                                                        "existing-tag-{}",
                                                        tag
                                                    ));
                                                    div()
                                                        .id(id)
                                                        .px_2()
                                                        .py_1()
                                                        .rounded_full()
                                                        .bg(theme::panel_alt())
                                                        .border_1()
                                                        .border_color(theme::border())
                                                        .text_xs()
                                                        .text_color(theme::muted())
                                                        .cursor_pointer()
                                                        .hover(|style| {
                                                            style.border_color(theme::accent())
                                                        })
                                                        .on_click(cx.listener(
                                                            move |this, _, _window, cx| {
                                                                this.add_tag(
                                                                    tag_for_click.clone(),
                                                                    cx,
                                                                )
                                                            },
                                                        ))
                                                        .child(format!("＋ {tag}"))
                                                }),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_end()
                                    .gap_2()
                                    .child(div().flex_1().min_w(px(0.)).child(
                                        self.form_field("自定义标签", self.form.tag_input.clone()),
                                    ))
                                    .child(primary_button("add-tag", "添加标签").on_click(
                                        cx.listener(|this, _, _window, cx| {
                                            this.add_tag_from_input(cx)
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap_3()
                            .mt_2()
                            .child(
                                secondary_button("cancel-form", "取消").on_click(
                                    cx.listener(|this, _, _window, cx| this.close_form(cx)),
                                ),
                            )
                            .child(primary_button("save-form", "保存记录").on_click(
                                cx.listener(|this, _, window, cx| this.submit_form(window, cx)),
                            )),
                    ),
            )
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.root_focus.clone()
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content: AnyElement = match self.tab {
            Tab::Dashboard => self.render_dashboard(cx).into_any_element(),
            Tab::Applications => self.render_applications(cx).into_any_element(),
            Tab::Analytics => self.render_analytics(cx).into_any_element(),
            Tab::Settings => self.render_settings(cx).into_any_element(),
        };
        let toast = self.toast.clone();

        div()
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme::app_bg())
            .track_focus(&self.root_focus)
            .on_action(cx.listener(Self::on_new_application))
            .on_action(cx.listener(Self::on_close_form))
            .on_action(cx.listener(Self::on_save_form))
            .on_action(cx.listener(Self::on_focus_search))
            .on_action(cx.listener(Self::on_add_tag))
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.))
                    .child(self.render_topbar(cx))
                    .child(
                        div()
                            .id("main-scroll")
                            .flex_1()
                            .min_h(px(0.))
                            .overflow_y_scroll()
                            .child(content),
                    ),
            )
            .when(self.show_form, |el| el.child(self.render_form_overlay(cx)))
            .when_some(toast, |el, toast| el.child(self.render_toast(&toast)))
    }
}
