//! 面试阶段统计与漏斗/趋势计算。

use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};

use crate::model::{JobApplication, Stage, Store, month_key, month_label};

/// 总览指标。
#[derive(Clone, Debug, Default)]
pub struct Overview {
    pub total: usize,
    pub active: usize,
    pub interviewing: usize,
    pub offers: usize,
    pub rejected: usize,
    pub withdrawn: usize,
    pub responded: usize,
    pub interviewed: usize,
    pub due_followups: usize,
    pub stale: usize,
    pub applied_last_30_days: usize,
    pub applied_this_month: usize,
    /// 投递日期晚于今天的记录数（手改数据或旧数据里才可能出现），
    /// 这类记录不计入“最近 30 天”，但会单列出来提示用户。
    pub future_applied: usize,
    /// 存在早于投递日期的阶段事件的记录数（补录/日期写错）。
    pub chronology_issues: usize,
    pub response_rate: f32,
    pub interview_rate: f32,
    pub offer_rate: f32,
    pub avg_days_to_interview: Option<f32>,
    pub avg_days_to_offer: Option<f32>,
}

/// 单个漏斗阶段的统计。
#[derive(Clone, Debug)]
pub struct StageStat {
    pub stage: Stage,
    /// 到达过该阶段（含之后阶段）的投递数。
    pub reached: usize,
    /// 上一阶段的到达投递数（第一个阶段为总投递数）。
    pub previous_reached: usize,
    /// 当前正处于该阶段的投递数。
    pub current: usize,
    /// 到达率：reached / total。
    pub conversion: f32,
    /// 相对上一阶段的留存率。
    pub step_conversion: f32,
    /// 相对上一阶段的流失率。
    pub drop_off: f32,
}

/// 月度趋势。
#[derive(Clone, Debug)]
pub struct MonthStat {
    pub key: String,
    pub label: String,
    pub applied: usize,
    pub interviews: usize,
    pub offers: usize,
}

/// 投递渠道统计。
#[derive(Clone, Debug)]
pub struct ChannelStat {
    pub channel: String,
    pub total: usize,
    pub responded: usize,
    pub interviews: usize,
    pub offers: usize,
    pub response_rate: f32,
    pub interview_rate: f32,
    pub offer_rate: f32,
}

/// 全部统计数据。
#[derive(Clone, Debug, Default)]
pub struct Analytics {
    pub overview: Overview,
    pub stages: Vec<StageStat>,
    pub months: Vec<MonthStat>,
    pub channels: Vec<ChannelStat>,
}

impl Analytics {
    pub fn compute(store: &Store, today: NaiveDate) -> Self {
        let applications = &store.applications;
        let total = applications.len();

        macro_rules! count {
            ($predicate:expr) => {
                applications.iter().filter($predicate).count()
            };
        }

        let responded = count!(|application| {
            application
                .max_reached_index()
                .is_some_and(|index| index >= 1)
        });
        let interviewed = count!(|application| application.reached(Stage::Interview1));
        let offers = count!(|application| application.is_offer());
        // 日期倒挂（事件早于投递）的记录不参与平均周期计算，只计数上报。
        let interview_days: Vec<i64> = applications
            .iter()
            .filter_map(|application| application.days_to_first_interview())
            .filter(|days| *days >= 0)
            .collect();
        let offer_days: Vec<i64> = applications
            .iter()
            .filter_map(|application| application.days_to_offer())
            .filter(|days| *days >= 0)
            .collect();
        let this_month = applications
            .iter()
            .filter(|application| {
                application.applied_at.year() == today.year()
                    && application.applied_at.month() == today.month()
            })
            .count();
        // 未来日期不计入“最近 30 天”，否则它会被算成刚投递却不出现在任何月度桶里。
        let last_30_days = applications
            .iter()
            .filter(|application| {
                application.applied_at <= today
                    && application.applied_at >= today - Duration::days(30)
            })
            .count();
        let future_applied = applications
            .iter()
            .filter(|application| application.applied_at > today)
            .count();

        let overview = Overview {
            total,
            active: count!(|application| application.is_active()),
            interviewing: count!(|application| application.is_interviewing()),
            offers,
            rejected: count!(|application| application.stage == Stage::Rejected),
            withdrawn: count!(|application| application.stage == Stage::Withdrawn),
            responded,
            interviewed,
            due_followups: count!(|application| application.follow_up_due(today)),
            stale: count!(|application| {
                application.is_active() && application.updated_at <= today - Duration::days(14)
            }),
            applied_last_30_days: last_30_days,
            applied_this_month: this_month,
            future_applied,
            chronology_issues: count!(|application| application.has_chronology_issue()),
            response_rate: ratio(responded, total),
            interview_rate: ratio(interviewed, total),
            offer_rate: ratio(offers, total),
            avg_days_to_interview: average(&interview_days),
            avg_days_to_offer: average(&offer_days),
        };

        let mut stages = Vec::with_capacity(Stage::PROGRESS.len());
        let mut previous = total;
        for stage in Stage::PROGRESS {
            let reached = applications
                .iter()
                .filter(|application| application.reached(stage))
                .count();
            let current = applications
                .iter()
                .filter(|application| application.stage == stage)
                .count();
            let step_conversion = ratio(reached, previous);
            stages.push(StageStat {
                stage,
                reached,
                previous_reached: previous,
                current,
                conversion: ratio(reached, total),
                step_conversion,
                drop_off: if previous == 0 {
                    0.0
                } else {
                    1.0 - step_conversion
                },
            });
            previous = reached;
        }

        let months = monthly_stats(applications, today, 6);
        let channels = channel_stats(applications);

        Self {
            overview,
            stages,
            months,
            channels,
        }
    }

