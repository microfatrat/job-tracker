//! 核心数据模型：投递记录、阶段事件与数据仓库。

use std::collections::{BTreeMap, HashSet};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 日期统一使用 YYYY-MM-DD 文本格式输入/展示。
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// 求职流程阶段。
///
/// 前 8 个是线性推进的“进度阶段”，用于漏斗与转化率计算；
/// Rejected / Withdrawn 是终止状态，会保留之前到达的阶段历史。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    #[default]
    Applied,
    ResumeScreening,
    WrittenTest,
    Interview1,
    Interview2,
    Interview3,
    HrInterview,
    Offer,
    Rejected,
    Withdrawn,
}

impl Stage {
    /// 线性进度阶段（不包含终止状态）。
    pub const PROGRESS: [Stage; 8] = [
        Stage::Applied,
        Stage::ResumeScreening,
        Stage::WrittenTest,
        Stage::Interview1,
        Stage::Interview2,
        Stage::Interview3,
        Stage::HrInterview,
        Stage::Offer,
    ];

    /// 全部阶段，用于筛选器展示。
    pub const ALL: [Stage; 10] = [
        Stage::Applied,
        Stage::ResumeScreening,
        Stage::WrittenTest,
        Stage::Interview1,
        Stage::Interview2,
        Stage::Interview3,
        Stage::HrInterview,
        Stage::Offer,
        Stage::Rejected,
        Stage::Withdrawn,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stage::Applied => "已投递",
            Stage::ResumeScreening => "简历筛选",
            Stage::WrittenTest => "笔试/测评",
            Stage::Interview1 => "一面",
            Stage::Interview2 => "二面",
            Stage::Interview3 => "三面/终面",
            Stage::HrInterview => "HR 面",
            Stage::Offer => "Offer",
            Stage::Rejected => "已拒绝",
            Stage::Withdrawn => "已放弃",
        }
    }

    /// 在 PROGRESS 中的下标；终止状态返回 None。
    pub fn progress_index(self) -> Option<usize> {
        Stage::PROGRESS.iter().position(|stage| *stage == self)
    }

    pub fn is_interview(self) -> bool {
        matches!(
            self,
            Stage::Interview1 | Stage::Interview2 | Stage::Interview3 | Stage::HrInterview
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Stage::Rejected | Stage::Withdrawn)
    }

    /// 仍在推进中的投递（未拒绝、未放弃）。
    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }

    /// 下一个线性阶段；若已是 Offer 则返回 None。
    pub fn next_progress(self) -> Option<Stage> {
        let index = self.progress_index()?;
        Stage::PROGRESS.get(index + 1).copied()
    }
}

/// 阶段变更事件，用于精确统计“到达过哪些阶段”和周期天数。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageEvent {
    pub stage: Stage,
    pub at: NaiveDate,
    #[serde(default)]
    pub note: String,
}

/// 一条投递记录。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobApplication {
    pub id: Uuid,
    pub company: String,
    pub position: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub salary: String,
    pub applied_at: NaiveDate,
    pub stage: Stage,
    pub updated_at: NaiveDate,
    #[serde(default)]
    pub next_action: String,
    #[serde(default)]
    pub next_action_at: Option<NaiveDate>,
    #[serde(default)]
    pub notes: String,
    /// 自定义标签 / 分类，例如「Rust」「远程」「目标公司」。
    /// serde(default) 保证旧版本没有 tags 字段的数据仍能正常读取。
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub history: Vec<StageEvent>,
}

impl JobApplication {
    pub fn new(
        company: impl Into<String>,
        position: impl Into<String>,
        applied_at: NaiveDate,
    ) -> Self {
        let company = company.into();
        let position = position.into();
        Self {
            id: Uuid::new_v4(),
            company,
            position,
            channel: String::new(),
            location: String::new(),
            salary: String::new(),
            applied_at,
            stage: Stage::Applied,
            updated_at: applied_at,
            next_action: String::new(),
            next_action_at: None,
            notes: String::new(),
            tags: Vec::new(),
            history: vec![StageEvent {
                stage: Stage::Applied,
                at: applied_at,
                note: "创建投递记录".to_string(),
            }],
        }
    }

    /// 修改当前阶段并记录事件；相同阶段只刷新更新时间。
    pub fn set_stage(&mut self, stage: Stage, at: NaiveDate, note: impl Into<String>) {
        if self.stage == stage {
            self.updated_at = at;
            return;
        }
        self.stage = stage;
        self.updated_at = at;
        self.history.push(StageEvent {
            stage,
            at,
            note: note.into(),
        });
    }

