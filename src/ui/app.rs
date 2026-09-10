//! 应用根视图：侧边栏、顶部栏、标签页切换与表单。

use std::{path::PathBuf, rc::Rc};

use chrono::NaiveDate;
use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, actions, div, prelude::*, px,
};
use gpui_component::{
    Icon, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    calendar::{Date, Matcher},
    date_picker::{DatePicker, DatePickerState},
    dialog::DialogButtonProps,
    input::{Input, InputEvent, InputState},
    notification::Notification,
    WindowExt as _,
};
use uuid::Uuid;

use crate::{
    demo,
    model::{self, JobApplication, Stage, Store},
    stats::Analytics,
    storage,
    ui::{components::*, theme},
};

actions!(
    job_tracker,
    [
        NewApplication,
        SaveForm,
        CloseForm,
        FocusSearch,
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

    pub fn icon(self) -> IconName {
        match self {
            Tab::Dashboard => IconName::LayoutDashboard,
            Tab::Applications => IconName::FolderClosed,
            Tab::Analytics => IconName::ChartPie,
            Tab::Settings => IconName::Settings,
        }
    }
}

/// 提示消息的类型（映射到组件库的 Notification 级别）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

/// 新增/编辑表单状态。
pub struct FormState {
    pub company: Entity<InputState>,
    pub position: Entity<InputState>,
    pub channel: Entity<InputState>,
    pub location: Entity<InputState>,
    pub salary: Entity<InputState>,
    /// 投递日期用日历选择（只能选到今天及以前）。
    pub applied_at: Entity<DatePickerState>,
    pub next_action: Entity<InputState>,
    /// 下一步日期用日历选择（可留空、可选未来）。
    pub next_action_at: Entity<DatePickerState>,
    pub notes: Entity<InputState>,
    pub tag_input: Entity<InputState>,
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

/// 建一个单行输入框（组件库的 InputState 需要 window 做 IME 初始化）。
fn new_input(window: &mut Window, cx: &mut App, placeholder: &str) -> Entity<InputState> {
    let placeholder = placeholder.to_string();
    cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
}

/// 建一个日历选择器。
///
/// `no_future` 为真时，未来的日期在日历里不可选（投递日期不允许晚于今天）；
/// `with_default` 为真时默认选中今天，否则留空。
fn new_date_picker(
    window: &mut Window,
    cx: &mut App,
    default: Option<NaiveDate>,
    no_future: bool,
) -> Entity<DatePickerState> {
    cx.new(move |cx| {
        let mut state = DatePickerState::new(window, cx).date_format(model::DATE_FORMAT);
        if no_future {
            let today = model::today();
            state = state.disabled_matcher(Matcher::custom(move |date| *date > today));
        }
        if let Some(date) = default {
            state.set_date(date, window, cx);
        }
        state
    })
}

impl FormState {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        let today = model::today();
        Self {
            company: new_input(window, cx, "例如：星海科技"),
            position: new_input(window, cx, "例如：Rust 后端工程师"),
            channel: new_input(window, cx, "BOSS直聘 / 内推 / 官网 ..."),
            location: new_input(window, cx, "例如：上海"),
            salary: new_input(window, cx, "例如：30-45K"),
            applied_at: new_date_picker(window, cx, Some(today), true),
            next_action: new_input(window, cx, "例如：准备二面"),
            next_action_at: new_date_picker(window, cx, None, false),
            notes: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("备注：面试反馈、联系人、薪资细节 ...")
                    .multi_line(true)
                    .rows(3)
            }),
            tag_input: new_input(window, cx, "输入自定义标签，回车或点击添加"),
            tags: Vec::new(),
            stage: Stage::Applied,
        }
    }

    /// 清空表单，准备新增。
    pub fn clear(&mut self, window: &mut Window, cx: &mut App) {
        for input in [
            &self.company,
            &self.position,
            &self.channel,
            &self.location,
            &self.salary,
            &self.next_action,
            &self.notes,
            &self.tag_input,
        ] {
            input.update(cx, |input, cx| input.set_value("", window, cx));
        }
        self.applied_at
            .update(cx, |state, cx| state.set_date(model::today(), window, cx));
        self.next_action_at
            .update(cx, |state, cx| state.set_date(Date::Single(None), window, cx));
        self.tags.clear();
        self.stage = Stage::Applied;
    }

    /// 载入一条已有记录，准备编辑。
    pub fn load(&mut self, application: &JobApplication, window: &mut Window, cx: &mut App) {
        let pairs: [(&Entity<InputState>, String); 7] = [
            (&self.company, application.company.clone()),
            (&self.position, application.position.clone()),
            (&self.channel, application.channel.clone()),
            (&self.location, application.location.clone()),
            (&self.salary, application.salary.clone()),
            (&self.next_action, application.next_action.clone()),
            (&self.notes, application.notes.clone()),
        ];
        for (input, value) in pairs {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        let applied_at = application.applied_at;
        self.applied_at
            .update(cx, |state, cx| state.set_date(applied_at, window, cx));
        let next_action_at = application.next_action_at;
        self.next_action_at.update(cx, |state, cx| {
            state.set_date(
                next_action_at.map_or(Date::Single(None), |date| Date::Single(Some(date))),
                window,
                cx,
            )
        });
        self.tags = application.tags.clone();
        self.tag_input
            .update(cx, |input, cx| input.set_value("", window, cx));
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
        // 日期来自日历选择器，这里只做「有没有选」的判断。
        let applied_at = match self.applied_at.read(cx).date() {
            Date::Single(Some(date)) => date,
            _ => return Err("请选择投递日期".to_string()),
        };
        let next_action_at = match self.next_action_at.read(cx).date() {
            Date::Single(Some(date)) => Some(date),
            _ => None,
        };
        // 投递日期晚于今天会让“最近 30 天 / 月度趋势”等口径对不上，这里直接拦下。
        let today = model::today();
        if applied_at > today {
            return Err(format!("投递日期不能晚于今天（{today}）"));
        }
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
    pub search: Entity<InputState>,
    pub stage_filter: Option<Stage>,
    pub tag_filter: Option<String>,
    pub selected: Option<Uuid>,
    pub form: FormState,
    pub show_form: bool,
    pub editing: Option<Uuid>,
    /// 待推送的提示（render 时交给组件库的 Notification 显示）。
    pub pending_notification: Option<(String, ToastKind)>,
    /// 表单刚打开时需要聚焦的输入框，在 render 里处理。
    pub pending_form_focus: bool,
    pub root_focus: FocusHandle,
    /// 清空数据需要二次确认。
    pub confirm_clear: bool,
    /// 重置为演示数据需要二次确认。
    pub confirm_reset: bool,
    /// 删除记录需要二次确认（存的是待删除的记录 id）。
    pub confirm_delete: Option<Uuid>,
    /// 上次读取数据文件时发现的问题（损坏、逐条恢复等），在设置页展示。
    pub load_notices: Vec<String>,
    /// 上次读取时被跳过的坏记录数。
    pub load_skipped: usize,
    /// 数据文件损坏时为原始文件生成的备份路径。
    pub load_backup: Option<PathBuf>,
    /// 数据文件损坏（禁止任何“写入演示数据”之类的误导提示）。
    pub data_damaged: bool,
    /// 统计数据缓存：(统计日期, 数据版本号, 结果)。
    /// 数据没变时不必每帧重算漏斗与趋势。
    analytics_cache: Option<(NaiveDate, u64, Rc<Analytics>)>,
    /// 每次写入数据后自增，用于让统计缓存失效。
    data_revision: u64,
}

impl RootView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data_path = storage::data_path();
        let first_run = !data_path.exists();
        let outcome = storage::load(&data_path);
        let mut store = outcome.store;
        let mut pending_notification = None;

        // 读取到的问题优先展示：数据损坏时绝不能再悄悄写演示数据。
        if outcome.damaged {
            let message = outcome
                .notices
                .first()
                .cloned()
                .unwrap_or_else(|| "数据文件读取异常".to_string());
            pending_notification = Some((message, ToastKind::Error));
        } else if first_run && store.is_empty() {
            store = demo::seed_demo(model::today());
            match storage::save(&data_path, &store) {
                Ok(()) => {
                    pending_notification = Some((
                        "已写入演示数据，可直接体验统计功能".to_string(),
                        ToastKind::Info,
                    ));
                }
                Err(error) => {
                    pending_notification =
                        Some((format!("写入演示数据失败：{error}"), ToastKind::Error));
                }
            }
        }

        for notice in &outcome.notices {
            eprintln!("{notice}");
        }

        let selected = store.applications.first().map(|application| application.id);
        let search = new_input(
            window,
            cx,
            "搜索公司 / 岗位 / 渠道 / 备注 / 下一步动作",
        );
        // 搜索框内容变化时重新渲染根视图，让列表过滤即时生效。
        cx.subscribe(&search, |_this, _input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        let form = FormState::new(window, cx);

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
            pending_notification,
            pending_form_focus: false,
            root_focus: cx.focus_handle(),
            confirm_clear: false,
            confirm_reset: false,
            confirm_delete: None,
            load_notices: outcome.notices,
            load_skipped: outcome.skipped,
            load_backup: outcome.backup,
            data_damaged: outcome.damaged,
            analytics_cache: None,
            data_revision: 0,
        }
    }

    /// 取统计数据：仅在“数据变了”或“跨天”时重算，其余时间复用缓存。
    /// 返回 `Rc` 句柄，调用方不会长期独占 `self` 的可变借用。
    fn analytics(&mut self) -> Rc<Analytics> {
        let today = model::today();
        let stale = match &self.analytics_cache {
            Some((cached_day, cached_revision, _)) => {
                *cached_day != today || *cached_revision != self.data_revision
            }
            None => true,
        };
        if stale {
            let computed = Rc::new(Analytics::compute(&self.store, today));
            self.analytics_cache = Some((today, self.data_revision, computed));
        }
        self.analytics_cache
            .as_ref()
            .map(|(_, _, analytics)| Rc::clone(analytics))
            .unwrap_or_default()
    }

    /// 数据被修改后调用：让统计缓存失效。
    fn invalidate_analytics(&mut self) {
        self.data_revision = self.data_revision.wrapping_add(1);
    }

    /// 当前搜索词。
    pub fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().to_string()
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
        self.confirm_reset = false;
        self.confirm_delete = None;
        cx.notify();
    }

    pub fn set_stage_filter(&mut self, stage: Option<Stage>, cx: &mut Context<Self>) {
        self.stage_filter = stage;
        self.sync_selection_with_filter(cx);
        cx.notify();
    }

    pub fn set_tag_filter(&mut self, tag: Option<String>, cx: &mut Context<Self>) {
        self.tag_filter = tag;
        self.sync_selection_with_filter(cx);
        cx.notify();
    }

    /// 筛选条件变化后，让选中项始终落在当前可见列表里，
    /// 避免右侧详情面板展示（甚至删除）列表中被过滤掉的记录。
    fn sync_selection_with_filter(&mut self, cx: &App) {
        let visible = self.visible_ids(cx);
        if let Some(id) = self.selected {
            if visible.contains(&id) {
                return;
            }
        }
        self.selected = visible.first().copied();
        self.confirm_delete = None;
    }

    /// 添加标签到当前表单；空标签或重复标签会被忽略。
    pub fn add_tag(&mut self, tag: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tag) = model::normalize_tag(&tag) else {
            self.set_toast("标签不能为空", ToastKind::Error, cx);
            cx.notify();
            return;
        };
        if self
            .form
            .tags
            .iter()
            .any(|existing| model::tags_equal(existing, &tag))
        {
            self.set_toast(format!("标签「{tag}」已经添加过了"), ToastKind::Info, cx);
            cx.notify();
            return;
        }
        self.form.tags.push(tag.clone());
        self.form
            .tag_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.set_toast(format!("已添加标签「{tag}」"), ToastKind::Success, cx);
        cx.notify();
    }

    /// 从表单输入框读取并添加标签。
    pub fn add_tag_from_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.form.tag_input.read(cx).value().to_string();
        self.add_tag(value, window, cx);
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
        self.confirm_delete = None;
        cx.notify();
    }

    /// 设置提示消息；组件库的 Notification 自己负责自动消失。
    pub fn set_toast(
        &mut self,
        message: impl Into<String>,
        kind: ToastKind,
        cx: &mut Context<Self>,
    ) {
        self.pending_notification = Some((message.into(), kind));
        cx.notify();
    }

    /// 把待推送的提示交给组件库的 Notification 列表。
    fn flush_notification(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((message, kind)) = self.pending_notification.take() {
            let notification = match kind {
                ToastKind::Info => Notification::info(message),
                ToastKind::Success => Notification::success(message),
                ToastKind::Error => Notification::error(message),
            }
            .autohide(true);
            window.push_notification(notification, cx);
        }
    }

    /// 落盘并让统计缓存失效。
    ///
    /// 成功时**不**覆盖提示消息：调用方刚刚设置的“已更新/已删除/已备份到 …”更有信息量；
    /// 只有失败才用错误提示盖掉它（保存失败必须让用户看到）。
    pub fn save_store(&mut self, cx: &mut Context<Self>) {
        self.invalidate_analytics();
        if let Err(error) = storage::save(&self.data_path, &self.store) {
            self.set_toast(format!("保存失败：{error}"), ToastKind::Error, cx);
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
                self.form.load(&application, window, cx);
            }
            None => self.form.clear(window, cx),
        }
        // 整体交给组件库的 Dialog：自带遮罩、ESC 关闭与焦点管理。
        self.pending_form_focus = true;
        self.open_form_dialog(window, cx);
        cx.notify();
    }

    /// 用组件库的 Dialog 打开新增/编辑表单。
    fn open_form_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity();
        let editing = self.editing.is_some();
        window.open_dialog(cx, move |dialog, _window, cx| {
            let this = view.read(cx);
            dialog
                .title(if editing {
                    "编辑投递记录"
                } else {
                    "新增投递记录"
                })
                .w(px(760.))
                .child(this.form_dialog_body(&view))
                .footer(|ok, cancel, window, cx| vec![cancel(window, cx), ok(window, cx)])
                .button_props(
                    DialogButtonProps::default()
                        .ok_text("保存记录")
                        .cancel_text("取消"),
                )
                .on_ok({
                    let view = view.clone();
                    move |_, window, cx| view.update(cx, |this, cx| this.submit_form(window, cx))
                })
                .on_cancel({
                    let view = view.clone();
                    move |_, _window, cx| {
                        view.update(cx, |this, cx| this.close_form(cx));
                        true
                    }
                })
        });
    }

    pub fn close_form(&mut self, cx: &mut Context<Self>) {
        self.show_form = false;
        self.editing = None;
        self.pending_form_focus = false;
        cx.notify();
    }

    /// 保存表单。返回 `false` 表示校验失败，Dialog 应保持打开。
    pub fn submit_form(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let data = match self.form.to_data(cx) {
            Ok(data) => data,
            Err(error) => {
                self.set_toast(error, ToastKind::Error, cx);
                cx.notify();
                return false;
            }
        };

        let today = model::today();
        // 没有任何改动时不必重写数据文件。
        let mut dirty = true;
        if let Some(id) = self.editing {
            let mut changed = false;
            let mut missing = false;
            if let Some(application) = self.store.get_mut(id) {
                let stage_changed = application.stage != data.stage;
                changed = application.company != data.company
                    || application.position != data.position
                    || application.channel != data.channel
                    || application.location != data.location
                    || application.salary != data.salary
                    || application.applied_at != data.applied_at
                    || application.next_action != data.next_action
                    || application.next_action_at != data.next_action_at
                    || application.notes != data.notes
                    || application.tags != data.tags
                    || application.stage != data.stage;

                if application.applied_at != data.applied_at {
                    // “已投递”事件的时间就是投递日期，跟着一起改，
                    // 否则面试等待/Offer 周期会按旧日期算。
                    if let Some(event) = application
                        .history
                        .iter_mut()
                        .find(|event| event.stage == Stage::Applied)
                    {
                        event.at = data.applied_at;
                    }
                }

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
                if stage_changed {
                    // set_stage 内部会更新 updated_at。
                    application.set_stage(data.stage, today, "编辑表单更新阶段");
                } else if changed {
                    // 只有真的改了内容才刷新更新时间，避免误清“久未更新”标记。
                    application.updated_at = today;
                }
            } else {
                missing = true;
            }
            dirty = changed && !missing;

            if missing {
                self.set_toast("要编辑的记录已不存在", ToastKind::Error, cx);
            } else if changed {
                self.set_toast("投递记录已更新", ToastKind::Success, cx);
            } else {
                self.set_toast("没有检测到改动", ToastKind::Info, cx);
            }
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
                // 补录历史记录时，阶段事件用投递日期而不是今天，
                // 否则“平均面试等待/Offer 周期”会被算成 0 天。
                application.set_stage(data.stage, data.applied_at, "创建记录时设置阶段");
            }
            let id = self.store.add(application);
            self.selected = Some(id);
            self.set_toast("已新增投递记录", ToastKind::Success, cx);
        }

        self.store.sort();
        self.show_form = false;
        self.editing = None;
        self.pending_form_focus = false;
        if dirty {
            self.save_store(cx);
        }
        window.close_dialog(cx);
        window.focus(&self.focus_handle(cx));
        cx.notify();
        true
    }

    /// 撤销选中记录的最后一次阶段变更，用于误点“推进/切换阶段”后的恢复。
    pub fn undo_last_stage_change(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        let today = model::today();
        let outcome = self.store.get_mut(id).map(|application| {
            application
                .undo_last_stage_change(today)
                .map(|removed| (removed, application.stage))
        });

        match outcome {
            Some(Some((removed, current))) => {
                self.set_toast(
                    format!("已撤销「{}」，当前阶段回到 {}", removed.label(), current.label()),
                    ToastKind::Info,
                    cx,
                );
                self.save_store(cx);
            }
            Some(None) => {
                self.set_toast("没有可撤销的阶段变更", ToastKind::Info, cx);
                cx.notify();
            }
            None => {
                self.set_toast("记录已不存在", ToastKind::Error, cx);
                cx.notify();
            }
        }
    }

    /// 第一次点击进入确认状态，第二次点击才真正删除。
    pub fn request_delete_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        if self.confirm_delete == Some(id) {
            self.delete_selected(cx);
        } else {
            self.confirm_delete = Some(id);
            self.set_toast("再次点击“确认删除”才会删除该记录", ToastKind::Info, cx);
            cx.notify();
        }
    }

    pub fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            return;
        };
        self.confirm_delete = None;

        // 删除后选中相邻记录（优先当前筛选下的下一条），而不是跳回第一条。
        let visible = self.visible_ids(cx);
        let position = visible.iter().position(|candidate| *candidate == id);
        let next = position.and_then(|index| {
            visible
                .get(index + 1)
                .or_else(|| index.checked_sub(1).and_then(|prev| visible.get(prev)))
                .copied()
        });

        if self.store.remove(id) {
            self.selected = next.or_else(|| self.store.applications.first().map(|a| a.id));
            self.set_toast("已删除该投递记录", ToastKind::Info, cx);
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
            self.set_toast(format!("已更新为 {}", stage.label()), ToastKind::Success, cx);
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
                self.set_toast(format!("已推进到 {label}"), ToastKind::Success, cx);
                self.save_store(cx);
            } else {
                self.set_toast("当前阶段无法继续推进", ToastKind::Info, cx);
                cx.notify();
            }
        }
    }

    /// 第一次点击进入确认状态，第二次点击才用演示数据覆盖现有数据。
    pub fn request_reset_demo(&mut self, cx: &mut Context<Self>) {
        if self.confirm_reset {
            self.reset_demo(cx);
        } else {
            self.confirm_reset = true;
            let hint = if self.store.is_empty() {
                "再次点击“重置为演示数据”确认".to_string()
            } else {
                format!(
                    "这会覆盖现有 {} 条记录（会先自动备份），再次点击确认",
                    self.store.len()
                )
            };
            self.set_toast(hint, ToastKind::Info, cx);
            cx.notify();
        }
    }

    pub fn reset_demo(&mut self, cx: &mut Context<Self>) {
        let backup = self.backup_current("reset");
        self.store = demo::seed_demo(model::today());
        self.selected = self
            .store
            .applications
            .first()
            .map(|application| application.id);
        self.confirm_clear = false;
        self.confirm_reset = false;
        let message = match backup {
            Some(Ok(path)) => format!("已重置为演示数据（原数据备份：{}）", path.display()),
            Some(Err(error)) => format!("已重置为演示数据，但备份失败：{error}"),
            None => "已重置为演示数据".to_string(),
        };
        self.set_toast(message, ToastKind::Info, cx);
        self.save_store(cx);
    }

    /// 覆盖性操作（重置/清空）前先把当前数据另存一份，失败也不阻断操作。
    fn backup_current(&self, tag: &str) -> Option<std::io::Result<PathBuf>> {
        if self.store.is_empty() {
            return None;
        }
        Some(storage::backup(&self.data_path, &self.store, tag))
    }

    /// 第一次点击进入确认状态，第二次点击才真正清空。
    pub fn request_clear_all(&mut self, cx: &mut Context<Self>) {
        if self.confirm_clear {
            self.clear_all(cx);
        } else {
            self.confirm_clear = true;
            let hint = if self.store.is_empty() {
                "数据已经是空的，再次点击确认写入空数据文件".to_string()
            } else {
                format!(
                    "这会清空现有 {} 条记录（会先自动备份），再次点击确认",
                    self.store.len()
                )
            };
            self.set_toast(hint, ToastKind::Info, cx);
            cx.notify();
        }
    }

    /// 清空全部投递记录，并写入空的 JSON 文件。
    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        let backup = self.backup_current("clear");
        self.store = Store::default();
        self.selected = None;
        self.stage_filter = None;
        self.tag_filter = None;
        self.confirm_clear = false;
        self.confirm_reset = false;
        self.confirm_delete = None;
        let message = match backup {
            Some(Ok(path)) => format!("已清空所有投递数据（原数据备份：{}）", path.display()),
            Some(Err(error)) => format!("已清空所有投递数据，但备份失败：{error}"),
            None => "已清空所有投递数据".to_string(),
        };
        self.set_toast(message, ToastKind::Info, cx);
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
            self.add_tag_from_input(window, cx);
        }
    }

    fn on_close_form(&mut self, _: &CloseForm, window: &mut Window, cx: &mut Context<Self>) {
        if self.show_form {
            self.close_form(cx);
            window.close_dialog(cx);
        }
    }

    fn on_save_form(&mut self, _: &SaveForm, window: &mut Window, cx: &mut Context<Self>) {
        if self.show_form {
            self.submit_form(window, cx);
        }
    }

    fn on_focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        let search = self.search.clone();
        search.update(cx, |state, cx| state.focus(window, cx));
        cx.notify();
    }

    fn nav_item(&self, tab: Tab, cx: &Context<Self>) -> Button {
        nav_button(
            cx,
            format!("nav-{:?}", tab),
            tab.icon(),
            tab.title(),
            self.tab == tab,
        )
        .on_click(cx.listener(move |this, _, _window, cx| this.set_tab(tab, cx)))
    }

    fn render_sidebar(&self, analytics: &Analytics, cx: &Context<Self>) -> impl IntoElement {
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
                            .w(px(300.))
                            .child(
                                Input::new(&self.search)
                                    .cleanable(true)
                                    .prefix(Icon::new(IconName::Search).text_color(theme::subtle())),
                            ),
                    )
                    .child(
                        primary_button("new-application", "新增投递")
                            .icon(Icon::new(IconName::Plus))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_form(None, window, cx)),
                            ),
                    ),
            )
    }

    fn form_field(&self, label: &str, input: Entity<InputState>) -> Div {
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
            .child(Input::new(&input).cleanable(true))
    }

    /// 表单里的日期字段：点击弹出日历选择，不需要手输日期。
    fn date_field(&self, label: &str, state: Entity<DatePickerState>, placeholder: &str) -> Div {
        let placeholder = placeholder.to_string();
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
                DatePicker::new(&state)
                    .placeholder(placeholder)
                    .cleanable(true),
            )
    }

    /// 表单内容（作为组件库 Dialog 的主体渲染）。
    ///
    /// Dialog 的构建闭包只能拿到 `&App`，所以这里的交互回调统一用
    /// `Entity<Self>` 句柄 + `update`，而不是 `cx.listener`。
    fn form_dialog_body(&self, view: &Entity<Self>) -> impl IntoElement {
        let stage_view = view.clone();
        let tag_view = view.clone();
        let store_tags = self.store.all_tags();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .text_color(theme::muted())
                    .child("阶段变更会自动写入历史，用于漏斗与周期统计"),
            )
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
                    .child(self.date_field(
                        "投递日期 *",
                        self.form.applied_at.clone(),
                        "选择投递日期",
                    ))
                    .child(self.form_field("下一步动作", self.form.next_action.clone()))
                    .child(self.date_field(
                        "下一步日期",
                        self.form.next_action_at.clone(),
                        "选择日期（可留空）",
                    )),
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
                            let view = stage_view.clone();
                            stage_chip(
                                Some(stage),
                                self.form.stage == stage,
                                format!("form-stage-{:?}", stage),
                            )
                            .on_click(move |_, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.form.stage = stage;
                                    cx.notify();
                                });
                            })
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
                                let remove_tag = tag.clone();
                                let view = tag_view.clone();
                                Button::new(SharedString::from(format!("form-tag-{tag}")))
                                    .xsmall()
                                    .ghost()
                                    .label(format!("{tag} ×"))
                                    .on_click(move |_, _window, cx| {
                                        let tag = remove_tag.clone();
                                        view.update(cx, |this, cx| this.remove_tag(&tag, cx));
                                    })
                            }),
                        ))
                    })
                    .when(!store_tags.is_empty(), |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(theme::subtle())
                                .child("从已有标签中选择"),
                        )
                        .child(div().flex().flex_row().flex_wrap().gap_2().children(
                            store_tags
                                .into_iter()
                                .filter(|tag| {
                                    !self
                                        .form
                                        .tags
                                        .iter()
                                        .any(|existing| model::tags_equal(existing, tag))
                                })
                                .map(|tag| {
                                    let view = tag_view.clone();
                                    let label = tag.clone();
                                    Button::new(SharedString::from(format!("existing-tag-{tag}")))
                                        .xsmall()
                                        .ghost()
                                        .icon(Icon::new(IconName::Plus))
                                        .label(label.clone())
                                        .on_click(move |_, window, cx| {
                                            let tag = label.clone();
                                            view.update(cx, |this, cx| this.add_tag(tag, window, cx));
                                        })
                                }),
                        ))
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap_2()
                            .child(
                                div().flex_1().min_w(px(0.)).child(
                                    self.form_field("自定义标签", self.form.tag_input.clone()),
                                ),
                            )
                            .child(
                                secondary_button("add-tag", "添加标签").on_click({
                                    let view = view.clone();
                                    move |_, window, cx| {
                                        view.update(cx, |this, cx| this.add_tag_from_input(window, cx));
                                    }
                                }),
                            ),
                    ),
            )
    }
}