    /// 最近的活动（阶段事件），按日期倒序。
    pub fn recent_activity(&self, store: &Store, limit: usize) -> Vec<Activity> {
        let mut activities: Vec<Activity> = store
            .applications
            .iter()
            .flat_map(|application| {
                application.history.iter().map(move |event| Activity {
                    application_id: application.id,
                    company: application.company.clone(),
                    position: application.position.clone(),
                    stage: event.stage,
                    at: event.at,
                    note: event.note.clone(),
                })
            })
            .collect();
        activities.sort_by_key(|activity| std::cmp::Reverse(activity.at));
        activities.truncate(limit);
        activities
    }

    /// 即将到期的下一步动作。
    pub fn follow_ups<'a>(
        &self,
        store: &'a Store,
        today: NaiveDate,
        limit: usize,
    ) -> Vec<&'a JobApplication> {
        let mut items: Vec<&JobApplication> = store
            .applications
            .iter()
            .filter(|application| {
                application.is_active()
                    && application
                        .next_action_at
                        .is_some_and(|date| date <= today + Duration::days(14))
                    && !application.next_action.trim().is_empty()
            })
            .collect();
        items.sort_by_key(|application| application.next_action_at);
        items.truncate(limit);
        items
    }
}

/// 一条活动记录。
#[derive(Clone, Debug)]
pub struct Activity {
    pub application_id: uuid::Uuid,
    pub company: String,
    pub position: String,
    pub stage: Stage,
    pub at: NaiveDate,
    pub note: String,
}

fn ratio(part: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        part as f32 / total as f32
    }
}

fn average(values: &[i64]) -> Option<f32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<i64>() as f32 / values.len() as f32)
    }
}

fn monthly_stats(
    applications: &[JobApplication],
    today: NaiveDate,
    count: usize,
) -> Vec<MonthStat> {
    let mut months = Vec::with_capacity(count);
    let mut cursor = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    for _ in 0..count {
        let next = next_month(cursor);
        let applied = applications
            .iter()
            .filter(|application| application.applied_at >= cursor && application.applied_at < next)
            .count();
        let interviews = applications
            .iter()
            .filter(|application| {
                application
                    .first_interview_at()
                    .is_some_and(|date| date >= cursor && date < next)
            })
            .count();
        let offers = applications
            .iter()
            .filter(|application| {
                application
                    .offer_at()
                    .is_some_and(|date| date >= cursor && date < next)
            })
            .count();
        months.push(MonthStat {
            key: month_key(cursor),
            label: month_label(cursor),
            applied,
            interviews,
            offers,
        });
        cursor = previous_month(cursor);
    }
    months.reverse();
    months
}