    /// 推进到下一个线性阶段；若已到 Offer 或处于终止状态则返回 false。
    pub fn advance(&mut self, at: NaiveDate) -> bool {
        let Some(next) = self.stage.next_progress() else {
            return false;
        };
        self.set_stage(next, at, format!("推进到 {}", next.label()));
        true
    }

    /// 撤销最后一次阶段变更：弹出最后一条阶段事件，并把当前阶段回退到上一条事件。
    ///
    /// 返回被撤销的阶段；历史里只剩起始事件时返回 `None`
    /// （不允许把记录退回“没有任何历史”的状态）。
    pub fn undo_last_stage_change(&mut self, at: NaiveDate) -> Option<Stage> {
        if self.history.len() <= 1 {
            return None;
        }
        let removed = self.history.pop()?;
        self.stage = self
            .history
            .last()
            .map(|event| event.stage)
            .unwrap_or(Stage::Applied);
        self.updated_at = at;
        Some(removed.stage)
    }

    /// 历史中到达过的最高线性阶段下标。
    pub fn max_reached_index(&self) -> Option<usize> {
        let mut max_index: Option<usize> = None;
        for event in &self.history {
            if let Some(index) = event.stage.progress_index() {
                max_index = Some(max_index.map_or(index, |current: usize| current.max(index)));
            }
        }
        if let Some(index) = self.stage.progress_index() {
            max_index = Some(max_index.map_or(index, |current: usize| current.max(index)));
        }
        max_index
    }

    /// 是否到达过某个线性阶段。
    pub fn reached(&self, stage: Stage) -> bool {
        let Some(target) = stage.progress_index() else {
            return self.stage == stage;
        };
        self.max_reached_index()
            .is_some_and(|index| index >= target)
    }

    /// 第一次进入面试的时间。
    pub fn first_interview_at(&self) -> Option<NaiveDate> {
        self.history
            .iter()
            .filter(|event| event.stage.is_interview())
            .map(|event| event.at)
            .min()
    }

    /// 第一次拿到 Offer 的时间。
    pub fn offer_at(&self) -> Option<NaiveDate> {
        self.history
            .iter()
            .filter(|event| event.stage == Stage::Offer)
            .map(|event| event.at)
            .min()
    }

    /// 投递 → 第一次面试的天数。
    ///
    /// 返回值可能是负数（事件日期早于投递日期），此时数据本身有问题，
    /// 由 [`JobApplication::has_chronology_issue`] 标记出来，统计时会被剔除。
    pub fn days_to_first_interview(&self) -> Option<i64> {
        self.first_interview_at()
            .map(|date| (date - self.applied_at).num_days())
    }

    /// 投递 → 拿到 Offer 的天数（同样可能为负数，见上）。
    pub fn days_to_offer(&self) -> Option<i64> {
        self.offer_at()
            .map(|date| (date - self.applied_at).num_days())
    }

    /// 是否存在早于投递日期的阶段事件（补录/手改日期写错）。
    pub fn has_chronology_issue(&self) -> bool {
        self.history.iter().any(|event| event.at < self.applied_at)
    }

    pub fn is_active(&self) -> bool {
        self.stage.is_active()
    }

    pub fn is_offer(&self) -> bool {
        self.stage == Stage::Offer || self.reached(Stage::Offer)
    }

    pub fn is_interviewing(&self) -> bool {
        self.stage.is_interview()
    }

    /// 用于搜索的小写文本（含公司/岗位/渠道/城市/薪资/备注/下一步动作/标签）。
    pub fn search_blob(&self) -> String {
        format!(
            "{} {} {} {} {} {} {} {}",
            self.company,
            self.position,
            self.channel,
            self.location,
            self.salary,
            self.notes,
            self.next_action,
            self.tags.join(" ")
        )
        .to_lowercase()
    }

    /// 是否存在已到期或今天要做的下一步动作。
    pub fn follow_up_due(&self, today: NaiveDate) -> bool {
        self.is_active()
            && self
                .next_action_at
                .is_some_and(|date| date <= today && !self.next_action.trim().is_empty())
    }
}

/// 标签长度上限（按字符数）。
pub const TAG_MAX_CHARS: usize = 24;

/// 标签的比较键：去首尾空白 + Unicode 小写折叠。
///
/// 标签列表去重、筛选匹配、迁移时的合并统一都用它，
/// 避免出现“列表里合并成一个标签、点它却筛不出记录”的不一致。
pub fn tag_key(tag: &str) -> String {
    tag.trim().to_lowercase()
}

