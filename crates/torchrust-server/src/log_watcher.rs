//! Log file watcher - polls game log and processes new content.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::log_path::{resolve_log_path, LogPathConfig};
use crate::state::AppState;

const MIN_POLL_INTERVAL: u64 = 1;
const MAX_POLL_INTERVAL: u64 = 10;
/// Chunk size for tailing (new content). Small is fine.
const READ_BUF_SIZE: usize = 64 * 1024;
/// Larger chunks when catching up on big files (300MB+). Fewer reads, faster.
const CATCHUP_CHUNK_SIZE: usize = 2 * 1024 * 1024;

pub struct LogWatcher {
    stop: Arc<AtomicBool>,
}

impl LogWatcher {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Start the log watcher. Spawns a task that polls the file.
    pub fn spawn(
        state: std::sync::Arc<AppState>,
        config: LogPathConfig,
        poll_interval_sec: u64,
        project_root: std::path::PathBuf,
    ) -> Option<(LogWatcher, PathBuf)> {
        let log_path = resolve_log_path(&config, &project_root)?;

        let interval_sec = poll_interval_sec
            .clamp(MIN_POLL_INTERVAL, MAX_POLL_INTERVAL)
            .max(1);

        let watcher = LogWatcher::new();
        let stop = watcher.stop.clone();
        let path = log_path.clone();

        let use_tmp = config.use_tmp_log;
        tokio::spawn(async move {
            log_watcher_loop(state, log_path, interval_sec, stop, use_tmp).await;
        });

        Some((watcher, path))
    }
}

async fn log_watcher_loop(
    state: Arc<AppState>,
    log_path: PathBuf,
    interval_sec: u64,
    stop: Arc<AtomicBool>,
    use_tmp_log: bool,
) {
    let mut file = match File::open(&log_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Log watcher: cannot open {}: {}", log_path.display(), e);
            return;
        }
    };

    // When using tmp (debug/backup), process from start. Otherwise tail new content only.
    let mut read_position = if use_tmp_log {
        0
    } else {
        file.seek(std::io::SeekFrom::End(0)).await.unwrap_or(0)
    };

    while !stop.load(Ordering::SeqCst) {
        let meta = match file.metadata().await {
            Ok(m) => m,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
                continue;
            }
        };

        let size = meta.len();
        if size < read_position {
            read_position = size;
        }
        if size <= read_position {
            // Caught up (or use_tmp with static file). Wait for new content.
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
            continue;
        }

        if std::env::var("TORCHRUST_LOG_DEBUG").is_ok() {
            eprintln!("Log watcher: new content {} bytes (pos {} -> {})", size - read_position, read_position, size);
        }

        // Use larger chunks when catching up on big files; smaller when tailing.
        let chunk_size = if use_tmp_log && (size - read_position) > READ_BUF_SIZE as u64 {
            CATCHUP_CHUNK_SIZE
        } else {
            READ_BUF_SIZE
        };
        let to_read = (size - read_position).min(chunk_size as u64) as usize;
        let mut buf = vec![0u8; to_read];

        match file.seek(std::io::SeekFrom::Start(read_position)).await {
            Ok(_) => {}
            Err(_) => continue,
        }

        match file.read_exact(&mut buf).await {
            Ok(_) => {
                read_position += to_read as u64;
                if let Ok(text) = String::from_utf8(buf) {
                    if !text.is_empty() {
                        if std::env::var("TORCHRUST_LOG_DEBUG").is_ok() {
                            eprintln!("Log watcher: read {} bytes at pos {}", to_read, read_position - to_read as u64);
                        }
                        if let Err(e) = state.process_log_chunk(&text).await {
                            eprintln!("Log watcher process error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if std::env::var("TORCHRUST_LOG_DEBUG").is_ok() {
                    eprintln!("Log watcher read error: {}", e);
                }
            }
        }

        // When catching up (use_tmp), don't sleep between chunks. Yield so server stays responsive.
        if use_tmp_log {
            tokio::task::yield_now().await;
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
        }
    }
}