fn next_month(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

fn previous_month(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 1 {
        (date.year() - 1, 12)
    } else {
        (date.year(), date.month() - 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

/// 按渠道分组统计。
///
/// 分组键做「去首尾空白 + Unicode 小写」归一，所以
/// `BOSS直聘` / `boss直聘` / `Boss直聘 ` 会合并成一组（展示名取第一次出现的写法）；
/// 渠道留空（含纯空格）与手填的「未填写」合并为“未填写”一组。
fn channel_stats(applications: &[JobApplication]) -> Vec<ChannelStat> {
    const UNKNOWN: &str = "未填写";

    let mut grouped: BTreeMap<String, ChannelStat> = BTreeMap::new();
    for application in applications {
        let trimmed = application.channel.trim();
        let (key, display) = if trimmed.is_empty() {
            (UNKNOWN.to_lowercase(), UNKNOWN.to_string())
        } else {
            (trimmed.to_lowercase(), trimmed.to_string())
        };
        let entry = grouped.entry(key).or_insert_with(|| ChannelStat {
            channel: display,
            total: 0,
            responded: 0,
            interviews: 0,
            offers: 0,
            response_rate: 0.0,
            interview_rate: 0.0,
            offer_rate: 0.0,
        });
        entry.total += 1;
        if application
            .max_reached_index()
            .is_some_and(|index| index >= 1)
        {
            entry.responded += 1;
        }
        if application.reached(Stage::Interview1) {
            entry.interviews += 1;
        }
        if application.is_offer() {
            entry.offers += 1;
        }
    }
    let mut channels: Vec<ChannelStat> = grouped
        .into_values()
        .map(|mut stat| {
            stat.response_rate = ratio(stat.responded, stat.total);
            stat.interview_rate = ratio(stat.interviews, stat.total);
            stat.offer_rate = ratio(stat.offers, stat.total);
            stat
        })
        .collect();
    channels.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| a.channel.cmp(&b.channel))
    });
    channels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{JobApplication, Stage, Store};

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn store_with_samples() -> Store {
        let mut store = Store::default();

        let mut a = JobApplication::new("A 公司", "后端工程师", date(2026, 1, 5));
        a.channel = "BOSS直聘".to_string();
        a.set_stage(Stage::ResumeScreening, date(2026, 1, 8), "通过筛选");
        a.set_stage(Stage::Interview1, date(2026, 1, 15), "一面");
        a.set_stage(Stage::Offer, date(2026, 1, 25), "Offer");
        store.add(a);

        let mut b = JobApplication::new("B 公司", "Rust 工程师", date(2026, 2, 1));
        b.channel = "内推".to_string();
        b.set_stage(Stage::ResumeScreening, date(2026, 2, 3), "通过筛选");
        b.set_stage(Stage::Rejected, date(2026, 2, 10), "感谢信");
        store.add(b);

        let mut c = JobApplication::new("C 公司", "全栈工程师", date(2026, 2, 20));
        c.channel = "BOSS直聘".to_string();
        c.set_stage(Stage::Interview1, date(2026, 3, 2), "一面");
        store.add(c);

        store
    }

    #[test]
    fn overview_counts_are_correct() {
        let store = store_with_samples();
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.overview.total, 3);
        assert_eq!(analytics.overview.offers, 1);
        assert_eq!(analytics.overview.rejected, 1);
        assert_eq!(analytics.overview.interviewed, 2);
        assert_eq!(analytics.overview.responded, 3);
        assert!((analytics.overview.offer_rate - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn funnel_reached_counts_are_monotonic() {
        let store = store_with_samples();
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        let reached: Vec<usize> = analytics.stages.iter().map(|stage| stage.reached).collect();
        assert_eq!(reached[0], 3);
        assert_eq!(reached[1], 3);
        assert_eq!(reached[3], 2);
        assert_eq!(reached[7], 1);
        for window in reached.windows(2) {
            assert!(window[0] >= window[1]);
        }
    }

    #[test]
    fn monthly_stats_have_six_months() {
        let store = store_with_samples();
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.months.len(), 6);
        assert_eq!(analytics.months.last().unwrap().key, "2026-03");
    }

    #[test]
    fn average_days_to_interview_is_computed() {
        let store = store_with_samples();
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        // A: 10 天，C: 10 天
        assert_eq!(analytics.overview.avg_days_to_interview, Some(10.0));
    }

    #[test]
    fn stage_stats_carry_previous_reached() {
        let store = store_with_samples();
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.stages[0].previous_reached, analytics.overview.total);
        for window in analytics.stages.windows(2) {
            assert_eq!(window[1].previous_reached, window[0].reached);
        }
    }

    #[test]
    fn channels_are_grouped_case_insensitively() {
        let mut store = Store::default();
        for (name, channel) in [
            ("A", "BOSS直聘"),
            ("B", "boss直聘"),
            ("C", " Boss直聘 "),
            ("D", ""),
            ("E", "   "),
            ("F", "未填写"),
        ] {
            let mut application = JobApplication::new(name, "工程师", date(2026, 1, 1));
            application.channel = channel.to_string();
            store.add(application);
        }

        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.channels.len(), 2, "{:?}", analytics.channels);
        let boss = analytics
            .channels
            .iter()
            .find(|channel| channel.channel.eq_ignore_ascii_case("boss直聘"))
            .expect("大小写不同的写法应合并");
        assert_eq!(boss.total, 3);
        let unknown = analytics
            .channels
            .iter()
            .find(|channel| channel.channel == "未填写")
            .expect("空白渠道应归入未填写");
        assert_eq!(unknown.total, 3);
        assert_eq!(
            analytics.channels.iter().map(|c| c.total).sum::<usize>(),
            analytics.overview.total
        );
    }

    #[test]
    fn future_dated_applications_are_reported_separately() {
        let mut store = Store::default();
        store.add(JobApplication::new("今天", "P", date(2026, 3, 10)));
        store.add(JobApplication::new("昨天", "P", date(2026, 3, 9)));
        store.add(JobApplication::new("下个月", "P", date(2026, 4, 20)));

        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.overview.total, 3);
        assert_eq!(analytics.overview.future_applied, 1);
        assert_eq!(
            analytics.overview.applied_last_30_days, 2,
            "未来日期不该被算成“最近 30 天投递”"
        );
        let month_sum: usize = analytics.months.iter().map(|month| month.applied).sum();
        assert_eq!(month_sum + analytics.overview.future_applied, analytics.overview.total);
    }

    #[test]
    fn chronology_issues_are_excluded_from_averages_but_counted() {
        let mut store = Store::default();
        let mut broken = JobApplication::new("倒挂公司", "P", date(2026, 3, 10));
        broken.set_stage(Stage::Interview1, date(2026, 3, 1), "日期写错");
        store.add(broken);

        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.overview.chronology_issues, 1);
        assert_eq!(
            analytics.overview.avg_days_to_interview, None,
            "负数天数不应悄悄按 0 参与平均"
        );

        let mut healthy = JobApplication::new("正常公司", "P", date(2026, 3, 1));
        healthy.set_stage(Stage::Interview1, date(2026, 3, 6), "一面");
        store.add(healthy);
        let analytics = Analytics::compute(&store, date(2026, 3, 10));
        assert_eq!(analytics.overview.avg_days_to_interview, Some(5.0));
        assert_eq!(analytics.overview.chronology_issues, 1);
    }

    #[test]
    fn empty_store_stays_safe() {
        let analytics = Analytics::compute(&Store::default(), date(2026, 3, 10));
        assert_eq!(analytics.overview.total, 0);
        assert_eq!(analytics.overview.offer_rate, 0.0);
        assert_eq!(analytics.overview.future_applied, 0);
        assert_eq!(analytics.overview.chronology_issues, 0);
        assert_eq!(analytics.months.len(), 6);
        assert_eq!(analytics.stages.len(), 8);
        assert_eq!(analytics.stages[0].previous_reached, 0);
        assert_eq!(analytics.stages[0].drop_off, 0.0);
    }

    #[test]
    fn month_buckets_handle_year_boundary() {
        let mut store = Store::default();
        store.add(JobApplication::new("去年末", "P", date(2025, 12, 31)));
        store.add(JobApplication::new("今年初", "P", date(2026, 1, 1)));

        let analytics = Analytics::compute(&store, date(2026, 1, 15));
        assert_eq!(analytics.months.first().unwrap().key, "2025-08");
        assert_eq!(analytics.months.last().unwrap().key, "2026-01");
        assert_eq!(
            analytics
                .months
                .iter()
                .find(|month| month.key == "2025-12")
                .unwrap()
                .applied,
            1
        );
        assert_eq!(
            analytics
                .months
                .iter()
                .find(|month| month.key == "2026-01")
                .unwrap()
                .applied,
            1
        );
    }
}
