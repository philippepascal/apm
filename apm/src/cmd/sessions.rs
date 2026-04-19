use anyhow::Result;
use std::path::Path;

#[derive(serde::Deserialize)]
struct SessionInfo {
    username: String,
    device_hint: Option<String>,
    last_seen: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

pub fn run(root: &Path) -> Result<()> {
    let config = apm_core::config::Config::load(root)?;
    let url = format!("{}/api/auth/sessions", config.server.url);
    match ureq::get(&url).call() {
        Ok(resp) => {
            let sessions: Vec<SessionInfo> = resp
                .into_json()
                .map_err(|e| anyhow::anyhow!("error: invalid response from server: {e}"))?;
            if sessions.is_empty() {
                println!("No active sessions.");
                return Ok(());
            }
            let col_user = sessions.iter().map(|s| s.username.len()).max().unwrap_or(0).max(8);
            let col_device = sessions
                .iter()
                .map(|s| s.device_hint.as_deref().unwrap_or("-").len())
                .max()
                .unwrap_or(0)
                .max(6);
            println!(
                "{:<col_user$}  {:<col_device$}  {:<20}  {}",
                "USERNAME", "DEVICE", "LAST SEEN", "EXPIRES"
            );
            for s in &sessions {
                let device = s.device_hint.as_deref().unwrap_or("-");
                let last_seen = s.last_seen.format("%Y-%m-%d %H:%M UTC").to_string();
                let expires = s.expires_at.format("%Y-%m-%d %H:%M UTC").to_string();
                println!(
                    "{:<col_user$}  {:<col_device$}  {:<20}  {}",
                    s.username, device, last_seen, expires
                );
            }
            Ok(())
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            eprintln!("error: server returned {code}: {body}");
            std::process::exit(1);
        }
        Err(e) => Err(anyhow::anyhow!("error: cannot connect to apm-server: {e}")),
    }
}
