//! 数据持久化：默认写入用户数据目录下的 JSON 文件。
//!
//! 读取侧的设计原则是“永不静默丢数据”：
//! - 数据文件不存在 → 空仓库，不报警；
//! - 能整体解析 → 直接使用；
//! - 整体解析失败 → 先把原始文件原样备份为 `名字.bad-时间戳.json`，
//!   再逐条抢救可以解析的记录，并把损坏情况通过 [`LoadOutcome::notices`]
//!   上报给命令行和图形界面（图形界面用 toast 提示）。

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::model::{JobApplication, Store};

/// 数据文件路径。
///
/// 优先级：
/// 1. 环境变量 JOB_TRACKER_DATA
/// 2. 平台数据目录
///    - Linux: XDG_DATA_HOME 或 ~/.local/share
///    - Windows: %APPDATA% 或 %LOCALAPPDATA%
///    - macOS: ~/Library/Application Support
/// 3. 当前目录下的 job_tracker_data.json
pub fn data_path() -> PathBuf {
    if let Some(path) = env::var_os("JOB_TRACKER_DATA") {
        return PathBuf::from(path);
    }

    if let Some(dir) = platform_data_dir() {
        return dir.join("job-tracker").join("applications.json");
    }

    PathBuf::from("job_tracker_data.json")
}

fn platform_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(dir) = env::var_os("XDG_DATA_HOME") {
            if !dir.is_empty() {
                return Some(PathBuf::from(dir));
            }
        }
        env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
    }

    #[cfg(target_os = "macos")]
    {
        env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        })
    }

    #[cfg(target_os = "windows")]
    {
        env::var_os("APPDATA")
            .or_else(|| env::var_os("LOCALAPPDATA"))
            .map(PathBuf::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
    }
}

/// 一次读取的结果：数据本身，以及需要提示给用户的问题。
#[derive(Debug, Default)]
pub struct LoadOutcome {
    pub store: Store,
    /// 需要展示给用户的提示（解析失败、逐条恢复、备份路径等）。
    pub notices: Vec<String>,
    /// 文件中无法解析、被跳过的记录数。
    pub skipped: usize,
    /// 解析失败时为原始文件生成的备份路径。
    pub backup: Option<PathBuf>,
    /// 文件存在但不可用（损坏或读不了）。调用方据此避免给出
    /// “没有数据，跑 --seed 吧”这类会覆盖可恢复数据的提示。
    pub damaged: bool,
}

/// 读取数据并返回详细结果（推荐入口）。
pub fn load(path: &Path) -> LoadOutcome {
    let mut outcome = LoadOutcome::default();

    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        // 文件不存在 = 首次运行，属于正常情况，不报警。
        Err(error) if error.kind() == io::ErrorKind::NotFound => return outcome,
        Err(error) => {
            outcome.damaged = true;
            outcome.notices.push(format!(
                "读取数据文件失败：{error}（路径：{}）。为避免覆盖原始文件，本次不会写入任何数据。",
                path.display()
            ));
            eprintln!("读取数据文件失败：{error}");
            return outcome;
        }
    };

    // 1) 先尝试整体解析，这是正常路径。
    match serde_json::from_str::<Store>(&text) {
        Ok(mut store) => {
            store.migrate();
            store.sort();
            outcome.store = store;
            return outcome;
        }
        Err(error) => {
            outcome.damaged = true;
            outcome.notices.push(format!(
                "数据文件无法整体解析：{error}。原始文件会先备份，再尽量逐条恢复。"
            ));
            eprintln!("数据文件解析失败：{error}");
        }
    }

    // 2) 整体失败 → 先备份原始内容（保证人工可修复），再逐条抢救。
    match backup_corrupted(path, &text) {
        Ok(backup) => {
            outcome.notices.push(format!(
                "已备份原始文件到：{}",
                backup.display()
            ));
            outcome.backup = Some(backup);
        }
        Err(error) => outcome
            .notices
            .push(format!("备份原始文件失败：{error}（请手动复制该文件）")),
    }

    let (store, skipped) = recover_records(&text);
    outcome.store = store;
    outcome.skipped = skipped;
    if skipped > 0 {
        outcome.notices.push(format!(
            "已从损坏文件中恢复 {} 条记录，跳过 {} 条无法解析的记录。",
            outcome.store.len(),
            skipped
        ));
    } else if outcome.store.is_empty() {
        outcome
            .notices
            .push("未能从损坏文件中恢复任何记录，请从备份文件中人工修复。".to_string());
    }

    outcome
}

