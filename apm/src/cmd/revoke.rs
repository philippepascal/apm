use anyhow::Result;
use std::path::Path;

pub fn run(
    root: &Path,
    username: Option<&str>,
    device: Option<&str>,
    all: bool,
) -> Result<()> {
    let config = apm_core::config::Config::load(root)?;
    let url = format!("{}/api/auth/sessions", config.server.url);
    let body = serde_json::json!({
        "username": username,
        "device": device,
        "all": all,
    });
    let client = reqwest::blocking::Client::new();
    let resp = client
        .delete(&url)
        .json(&body)
        .send()
        .map_err(|e| anyhow::anyhow!("error: cannot connect to apm-server: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        eprintln!("error: server returned {status}");
        std::process::exit(1);
    }
    let json: serde_json::Value = resp
        .json()
        .map_err(|e| anyhow::anyhow!("error: invalid response from server: {e}"))?;
    let revoked = json
        .get("revoked")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("error: missing revoked field in response"))? as usize;
    if revoked == 0 {
        if let Some(u) = username {
            println!("No sessions found for {u}.");
        } else {
            println!("Revoked 0 session(s).");
        }
    } else {
        println!("Revoked {revoked} session(s).");
    }
    Ok(())
}
