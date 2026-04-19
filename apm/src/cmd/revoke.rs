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
    match ureq::delete(&url).send_json(&body) {
        Ok(resp) => {
            let json: serde_json::Value = resp
                .into_json()
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
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            eprintln!("error: server returned {code}: {body}");
            std::process::exit(1);
        }
        Err(e) => Err(anyhow::anyhow!("error: cannot connect to apm-server: {e}")),
    }
}