/// 窗口最外层容器。
///
/// `gpui_component::Root` 只负责渲染内嵌视图，Dialog / Sheet / Notification
/// 这些浮层要由应用自己渲染；而且浮层里的表单需要读取 `RootView`，
/// 如果直接在 `RootView::render` 里渲染就会「自己读自己」而 panic，
/// 所以这里多包一层。
pub struct AppShell {
    view: Entity<RootView>,
}

impl AppShell {
    pub fn new(view: Entity<RootView>) -> Self {
        Self { view }
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(self.view.clone())
            .children(gpui_component::Root::render_dialog_layer(window, cx))
            .children(gpui_component::Root::render_sheet_layer(window, cx))
            .children(gpui_component::Root::render_notification_layer(window, cx))
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.root_focus.clone()
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 每帧只算一次统计，仪表盘 / 阶段统计 / 侧边栏共用。
        let analytics = self.analytics();
        let content: AnyElement = match self.tab {
            Tab::Dashboard => self.render_dashboard(&analytics, cx).into_any_element(),
            Tab::Applications => self.render_applications(cx).into_any_element(),
            Tab::Analytics => self.render_analytics(&analytics, cx).into_any_element(),
            Tab::Settings => self.render_settings(cx).into_any_element(),
        };
        // 待推送的提示交给组件库的 Notification。
        self.flush_notification(window, cx);
        // 表单刚打开时把焦点放到第一个输入框。
        if self.pending_form_focus {
            self.pending_form_focus = false;
            let company = self.form.company.clone();
            company.update(cx, |state, cx| state.focus(window, cx));
        }

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
            .child(self.render_sidebar(&analytics, cx))
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
    }
}
