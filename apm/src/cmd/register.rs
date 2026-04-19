use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, username: &str, inferred: bool) -> Result<()> {
    if inferred {
        eprintln!("Registering as: {username}");
    }
    let config = apm_core::config::Config::load(root)?;
    let url = format!("{}/api/auth/otp", config.server.url);
    let body = serde_json::json!({"username": username});
    match ureq::post(&url).send_json(&body) {
        Ok(resp) => {
            let json: serde_json::Value = resp
                .into_json()
                .map_err(|e| anyhow::anyhow!("error: invalid response from server: {e}"))?;
            let otp = json
                .get("otp")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("error: missing otp field in response"))?;
            println!("{otp}");
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
