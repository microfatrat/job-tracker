//! 数据持久化：默认写入用户数据目录下的 JSON 文件。

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::model::Store;

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

/// 读取数据；文件不存在或解析失败时返回空仓库，并打印提示。
pub fn load(path: &Path) -> Store {
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Store>(&text) {
            Ok(mut store) => {
                // 兼容旧版本数据：没有 tags 字段时 serde(default) 会补空数组，
                // migrate() 会把 version 提升到 2。
                store.migrate();
                store.sort();
                store
            }
            Err(error) => {
                eprintln!("数据文件解析失败，将使用空数据：{error}");
                Store::default()
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => Store::default(),
        Err(error) => {
            eprintln!("读取数据文件失败，将使用空数据：{error}");
            Store::default()
        }
    }
}

/// 保存数据（先写临时文件再替换，尽量避免写一半损坏）。
pub fn save(path: &Path, store: &Store) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let json = serde_json::to_string_pretty(store)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows 上 rename 不会覆盖已存在的目标文件；先尝试直接替换，
        // 如果目标已存在或被杀毒/索引程序占用，再删除后重试一次。
        if fs::rename(&tmp, path).is_err() {
            let _ = fs::remove_file(path);
            fs::rename(&tmp, path)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(&tmp, path)?;
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    #[test]
    fn storage_round_trip() {
        let path =
            std::env::temp_dir().join(format!("job-tracker-test-{}.json", std::process::id()));
        let store = demo::seed_demo(crate::model::today());
        save(&path, &store).expect("save should succeed");
        let loaded = load(&path);
        assert_eq!(loaded.len(), store.len());
        assert_eq!(
            loaded.applications[0].company,
            store.applications[0].company
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn empty_store_round_trip() {
        let path =
            std::env::temp_dir().join(format!("job-tracker-empty-{}.json", std::process::id()));
        let store = crate::model::Store::default();
        save(&path, &store).expect("save should succeed");
        let loaded = load(&path);
        assert!(loaded.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_loads_empty_store() {
        let path = std::env::temp_dir().join("job-tracker-does-not-exist.json");
        let _ = fs::remove_file(&path);
        let store = load(&path);
        assert!(store.is_empty());
    }

    #[test]
    fn old_data_without_tags_is_migrated() {
        let path =
            std::env::temp_dir().join(format!("job-tracker-old-{}.json", std::process::id()));
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
        let store = load(&path);
        assert_eq!(store.version, 2);
        assert_eq!(store.applications.len(), 1);
        assert!(store.applications[0].tags.is_empty());
        assert_eq!(store.applications[0].company, "旧版本公司");
        let _ = fs::remove_file(&path);
    }
}
