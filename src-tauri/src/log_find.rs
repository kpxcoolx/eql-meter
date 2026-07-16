use crate::parse::character_from_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundLog {
    pub path: String,
    pub character: Option<String>,
    pub modified_secs: u64,
    pub size_bytes: u64,
    pub source: String,
}

pub fn candidate_log_dirs() -> Vec<(PathBuf, &'static str)> {
    let mut dirs = Vec::new();

    // Official public install path (Windows)
    if let Ok(public) = std::env::var("PUBLIC") {
        dirs.push((
            PathBuf::from(public)
                .join("Daybreak Game Company")
                .join("Installed Games")
                .join("EverQuest Legends")
                .join("Logs"),
            "windows",
        ));
    }
    dirs.push((
        PathBuf::from(
            r"C:\Users\Public\Daybreak Game Company\Installed Games\EverQuest Legends\Logs",
        ),
        "windows",
    ));

    // Native Mac: osxEQL Wine prefix (Apple Silicon, no VM)
    for wine_logs in osxeql_log_dirs() {
        dirs.push((wine_logs, "osxeql"));
    }

    // Any mounted Windows volume from Parallels / VMware / etc.
    for volume_logs in parallels_log_dirs() {
        dirs.push((volume_logs, "parallels"));
    }

    // Local sample logs while developing
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push((cwd.join("samples"), "sample"));
        if let Some(parent) = cwd.parent() {
            dirs.push((parent.join("samples"), "sample"));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push((parent.join("Logs"), "local"));
            dirs.push((parent.join("samples"), "sample"));
        }
    }

    dirs
}

/// EverQuest Legends Logs folder inside the osxEQL Wine prefix.
pub fn osxeql_log_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return dirs;
    };
    dirs.push(
        home.join("Library/Application Support/osxEQL/prefix/drive_c/users/Public")
            .join("Daybreak Game Company")
            .join("Installed Games")
            .join("EverQuest Legends")
            .join("Logs"),
    );
    dirs
}

/// Find EverQuest Legends Logs folders on mounted Windows disks (Parallels).
pub fn parallels_log_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let known = PathBuf::from(
        "/Volumes/[C] Windows 11.hidden/Users/Public/Daybreak Game Company/Installed Games/EverQuest Legends/Logs",
    );
    dirs.push(known);

    let Ok(volumes) = fs::read_dir("/Volumes") else {
        return dirs;
    };

    for entry in volumes.flatten() {
        let volume = entry.path();
        let logs = volume
            .join("Users/Public/Daybreak Game Company")
            .join("Installed Games")
            .join("EverQuest Legends")
            .join("Logs");
        if !dirs.iter().any(|d| d == &logs) {
            dirs.push(logs);
        }
    }

    dirs
}

pub fn find_eq_logs() -> Vec<FoundLog> {
    let mut found = Vec::new();

    for (dir, source) in candidate_log_dirs() {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_eq_log_file(&path) {
                continue;
            }
            if let Some(mut item) = describe_log(&path) {
                item.source = source.to_string();
                found.push(item);
            }
        }
    }

    found.sort_by(|a, b| {
        let source_rank = |s: &str| match s {
            "osxeql" => 0,
            "parallels" | "windows" => 1,
            "local" => 2,
            _ => 3,
        };
        source_rank(&a.source)
            .cmp(&source_rank(&b.source))
            .then(b.modified_secs.cmp(&a.modified_secs))
            .then(b.size_bytes.cmp(&a.size_bytes))
    });
    found.dedup_by(|a, b| a.path == b.path);
    found
}

pub fn best_log() -> Option<FoundLog> {
    find_eq_logs().into_iter().next()
}

pub fn best_parallels_log() -> Option<FoundLog> {
    find_eq_logs()
        .into_iter()
        .find(|log| log.source == "parallels")
}

fn is_eq_log_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    lower.starts_with("eqlog_") && lower.ends_with(".txt")
}

fn describe_log(path: &Path) -> Option<FoundLog> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let modified_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path_str = path.to_string_lossy().to_string();
    Some(FoundLog {
        character: character_from_path(&path_str),
        path: path_str,
        modified_secs,
        size_bytes: meta.len(),
        source: "unknown".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sample_kenkyo_log() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../samples");
        assert!(path.is_dir());
        let found = find_eq_logs();
        assert!(
            found.iter().any(|f| f.path.contains("eqlog_Kenkyo_freeport")),
            "expected Kenkyo sample in find results: {found:?}"
        );
    }

    #[test]
    fn osxeql_log_dir_uses_application_support_prefix() {
        let dirs = osxeql_log_dirs();
        assert!(
            dirs.iter().any(|d| {
                d.to_string_lossy()
                    .contains("Library/Application Support/osxEQL/prefix/drive_c")
                    && d.ends_with("EverQuest Legends/Logs")
            }),
            "expected osxEQL Wine Logs path: {dirs:?}"
        );
    }
}
