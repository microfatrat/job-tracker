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
        let interview_days: Vec<i64> = applications
            .iter()
            .filter_map(|application| application.days_to_first_interview())
            .collect();
        let offer_days: Vec<i64> = applications
            .iter()
            .filter_map(|application| application.days_to_offer())
            .collect();
        let this_month = applications
            .iter()
            .filter(|application| {
                application.applied_at.year() == today.year()
                    && application.applied_at.month() == today.month()
            })
            .count();
        let last_30_days = applications
            .iter()
            .filter(|application| application.applied_at >= today - Duration::days(30))
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
            response_rate: ratio(responded, total),
            interview_rate: ratio(interviewed, total),
            offer_rate: ratio(offers, total),
            avg_days_to_interview: average(&interview_days),
            avg_days_to_offer: average(&offer_days),
        };

        let mut stages = Vec::with_capacity(Stage::PROGRESS.len());
        let mut previous_reached = total;
        for stage in Stage::PROGRESS {
            let reached = applications
                .iter()
                .filter(|application| application.reached(stage))
                .count();
            let current = applications
                .iter()
                .filter(|application| application.stage == stage)
                .count();
            let step_conversion = ratio(reached, previous_reached);
            stages.push(StageStat {
                stage,
                reached,
                current,
                conversion: ratio(reached, total),
                step_conversion,
                drop_off: if previous_reached == 0 {
                    0.0
                } else {
                    1.0 - step_conversion
                },
            });
            previous_reached = reached;
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
        activities.sort_by(|a, b| b.at.cmp(&a.at));
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

fn channel_stats(applications: &[JobApplication]) -> Vec<ChannelStat> {
    let mut grouped: BTreeMap<String, ChannelStat> = BTreeMap::new();
    for application in applications {
        let channel = if application.channel.trim().is_empty() {
            "未填写".to_string()
        } else {
            application.channel.trim().to_string()
        };
        let entry = grouped
            .entry(channel.clone())
            .or_insert_with(|| ChannelStat {
                channel,
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
}
