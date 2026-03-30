use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Option<Mutex<std::io::BufWriter<std::fs::File>>>> = OnceLock::new();
static AGENT: OnceLock<String> = OnceLock::new();

pub fn default_log_path(project_name: &str) -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home)
            .join("Library/Logs/apm")
            .join(format!("{project_name}.log"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_STATE_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                std::path::PathBuf::from(home).join(".local/state")
            });
        base.join("apm").join(format!("{project_name}.log"))
    }
}

pub fn resolve_log_path(project_name: &str, override_path: Option<&std::path::Path>) -> std::path::PathBuf {
    if let Some(p) = override_path {
        expand_tilde(p)
    } else {
        default_log_path(project_name)
    }
}

fn expand_tilde(path: &std::path::Path) -> std::path::PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home).join(rest)
    } else {
        path.to_path_buf()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_log_path_contains_apm_and_ends_log() {
        let p = default_log_path("myproject");
        let s = p.to_string_lossy();
        assert!(s.contains("apm"), "path should contain 'apm': {s}");
        assert!(s.ends_with(".log"), "path should end with .log: {s}");
        assert!(s.contains("myproject"), "path should contain project name: {s}");
    }
    #[test]
    fn tilde_expansion() {
        let home = std::env::var("HOME").unwrap();
        let result = expand_tilde(std::path::Path::new("~/foo.log"));
        assert_eq!(result, std::path::PathBuf::from(&home).join("foo.log"));
    }
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