/// 逐条解析 `applications`，跳过无法解析的记录。
fn recover_records(text: &str) -> (Store, usize) {
    let mut store = Store::default();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return (store, 0);
    };

    if let Some(version) = value.get("version").and_then(|value| value.as_u64()) {
        store.version = version.min(u32::MAX as u64) as u32;
    }

    let mut skipped = 0usize;
    if let Some(items) = value.get("applications").and_then(|value| value.as_array()) {
        for item in items {
            match serde_json::from_value::<JobApplication>(item.clone()) {
                Ok(application) => store.applications.push(application),
                Err(_) => skipped += 1,
            }
        }
    }

    store.migrate();
    store.sort();
    (store, skipped)
}

/// 把当前数据另存为 `名字.<tag>-时间戳.json`（同目录）。
///
/// 用于“重置为演示数据 / 清空所有数据 / --seed --force”这类覆盖性操作前的兜底备份。
pub fn backup(path: &Path, store: &Store, tag: &str) -> io::Result<PathBuf> {
    let json = serde_json::to_string_pretty(store)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let target = unique_sidecar(path, tag)?;

    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&target, format!("{json}\n"))?;
    Ok(target)
}

/// 生成一个同目录、不重名的 `名字.<tag>-时间戳.json` 路径。
fn unique_sidecar(path: &Path, tag: &str) -> io::Result<PathBuf> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "job-tracker".to_string());

    let mut candidate = path.with_file_name(format!("{stem}.{tag}-{stamp}.json"));
    let mut seq = 1u32;
    while candidate.exists() {
        candidate = path.with_file_name(format!("{stem}.{tag}-{stamp}-{seq}.json"));
        seq += 1;
    }
    if let Some(parent) = candidate.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(candidate)
}

/// 把损坏的原始文件另存为 `名字.bad-时间戳.json`（同目录，绝不跨文件系统）。
fn backup_corrupted(path: &Path, text: &str) -> io::Result<PathBuf> {
    let target = unique_sidecar(path, "bad")?;
    fs::write(&target, text)?;
    Ok(target)
}

/// 每次保存都使用独立的临时文件名，避免多个进程/线程互相截断同一个临时文件。
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_path(path: &Path) -> PathBuf {
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "job-tracker".to_string());

    let name = match path.extension().map(|ext| ext.to_string_lossy().to_string()) {
        // applications.json → applications.1234-0.json.tmp（仍然匹配 .gitignore 的 *.json.tmp）
        Some(ext) if !ext.is_empty() => format!("{stem}.{pid}-{seq}.{ext}.tmp"),
        _ => format!("{stem}.{pid}-{seq}.tmp"),
    };
    path.with_file_name(name)
}

/// 保存数据（先写临时文件再原子替换，尽量避免写一半损坏）。
pub fn save(path: &Path, store: &Store) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let json = serde_json::to_string_pretty(store)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tmp = temp_path(path);

    let result = (|| -> io::Result<()> {
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        replace(&tmp, path)?;
        sync_dir(path);
        Ok(())
    })();

    if result.is_err() {
        // 失败时不要留下半截临时文件。
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// 把临时文件替换到目标路径。
fn replace(tmp: &Path, path: &Path) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Windows 上 rename 不会覆盖已存在的目标文件；先尝试直接替换，
        // 如果目标已存在或被杀毒/索引程序占用，再删除后重试一次。
        if fs::rename(tmp, path).is_err() {
            let _ = fs::remove_file(path);
            fs::rename(tmp, path)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(tmp, path)?;
    }

    Ok(())
}