/// 规范化标签：去掉首尾空白，空标签返回 None，最长 [`TAG_MAX_CHARS`] 个字符。
pub fn normalize_tag(tag: &str) -> Option<String> {
    let tag = tag.trim();
    if tag.is_empty() {
        return None;
    }
    Some(tag.chars().take(TAG_MAX_CHARS).collect())
}

/// 标签比较：忽略首尾空白与大小写（Unicode 感知）。
pub fn tags_equal(a: &str, b: &str) -> bool {
    tag_key(a) == tag_key(b)
}

/// 当前数据格式版本。
///
/// - v1 → v2：新增 `tags` 字段；
/// - v2 → v3：补全缺失的 `history`（按投递日期补起始事件）、规范化标签。
pub const CURRENT_VERSION: u32 = 3;

fn default_version() -> u32 {
    CURRENT_VERSION
}

/// 持久化到 JSON 的数据仓库。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Store {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub applications: Vec<JobApplication>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: default_version(),
            applications: Vec::new(),
        }
    }
}

impl Store {
    /// 数据迁移（幂等）。
    ///
    /// - 旧版本没有 `tags` / `history` 字段时 serde 会补成空；
    /// - 缺失 `history` 的记录按投递日期补一条起始事件，
    ///   否则“到达过已投递”会算不出来，漏斗第一行会凭空流失；
    /// - 标签统一规范化（去空白、限长、按 [`tag_key`] 去重）；
    /// - 最后把 `version` 提升到 [`CURRENT_VERSION`]，下次保存即写回新版格式。
    pub fn migrate(&mut self) {
        for application in &mut self.applications {
            if application.history.is_empty() {
                application.history.push(StageEvent {
                    stage: Stage::Applied,
                    at: application.applied_at,
                    note: "创建投递记录".to_string(),
                });
            }

            let mut seen: HashSet<String> = HashSet::new();
            let mut tags: Vec<String> = Vec::with_capacity(application.tags.len());
            for tag in std::mem::take(&mut application.tags) {
                if let Some(tag) = normalize_tag(&tag) {
                    if seen.insert(tag_key(&tag)) {
                        tags.push(tag);
                    }
                }
            }
            application.tags = tags;

            application.history.sort_by_key(|event| event.at);
        }

        if self.version < CURRENT_VERSION {
            self.version = CURRENT_VERSION;
        }
    }

    pub fn len(&self) -> usize {
        self.applications.len()
    }

    pub fn is_empty(&self) -> bool {
        self.applications.is_empty()
    }

    pub fn add(&mut self, application: JobApplication) -> Uuid {
        let id = application.id;
        self.applications.push(application);
        id
    }

    pub fn remove(&mut self, id: Uuid) -> bool {
        let before = self.applications.len();
        self.applications.retain(|application| application.id != id);
        self.applications.len() != before
    }

