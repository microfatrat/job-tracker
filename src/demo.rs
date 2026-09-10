//! 首次运行时写入的演示数据，让统计面板一打开就有内容。

use chrono::{Duration, NaiveDate};

use crate::model::{JobApplication, Stage, Store};

/// 生成演示数据。
pub fn seed_demo(today: NaiveDate) -> Store {
    let mut store = Store::default();

    let records: Vec<DemoRecord> = vec![
        DemoRecord {
            company: "星海科技",
            position: "Rust 后端工程师",
            channel: "BOSS直聘",
            location: "上海",
            salary: "30-45K",
            applied_days_ago: 96,
            stages: vec![
                (Stage::ResumeScreening, 2),
                (Stage::WrittenTest, 6),
                (Stage::Interview1, 12),
                (Stage::Interview2, 18),
                (Stage::HrInterview, 25),
                (Stage::Offer, 31),
            ],
            next_action: "确认 Offer 薪资与入职时间",
            next_action_in: 2,
            notes: "两轮技术面反馈很好，HR 说审批已通过。",
        },
        DemoRecord {
            company: "云图数据",
            position: "分布式存储工程师",
            channel: "内推",
            location: "北京",
            salary: "35-55K",
            applied_days_ago: 84,
            stages: vec![
                (Stage::ResumeScreening, 1),
                (Stage::Interview1, 8),
                (Stage::Interview2, 15),
                (Stage::Rejected, 22),
            ],
            next_action: "",
            next_action_in: 0,
            notes: "二面架构题回答不够深入，已收到感谢信。",
        },
        DemoRecord {
            company: "澜舟智能",
            position: "AI 平台工程师",
            channel: "猎聘",
            location: "杭州",
            salary: "40-60K",
            applied_days_ago: 72,
            stages: vec![
                (Stage::ResumeScreening, 3),
                (Stage::WrittenTest, 7),
                (Stage::Interview1, 14),
                (Stage::Interview2, 21),
                (Stage::HrInterview, 28),
            ],
            next_action: "准备 HR 面：离职原因与期望薪资",
            next_action_in: 1,
            notes: "团队做推理平台，技术栈匹配度高。",
        },
        DemoRecord {
            company: "极光互动",
            position: "游戏服务端工程师",
            channel: "官网",
            location: "深圳",
            salary: "25-40K",
            applied_days_ago: 66,
            stages: vec![
                (Stage::ResumeScreening, 4),
                (Stage::WrittenTest, 9),
                (Stage::Rejected, 16),
            ],
            next_action: "",
            next_action_in: 0,
            notes: "笔试算法题没做完，流程终止。",
        },
        DemoRecord {
            company: "晨曦软件",
            position: "全栈工程师",
            channel: "BOSS直聘",
            location: "成都",
            salary: "20-32K",
            applied_days_ago: 58,
            stages: vec![(Stage::ResumeScreening, 2), (Stage::Interview1, 10)],
            next_action: "等待一面结果，3 天后可跟进",
            next_action_in: 3,
            notes: "面试官对 Side Project 很感兴趣。",
        },
        DemoRecord {
            company: "远山网络",
            position: "后端开发工程师",
            channel: "拉勾",
            location: "广州",
            salary: "22-35K",
            applied_days_ago: 51,
            stages: vec![(Stage::ResumeScreening, 5)],
            next_action: "简历仍在筛选，礼貌跟进一次",
            next_action_in: -1,
            notes: "HR 说本周会安排面试。",
        },
        DemoRecord {
            company: "深蓝安全",
            position: "安全研发工程师",
            channel: "内推",
            location: "北京",
            salary: "28-45K",
            applied_days_ago: 44,
            stages: vec![
                (Stage::ResumeScreening, 1),
                (Stage::WrittenTest, 4),
                (Stage::Interview1, 11),
            ],
            next_action: "准备二面：网络安全协议与 Rust 内存安全",
            next_action_in: 4,
            notes: "一面重点问了 TLS 与零拷贝。",
        },
        DemoRecord {
            company: "木星云",
            position: "云原生开发工程师",
            channel: "LinkedIn",
            location: "远程",
            salary: "35-50K",
            applied_days_ago: 39,
            stages: vec![
                (Stage::ResumeScreening, 3),
                (Stage::Interview1, 12),
                (Stage::Interview2, 19),
            ],
            next_action: "等待终面安排",
            next_action_in: 6,
            notes: "外企远程岗位，时区友好。",
        },
        DemoRecord {
            company: "光年信息",
            position: "数据平台工程师",
            channel: "官网",
            location: "南京",
            salary: "24-36K",
            applied_days_ago: 33,
            stages: vec![(Stage::ResumeScreening, 2)],
            next_action: "补交作品集链接",
            next_action_in: 0,
            notes: "HR 要求补充 GitHub 与博客链接。",
        },
        DemoRecord {
            company: "青柠科技",
            position: "Rust 工具链工程师",
            channel: "内推",
            location: "上海",
            salary: "32-48K",
            applied_days_ago: 27,
            stages: vec![(Stage::ResumeScreening, 2), (Stage::Interview1, 9)],
            next_action: "整理一面反馈，准备二面项目深挖",
            next_action_in: 2,
            notes: "岗位偏编译器与构建系统，需要补 Cargo 原理。",
        },
        DemoRecord {
            company: "风语者",
            position: "音视频开发工程师",
            channel: "BOSS直聘",
            location: "杭州",
            salary: "26-40K",
            applied_days_ago: 21,
            stages: vec![(Stage::ResumeScreening, 3)],
            next_action: "",
            next_action_in: 0,
            notes: "岗位方向与预期有偏差，继续观察。",
        },
        DemoRecord {
            company: "智子实验室",
            position: "系统软件工程师",
            channel: "猎头",
            location: "北京",
            salary: "45-65K",
            applied_days_ago: 16,
            stages: vec![(Stage::ResumeScreening, 1), (Stage::Interview1, 7)],
            next_action: "确认二面时间",
            next_action_in: 1,
            notes: "做操作系统内核相关，技术挑战大。",
        },
        DemoRecord {
            company: "南山智造",
            position: "嵌入式软件工程师",
            channel: "拉勾",
            location: "深圳",
            salary: "22-34K",
            applied_days_ago: 12,
            stages: vec![(Stage::ResumeScreening, 2)],
            next_action: "等待笔试通知",
            next_action_in: 5,
            notes: "需要复习 C 与 RTOS。",
        },
        DemoRecord {
            company: "白泽科技",
            position: "大模型应用工程师",
            channel: "内推",
            location: "上海",
            salary: "40-60K",
            applied_days_ago: 8,
            stages: vec![(Stage::ResumeScreening, 1)],
            next_action: "准备 RAG 项目讲解",
            next_action_in: 3,
            notes: "内推人已帮忙推到用人经理。",
        },
        DemoRecord {
            company: "海角网络",
            position: "平台工程师",
            channel: "BOSS直聘",
            location: "厦门",
            salary: "20-30K",
            applied_days_ago: 5,
            stages: vec![],
            next_action: "等待简历初筛",
            next_action_in: 4,
            notes: "刚投递，暂无反馈。",
        },
        DemoRecord {
            company: "晨曦软件",
            position: "Node.js 后端工程师",
            channel: "官网",
            location: "成都",
            salary: "18-28K",
            applied_days_ago: 3,
            stages: vec![],
            next_action: "",
            next_action_in: 0,
            notes: "同一家公司第二个岗位，作为备选。",
        },
        DemoRecord {
            company: "极目机器人",
            position: "运动控制工程师",
            channel: "猎聘",
            location: "苏州",
            salary: "25-38K",
            applied_days_ago: 2,
            stages: vec![],
            next_action: "补充项目视频",
            next_action_in: 2,
            notes: "HR 希望先看一段项目演示视频。",
        },
        DemoRecord {
            company: "蓝鲸数据",
            position: "实时计算工程师",
            channel: "LinkedIn",
            location: "远程",
            salary: "35-52K",
            applied_days_ago: 1,
            stages: vec![],
            next_action: "",
            next_action_in: 0,
            notes: "刚完成投递，等待回复。",
        },
    ];

    for record in records {
        let applied_at = today - Duration::days(record.applied_days_ago);
        let mut application = JobApplication::new(record.company, record.position, applied_at);
        application.channel = record.channel.to_string();
        application.location = record.location.to_string();
        application.salary = record.salary.to_string();
        application.notes = record.notes.to_string();
        for (stage, days_after) in record.stages {
            application.set_stage(
                stage,
                applied_at + Duration::days(days_after),
                format!("{}：{}", stage.label(), record.company),
            );
        }
        if !record.next_action.is_empty() {
            application.next_action = record.next_action.to_string();
            application.next_action_at = Some(today + Duration::days(record.next_action_in));
        }

        // 根据渠道、地点、岗位等自动生成演示标签，方便体验标签筛选。
        let mut tags = Vec::new();
        if !application.channel.trim().is_empty() {
            tags.push(application.channel.clone());
        }
        if application.location.contains("远程") {
            tags.push("远程".to_string());
        }
        if application.position.contains("Rust") {
            tags.push("Rust".to_string());
        }
        if application.position.contains("AI") || application.position.contains("大模型") {
            tags.push("AI".to_string());
        }
        if application.salary.contains("45")
            || application.salary.contains("55")
            || application.salary.contains("60")
        {
            tags.push("高薪".to_string());
        }
        if application.stage == Stage::Offer {
            tags.push("Offer".to_string());
        }
        if application.company == "星海科技" || application.company == "澜舟智能" {
            tags.push("目标公司".to_string());
        }
        application.tags = tags;

        store.add(application);
    }

    store.sort();
    store
}

struct DemoRecord {
    company: &'static str,
    position: &'static str,
    channel: &'static str,
    location: &'static str,
    salary: &'static str,
    applied_days_ago: i64,
    stages: Vec<(Stage, i64)>,
    next_action: &'static str,
    next_action_in: i64,
    notes: &'static str,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::today;

    #[test]
    fn demo_data_is_consistent() {
        let store = seed_demo(today());
        assert!(store.len() >= 10, "演示数据应该足够展示统计效果");
        for application in &store.applications {
            assert!(!application.company.trim().is_empty());
            assert!(!application.position.trim().is_empty());
            assert!(application.max_reached_index().is_some());
            assert!(!application.history.is_empty());
            assert!(!application.tags.is_empty(), "演示数据应该带有标签");
            assert!(application.tags.iter().all(|tag| !tag.trim().is_empty()));
        }
    }
}
