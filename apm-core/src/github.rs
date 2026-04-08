use anyhow::{Context, Result};

pub fn gh_username() -> Option<String> {
    std::process::Command::new("gh")
        .args(["api", "user", "-q", ".login"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}

pub fn fetch_authenticated_user(token: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp: serde_json::Value = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "apm")
        .send()
        .context("GitHub API request failed")?
        .error_for_status()
        .context("GitHub API returned error status")?
        .json()
        .context("GitHub API response is not valid JSON")?;
    resp["login"]
        .as_str()
        .map(|s| s.to_string())
        .context("GitHub API response missing 'login' field")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn fetch_authenticated_user_live() {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN required");
        let login = fetch_authenticated_user(&token).unwrap();
        assert!(!login.is_empty());
    }

}
