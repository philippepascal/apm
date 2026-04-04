use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn run_engine_loop(
    root: &Path,
    cancel: Arc<AtomicBool>,
    interval_secs: u64,
    max_concurrent: usize,
    skip_permissions: bool,
    epic_filter: Option<String>,
) -> Result<()> {
    let mut workers: Vec<(String, std::process::Child, std::path::PathBuf)> = Vec::new();
    let mut no_more = false;
    let mut next_poll = Instant::now();

    loop {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let mut reaped = false;
        workers.retain_mut(|(_, child, _pid_path)| {
            let done = matches!(child.try_wait(), Ok(Some(_)));
            if done {
                reaped = true;
            }
            !done
        });

        if reaped {
            next_poll = Instant::now();
            no_more = false;
        }

        if no_more {
            let now = Instant::now();
            if now < next_poll {
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            no_more = false;
        }

        if !no_more && workers.len() < max_concurrent {
            match crate::start::spawn_next_worker(root, true, skip_permissions, epic_filter.as_deref()) {
                Ok(None) => {
                    next_poll = Instant::now() + Duration::from_secs(interval_secs);
                    no_more = true;
                }
                Ok(Some((id, child, pid_path))) => {
                    workers.push((id, child, pid_path));
                    no_more = false;
                }
                Err(_) => {
                    no_more = true;
                    std::thread::sleep(Duration::from_secs(30));
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_flag_stops_loop_immediately() {
        let cancel = Arc::new(AtomicBool::new(true));
        let root = std::path::Path::new("/nonexistent");
        let result = run_engine_loop(root, cancel, 30, 1, false, None);
        assert!(result.is_ok());
    }
}
