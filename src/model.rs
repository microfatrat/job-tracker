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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
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

impl Default for Stage {
    fn default() -> Self {
        Stage::Applied
    }
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

    pub fn days_to_first_interview(&self) -> Option<i64> {
        self.first_interview_at()
            .map(|date| (date - self.applied_at).num_days().max(0))
    }

    pub fn days_to_offer(&self) -> Option<i64> {
        self.offer_at()
            .map(|date| (date - self.applied_at).num_days().max(0))
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

    /// 用于搜索的小写文本。
    pub fn search_blob(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}",
            self.company,
            self.position,
            self.channel,
            self.location,
            self.salary,
            self.notes,
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

/// 规范化标签：去掉首尾空白，空标签返回 None，最长 24 个字符。
pub fn normalize_tag(tag: &str) -> Option<String> {
    let tag = tag.trim();
    if tag.is_empty() {
        return None;
    }
    Some(tag.chars().take(24).collect())
}

/// 标签比较：忽略 ASCII 大小写。
pub fn tags_equal(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn default_version() -> u32 {
    2
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
    /// 数据迁移：旧版本没有 tags 字段，serde(default) 会补成空数组；
    /// 这里把 version 提升到 2，下一次保存就会写成新版格式。
    pub fn migrate(&mut self) {
        if self.version < 2 {
            self.version = 2;
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

    /// 所有投递记录中出现过的标签（去重，按字母/拼音顺序）。
    pub fn all_tags(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut tags = Vec::new();
        for application in &self.applications {
            for tag in &application.tags {
                let key = tag.to_lowercase();
                if seen.insert(key) {
                    tags.push(tag.clone());
                }
            }
        }
        tags.sort_by_key(|tag| tag.to_lowercase());
        tags
    }

    /// 标签使用次数统计，按次数从高到低排序。
    pub fn tag_counts(&self) -> Vec<(String, usize)> {
        let mut counts: BTreeMap<String, (String, usize)> = BTreeMap::new();
        for application in &self.applications {
            for tag in &application.tags {
                let key = tag.to_lowercase();
                counts
                    .entry(key)
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert_with(|| (tag.clone(), 1));
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
}