/// 尽力把目录元数据刷到磁盘，让 rename 结果在崩溃后也可恢复（失败不影响保存结果）。
fn sync_dir(path: &Path) {
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(parent) = path.parent() {
            let dir = if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            };
            if let Ok(handle) = fs::File::open(dir) {
                let _ = handle.sync_all();
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let _ = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    /// 每个测试用独立文件，避免并行执行时互相干扰。
    fn temp_file(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "job-tracker-{name}-{}.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn storage_round_trip() {
        let path = temp_file("round-trip");
        let store = demo::seed_demo(crate::model::today());
        save(&path, &store).expect("save should succeed");
        let loaded = load(&path);
        assert!(loaded.notices.is_empty(), "正常数据不应产生提示");
        assert!(!loaded.damaged);
        assert_eq!(loaded.store.len(), store.len());
        assert_eq!(
            loaded.store.applications[0].company,
            store.applications[0].company
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn empty_store_round_trip() {
        let path = temp_file("empty");
        let store = Store::default();
        save(&path, &store).expect("save should succeed");
        assert!(load(&path).store.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_loads_empty_store_without_warning() {
        let path = std::env::temp_dir().join("job-tracker-does-not-exist.json");
        let _ = fs::remove_file(&path);
        let outcome = load(&path);
        assert!(outcome.store.is_empty());
        assert!(!outcome.damaged, "首次运行不应被标记为数据损坏");
        assert!(outcome.notices.is_empty());
    }

    #[test]
    fn old_data_without_tags_is_migrated() {
        let path = temp_file("old-version");
        let old_json = r#"{
            "version": 1,
            "applications": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "company": "旧版本公司",
                    "position": "Rust 工程师",
                    "applied_at": "2026-01-01",
                    "stage": "Applied",
                    "updated_at": "2026-01-01"
                }
            ]
        }"#;
        fs::write(&path, old_json).expect("write old data");
        let outcome = load(&path);
        assert!(!outcome.damaged);
        assert_eq!(outcome.store.version, crate::model::CURRENT_VERSION);
        assert_eq!(outcome.store.applications.len(), 1);
        assert!(outcome.store.applications[0].tags.is_empty());
        assert_eq!(outcome.store.applications[0].company, "旧版本公司");
        // 旧数据缺少 history 时按投递日期补一条起始事件，漏斗才不会凭空流失。
        assert_eq!(outcome.store.applications[0].history.len(), 1);
        assert_eq!(
            outcome.store.applications[0].history[0].at,
            outcome.store.applications[0].applied_at
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn broken_file_is_backed_up_and_partially_recovered() {
        let path = temp_file("broken");
        let broken = r#"{
            "version": 2,
            "applications": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "company": "好公司",
                    "position": "Rust 工程师",
                    "channel": "内推",
                    "applied_at": "2026-01-01",
                    "stage": "Applied",
                    "updated_at": "2026-01-01"
                },
                {
                    "id": "00000000-0000-0000-0000-000000000002",
                    "company": "坏日期公司",
                    "position": "Rust 工程师",
                    "applied_at": "2026/1/1",
                    "stage": "Applied",
                    "updated_at": "2026-01-01"
                }
            ]
        }"#;
        fs::write(&path, broken).expect("write broken data");

        let outcome = load(&path);
        assert!(outcome.damaged, "整体解析失败必须被标记");
        assert_eq!(outcome.store.len(), 1, "可解析的记录应被抢救出来");
        assert_eq!(outcome.skipped, 1, "坏记录被跳过并计数");
        assert_eq!(outcome.store.applications[0].company, "好公司");

        let backup = outcome.backup.expect("必须生成备份文件");
        assert!(backup.exists(), "备份文件应真实存在");
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            broken,
            "备份必须是原始文件内容"
        );
        assert!(
            outcome.notices.iter().any(|notice| notice.contains("备份")),
            "提示里应说明备份位置：{:?}",
            outcome.notices
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
    }

    #[test]
    fn unparsable_file_is_never_silently_replaced() {
        let path = temp_file("not-json");
        let original = "{ 这不是 JSON";
        fs::write(&path, original).expect("write garbage");

        let outcome = load(&path);
        assert!(outcome.damaged);
        assert!(outcome.store.is_empty());
        let backup = outcome.backup.clone().expect("必须生成备份文件");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);

        // 即便用户随后保存（GUI 里任何一次编辑），原始内容也还在备份里。
        save(&path, &outcome.store).expect("save should succeed");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        assert!(!fs::read_to_string(&path).unwrap().contains("这不是"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
    }

    #[test]
    fn concurrent_saves_do_not_corrupt_the_file() {
        use std::sync::Arc;

        let path = Arc::new(temp_file("concurrent"));
        let mut handles = Vec::new();
        for index in 0..4u32 {
            let path = path.clone();
            handles.push(std::thread::spawn(move || {
                let mut store = Store::default();
                for i in 0..25u32 {
                    let mut application = JobApplication::new(
                        format!("并发公司-{index}-{i}"),
                        "工程师",
                        crate::model::today(),
                    );
                    // 让每个线程的数据量不同，放大互相截断的机会。
                    application.notes = "x".repeat(2_000 * (index as usize + 1));
                    store.add(application);
                }
                for _ in 0..8 {
                    save(&path, &store).expect("并发保存不应失败");
                }
            }));
        }
        for handle in handles {
            handle.join().expect("线程不应 panic");
        }

        let outcome = load(&path);
        assert!(!outcome.damaged, "并发保存后文件必须是合法 JSON");
        assert_eq!(outcome.store.len(), 25);

        // 不应该留下任何临时文件。
        let dir = path.parent().unwrap();
        let leftovers: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.contains(&format!("{}", std::process::id())) && name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "残留临时文件：{leftovers:?}");

        let _ = fs::remove_file(path.as_path());
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = std::env::temp_dir().join(format!("job-tracker-nested-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("nested").join("applications.json");
        save(&path, &Store::default()).expect("should create parents");
        assert!(path.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
