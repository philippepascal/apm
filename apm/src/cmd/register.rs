use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, username: &str, inferred: bool) -> Result<()> {
    if inferred {
        eprintln!("Registering as: {username}");
    }
    let config = apm_core::config::Config::load(root)?;
    let url = format!("{}/api/auth/otp", config.server.url);
    let body = serde_json::json!({"username": username});
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
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
    let otp = json
        .get("otp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("error: missing otp field in response"))?;
    println!("{otp}");
    Ok(())
}
