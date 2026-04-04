use anyhow::{Context, Result};

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

pub fn fetch_repo_collaborators(token: &str, repo: &str) -> Result<Vec<String>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/repos/{repo}/collaborators");
    let resp: serde_json::Value = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "apm")
        .send()
        .context("GitHub API request failed")?
        .error_for_status()
        .context("GitHub API returned error status")?
        .json()
        .context("GitHub API response is not valid JSON")?;
    let logins = resp
        .as_array()
        .context("GitHub collaborators response is not an array")?
        .iter()
        .filter_map(|u| u["login"].as_str().map(|s| s.to_string()))
        .collect();
    Ok(logins)
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

    #[test]
    #[ignore]
    fn fetch_repo_collaborators_live() {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN required");
        let repo = std::env::var("GITHUB_REPO").expect("GITHUB_REPO required (owner/name)");
        let logins = fetch_repo_collaborators(&token, &repo).unwrap();
        assert!(!logins.is_empty());
    }
}
