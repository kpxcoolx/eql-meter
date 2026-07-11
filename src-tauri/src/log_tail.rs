use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Open a log file for shared reading so the game can keep writing (Windows).
pub fn open_shared_read(path: &Path) -> std::io::Result<File> {
    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
        OpenOptions::new()
            .read(true)
            .share_mode(0x00000001 | 0x00000002 | 0x00000004)
            .open(path)
    }

    #[cfg(not(windows))]
    {
        // Re-open each read on Mac/Parallels mounts — cached handles can miss appends.
        File::open(path)
    }
}

pub struct TailHandle {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl TailHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TailHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Tail a log file from the end (or from start if `from_start`).
/// Uses filesystem events when available, and always polls so Parallels
/// / network mounts still update live.
pub fn start_tailing<F>(
    path: PathBuf,
    from_start: bool,
    mut on_line: F,
) -> Result<TailHandle, String>
where
    F: FnMut(String) + Send + 'static,
{
    if !path.exists() {
        return Err(format!("Log file not found: {}", path.display()));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);

    let join = thread::spawn(move || {
        let mut offset = if from_start {
            0u64
        } else {
            std::fs::metadata(&path)
                .map(|m| m.len())
                .unwrap_or(0)
        };

        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )
        .ok();

        if let Some(watcher) = watcher.as_mut() {
            let watch_dir = path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            // Parallels mounts often don't support reliable FSEvents — ignore watch errors.
            let _ = watcher.watch(&watch_dir, RecursiveMode::NonRecursive);
        }

        let mut pending = String::new();
        // Parallels virtual disks need frequent polling; events alone are not enough.
        let poll_every = Duration::from_millis(150);

        while !stop_flag.load(Ordering::SeqCst) {
            while let Ok(Ok(event)) = rx.try_recv() {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Any => {}
                    _ => continue,
                }
            }

            match read_new_bytes(&path, &mut offset, &mut pending) {
                Ok(lines) => {
                    for line in lines {
                        on_line(line);
                    }
                }
                Err(_err) => {
                    // Transient lock / mount hiccup — retry next poll.
                }
            }

            thread::sleep(poll_every);
        }
    });

    Ok(TailHandle {
        stop,
        join: Some(join),
    })
}

fn read_new_bytes(
    path: &Path,
    offset: &mut u64,
    pending: &mut String,
) -> std::io::Result<Vec<String>> {
    let meta = std::fs::metadata(path)?;
    let len = meta.len();

    // Truncated / rotated
    if len < *offset {
        *offset = 0;
        pending.clear();
    }

    if len == *offset {
        return Ok(Vec::new());
    }

    let mut file = open_shared_read(path)?;
    file.seek(SeekFrom::Start(*offset))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    *offset = len;

    let chunk = String::from_utf8_lossy(&buf);
    pending.push_str(&chunk);

    let mut lines = Vec::new();
    while let Some(pos) = pending.find('\n') {
        let mut line = pending[..pos].to_string();
        if line.ends_with('\r') {
            line.pop();
        }
        *pending = pending[pos + 1..].to_string();
        if !line.trim().is_empty() {
            lines.push(line);
        }
    }

    Ok(lines)
}
