//! Log path resolution for Torchlight Infinite game log.
//!
//! Priority:
//! 1. config.log_path (manual override)
//! 2. config.use_tmp_log → tmp folder (debug: most recent UE_game*.log)
//! 3. Dynamic resolution (Windows: find game process, derive path)

use std::path::{Path, PathBuf};

/// Relative path from game exe to log (Python: exe + "/../../../TorchLight/Saved/Logs/UE_game.log")
#[cfg(target_os = "windows")]
const LOG_RELATIVE: &str = "../../../TorchLight/Saved/Logs/UE_game.log";

#[derive(Debug, Clone, Default)]
pub struct LogPathConfig {
    pub log_path: Option<String>,
    pub use_tmp_log: bool,
}

/// Resolve log path for tmp/debug mode: most recent UE_game*.log in tmp folder.
fn resolve_tmp_log_path(project_root: &Path) -> Option<PathBuf> {
    let tmp_dir = project_root.join("tmp");
    let entries = std::fs::read_dir(&tmp_dir).ok()?;
    let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    for e in entries.flatten() {
        let path = e.path();
        if path.is_file() {
            let name = path.file_name()?.to_str()?;
            if name.starts_with("UE_game") && name.ends_with(".log") {
                let mtime = std::fs::metadata(&path).ok()?.modified().ok()?;
                log_files.push((path, mtime));
            }
        }
    }

    if log_files.is_empty() {
        return None;
    }

    // Prefer UE_game.log, else most recent by mtime
    if let Some((p, _)) = log_files
        .iter()
        .find(|(p, _)| p.file_name().and_then(|n| n.to_str()) == Some("UE_game.log"))
    {
        return Some(p.clone());
    }

    log_files.sort_by(|a, b| b.1.cmp(&a.1));
    Some(log_files.first()?.0.clone())
}

/// Resolve log path dynamically on Windows by finding Torchlight process.
#[cfg(target_os = "windows")]
fn resolve_dynamic_windows() -> Option<PathBuf> {
    use std::process::Command;
        let process_names = [
            "TorchlightInfinite.exe",
            "Torchlight Infinite.exe",
            "TorchlightInfinite-Win64-Shipping.exe",
        ];

        for proc_name in process_names {
            let filter = format!("Name='{}'", proc_name);
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Get-CimInstance Win32_Process -Filter \"{}\" | Select-Object -ExpandProperty ExecutablePath",
                        filter
                    ),
                ])
                .output()
                .ok()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let exe_path = stdout.trim();
            if !exe_path.is_empty() {
                let exe = Path::new(exe_path);
                if let Some(parent) = exe.parent() {
                    let log_path = parent.join(LOG_RELATIVE);
                    if log_path.exists() {
                        return Some(log_path.canonicalize().unwrap_or(log_path));
                    }
                    return Some(log_path);
                }
            }
        }
        None
}

/// Resolve log path dynamically on Linux (Steam common path).
#[cfg(target_os = "linux")]
fn resolve_dynamic_linux() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let candidates = [
            format!(
                "{}/.steam/steam/steamapps/common/Torchlight Infinite/TorchLight/Saved/Logs/UE_game.log",
                home
            ),
            format!(
                "{}/.local/share/Steam/steamapps/common/Torchlight Infinite/TorchLight/Saved/Logs/UE_game.log",
                home
            ),
        ];

        for p in candidates {
            let path = Path::new(&p);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
        None
}

/// Resolve the game log file path.
pub fn resolve_log_path(config: &LogPathConfig, project_root: &Path) -> Option<PathBuf> {
    // 1. Manual override
    if let Some(ref log_path) = config.log_path {
        let trimmed = log_path.trim();
        if !trimmed.is_empty() {
            let p = Path::new(trimmed);
            let path = if p.is_absolute() {
                p.to_path_buf()
            } else {
                project_root.join(trimmed)
            };
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 2. Debug: use tmp folder
    if config.use_tmp_log {
        if let Some(p) = resolve_tmp_log_path(project_root) {
            return Some(p);
        }
    }

    // 3. Dynamic resolution
    #[cfg(target_os = "windows")]
    return resolve_dynamic_windows();

    #[cfg(target_os = "linux")]
    return resolve_dynamic_linux();

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    None
}
