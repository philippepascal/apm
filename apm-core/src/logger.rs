use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Option<Mutex<std::io::BufWriter<std::fs::File>>>> = OnceLock::new();
static AGENT: OnceLock<String> = OnceLock::new();

pub fn init(root: &Path, log_file: &Path, agent: &str) {
    AGENT.get_or_init(|| agent.to_string());
    let path = if log_file.is_absolute() {
        log_file.to_path_buf()
    } else {
        root.join(log_file)
    };
    let file = OpenOptions::new().create(true).append(true).open(&path).ok();
    LOGGER.get_or_init(|| file.map(|f| Mutex::new(std::io::BufWriter::new(f))));
}

pub fn log(action: &str, detail: &str) {
    let Some(Some(mutex)) = LOGGER.get() else { return };
    let agent = AGENT.get().map(|s| s.as_str()).unwrap_or("apm");
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let line = format!("{now} [{agent}] {action} {detail}\n");
    if let Ok(mut writer) = mutex.lock() {
        let _ = writer.write_all(line.as_bytes());
        let _ = writer.flush();
    }
}