    pub fn get(&self, id: Uuid) -> Option<&JobApplication> {
        self.applications
            .iter()
            .find(|application| application.id == id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut JobApplication> {
        self.applications
            .iter_mut()
            .find(|application| application.id == id)
    }

    /// 按投递日期倒序排序。
    pub fn sort(&mut self) {
        self.applications.sort_by(|a, b| {
            b.applied_at
                .cmp(&a.applied_at)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
    }

    /// 根据搜索词与阶段过滤，返回匹配的投递 id（保持原顺序）。
    pub fn filter_ids(&self, query: &str, stage: Option<Stage>, tag: Option<&str>) -> Vec<Uuid> {
        let query = query.trim().to_lowercase();
        self.applications
            .iter()
            .filter(|application| {
                stage.is_none_or(|stage| application.stage == stage)
                    && tag.is_none_or(|tag| {
                        application
                            .tags
                            .iter()
                            .any(|candidate| tags_equal(candidate, tag))
                    })
                    && (query.is_empty() || application.search_blob().contains(&query))
            })
            .map(|application| application.id)
            .collect()
    }

    /// 所有投递记录中出现过的标签（按 [`tag_key`] 去重，按字母/拼音顺序）。
    pub fn all_tags(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut tags = Vec::new();
        for application in &self.applications {
            for tag in &application.tags {
                let key = tag_key(tag);
                if !key.is_empty() && seen.insert(key) {
                    tags.push(tag.trim().to_string());
                }
            }
        }
        tags.sort_by_key(|tag| tag_key(tag));
        tags
    }

    /// 标签使用次数统计，按次数从高到低排序。
    pub fn tag_counts(&self) -> Vec<(String, usize)> {
        let mut counts: BTreeMap<String, (String, usize)> = BTreeMap::new();
        for application in &self.applications {
            for tag in &application.tags {
                let key = tag_key(tag);
                if key.is_empty() {
                    continue;
                }
                counts
                    .entry(key)
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert_with(|| (tag.trim().to_string(), 1));
            }
        }
        let mut result: Vec<(String, usize)> = counts.into_values().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        result
    }

    /// 统计当前阶段分布。
    pub fn stage_counts(&self) -> Vec<(Stage, usize)> {
        Stage::ALL
            .iter()
            .map(|stage| {
                (
                    *stage,
                    self.applications
                        .iter()
                        .filter(|application| application.stage == *stage)
                        .count(),
                )
            })
            .collect()
    }
}

/// 今天（本地时区）。
pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// 解析 YYYY-MM-DD 日期。
pub fn parse_date(text: &str) -> Result<NaiveDate, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("日期不能为空".to_string());
    }
    NaiveDate::parse_from_str(text, DATE_FORMAT)
        .map_err(|_| format!("日期格式应为 YYYY-MM-DD：{text}"))
}

/// 解析可选日期；空字符串返回 None。
pub fn parse_optional_date(text: &str) -> Result<Option<NaiveDate>, String> {
    let text = text.trim();
    if text.is_empty() {
        Ok(None)
    } else {
        parse_date(text).map(Some)
    }
}

/// 月份键，例如 2026-09。
pub fn month_key(date: NaiveDate) -> String {
    format!("{:04}-{:02}", date.year(), date.month())
}

/// 月份展示名，例如 9月。
pub fn month_label(date: NaiveDate) -> String {
    format!("{}月", date.month())
}

/// 列表里展示日期：同年只显示 `MM-DD`，跨年补全年份，避免看不出是哪一年。
pub fn format_date_short(date: NaiveDate, today: NaiveDate) -> String {
    if date.year() == today.year() {
        date.format("%m-%d").to_string()
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn tags_are_collected_and_counted() {
        let mut store = Store::default();
        let mut first = JobApplication::new("A 公司", "Rust 工程师", date(2026, 1, 1));
        first.tags = vec!["Rust".into(), "远程".into()];
        let mut second = JobApplication::new("B 公司", "AI 工程师", date(2026, 1, 2));
        second.tags = vec!["rust".into(), "AI".into()];
        store.add(first);
        store.add(second);

        let tags = store.all_tags();
        assert!(tags.iter().any(|tag| tag == "Rust"));
        assert!(tags.iter().any(|tag| tag == "AI"));
        assert_eq!(
            tags.iter()
                .filter(|tag| tag.eq_ignore_ascii_case("rust"))
                .count(),
            1
        );

        let counts = store.tag_counts();
        assert_eq!(
            counts
                .iter()
                .find(|(tag, _)| tag.eq_ignore_ascii_case("rust"))
                .map(|(_, count)| *count),
            Some(2)
        );
    }

    #[test]
    fn filter_by_tag_is_case_insensitive() {
        let mut store = Store::default();
        let mut application = JobApplication::new("A 公司", "Rust 工程师", date(2026, 1, 1));
        application.tags = vec!["Rust".into()];
        store.add(application);

        assert_eq!(store.filter_ids("", None, Some("rust")).len(), 1);
        assert_eq!(store.filter_ids("", None, Some("AI")).len(), 0);
    }

    #[test]
    fn normalize_tag_trims_and_limits_length() {
        assert_eq!(normalize_tag("  Rust  "), Some("Rust".to_string()));
        assert_eq!(normalize_tag("   "), None);
        assert_eq!(
            normalize_tag("123456789012345678901234567890"),
            Some("123456789012345678901234".to_string())
        );
    }

    #[test]
    fn tag_helpers_are_unicode_aware_and_consistent() {
        // 非 ASCII 大小写也要能匹配上，否则列表去重与筛选会自相矛盾。
        assert!(tags_equal("Ä", "ä"));
        assert!(tags_equal("  Rust ", "rust"));
        assert_eq!(tag_key(" Ｒｕｓｔ "), "ｒｕｓｔ");
        assert!(!tags_equal("Rust", "Go"));

        let mut store = Store::default();
        let mut first = JobApplication::new("A", "P", date(2026, 1, 1));
        first.tags = vec!["Ä".into()];
        let mut second = JobApplication::new("B", "P", date(2026, 1, 2));
        second.tags = vec!["ä".into()];
        store.add(first);
        store.add(second);

        let tags = store.all_tags();
        assert_eq!(tags.len(), 1, "两种写法应合并为一个标签：{tags:?}");
        // 列表里展示的标签，点它必须能筛出所有用它标记的记录。
        assert_eq!(store.filter_ids("", None, Some(&tags[0])).len(), 2);
    }

    #[test]
    fn migrate_backfills_history_and_normalizes_tags() {
        let mut store = Store {
            version: 1,
            ..Default::default()
        };
        let mut application = JobApplication::new("旧公司", "工程师", date(2026, 2, 3));
        application.stage = Stage::Rejected;
        application.history.clear();
        application.tags = vec!["  Rust  ".into(), "rust".into(), "   ".into(), "x".repeat(40)];
        store.add(application);

        store.migrate();

        assert_eq!(store.version, CURRENT_VERSION);
        let migrated = &store.applications[0];
        assert_eq!(migrated.history.len(), 1, "缺少 history 时补起始事件");
        assert_eq!(migrated.history[0].at, migrated.applied_at);
        assert_eq!(migrated.tags, vec!["Rust".to_string(), "x".repeat(TAG_MAX_CHARS)]);
        // 补完 history 后，漏斗第一行不会再凭空流失。
        assert!(migrated.reached(Stage::Applied));
        assert_eq!(migrated.max_reached_index(), Some(0));

        // 幂等：再迁移一次结果不变。
        let before = store.clone();
        store.migrate();
        assert_eq!(store.applications[0].tags, before.applications[0].tags);
        assert_eq!(store.applications[0].history.len(), 1);
    }

    #[test]
    fn search_covers_next_action_and_notes() {
        let mut store = Store::default();
        let mut application = JobApplication::new("某公司", "后端工程师", date(2026, 1, 1));
        application.next_action = "准备二面技术问题".into();
        application.notes = "HR 说本周内答复".into();
        let id = store.add(application);

        assert_eq!(store.filter_ids("准备二面", None, None), vec![id]);
        assert_eq!(store.filter_ids("本周内答复", None, None), vec![id]);
        assert!(store.filter_ids("不存在的关键词", None, None).is_empty());
    }

    #[test]
    fn day_deltas_keep_negative_values_and_are_flagged() {
        let mut application = JobApplication::new("倒挂公司", "工程师", date(2026, 3, 10));
        assert_eq!(application.days_to_first_interview(), None);
        assert!(!application.has_chronology_issue());

        application.set_stage(Stage::Interview1, date(2026, 3, 1), "面试日期早于投递");
        assert_eq!(application.days_to_first_interview(), Some(-9));
        assert!(application.has_chronology_issue());

        application.set_stage(Stage::Offer, date(2026, 3, 20), "Offer");
        assert_eq!(application.days_to_offer(), Some(10));
        assert!(
            application.has_chronology_issue(),
            "历史里仍有早于投递日期的事件"
        );
    }

    #[test]
    fn undo_last_stage_change_restores_previous_stage() {
        let mut application = JobApplication::new("手滑公司", "工程师", date(2026, 3, 1));
        assert_eq!(
            application.undo_last_stage_change(date(2026, 3, 2)),
            None,
            "只剩起始事件时不能撤销"
        );

        application.set_stage(Stage::Interview1, date(2026, 3, 5), "一面");
        application.set_stage(Stage::Offer, date(2026, 3, 9), "手滑点成 Offer");
        assert_eq!(application.stage, Stage::Offer);
        assert_eq!(application.history.len(), 3);

        assert_eq!(
            application.undo_last_stage_change(date(2026, 3, 10)),
            Some(Stage::Offer)
        );
        assert_eq!(application.stage, Stage::Interview1);
        assert_eq!(application.history.len(), 2);
        assert_eq!(application.updated_at, date(2026, 3, 10));
        // 撤销后“到达过 Offer”不再成立，Offer 率不会被误算。
        assert!(!application.is_offer());
        assert!(application.reached(Stage::Interview1));

        // 继续撤销回到“已投递”，再撤销就没有可撤销的了。
        assert_eq!(
            application.undo_last_stage_change(date(2026, 3, 10)),
            Some(Stage::Interview1)
        );
        assert_eq!(application.stage, Stage::Applied);
        assert_eq!(application.undo_last_stage_change(date(2026, 3, 10)), None);
    }
}
