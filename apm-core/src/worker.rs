use anyhow::Result;
use std::path::Path;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PidFile {
    pub ticket_id: String,
    pub started_at: String,
}

pub fn read_pid_file(path: &Path) -> Result<(u32, PidFile)> {
    #[derive(serde::Deserialize)]
    struct Raw {
        pid: u32,
        ticket_id: String,
        started_at: String,
    }
    let content = std::fs::read_to_string(path)?;
    let raw: Raw = serde_json::from_str(&content)?;
    Ok((raw.pid, PidFile { ticket_id: raw.ticket_id, started_at: raw.started_at }))
}

pub fn is_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn elapsed_since(started_at: &str) -> String {
    let Ok(started) = chrono::DateTime::parse_from_rfc3339(started_at)
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&started_at.replace('Z', "+00:00"))
        })
    else {
        return "—".to_string();
    };
    let now = chrono::Utc::now();
    let secs = (now.timestamp() - started.timestamp()).max(0) as u64;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h {m}m")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_alive_returns_true_for_current_process() {
        assert!(is_alive(std::process::id()));
    }

    #[test]
    fn is_alive_returns_false_for_dead_pid() {
        assert!(!is_alive(99999999));
    }

    #[test]
    fn read_pid_file_parses_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.pid");
        std::fs::write(&path, r#"{"pid":12345,"ticket_id":"0042","started_at":"2026-01-01T00:00Z"}"#).unwrap();
        let (pid, pf) = read_pid_file(&path).unwrap();
        assert_eq!(pid, 12345);
        assert_eq!(pf.ticket_id, "0042");
    }

    #[test]
    fn elapsed_since_seconds() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::seconds(30))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert!(s.ends_with('s'), "expected seconds, got: {s}");
    }

    #[test]
    fn elapsed_since_minutes() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::minutes(42))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert_eq!(s, "42m");
    }

    #[test]
    fn elapsed_since_hours() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::hours(2) - chrono::Duration::minutes(15))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert_eq!(s, "2h 15m");
    }

    #[test]
    fn elapsed_since_invalid_returns_dash() {
        assert_eq!(elapsed_since("not-a-date"), "—");
    }
}
