use anyhow::{bail, Context, Result};
use std::path::Path;

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
        .context("GitHub API response is not an array")?
        .iter()
        .filter_map(|v| v["login"].as_str().map(|s| s.to_string()))
        .collect();
    Ok(logins)
}

pub fn gh_pr_create_or_update(root: &Path, branch: &str, default_branch: &str, id: &str, title: &str, body: &str, messages: &mut Vec<String>) -> Result<()> {
    let existing = std::process::Command::new("gh")
        .args(["pr", "list", "--head", branch, "--state", "open", "--json", "number", "--jq", ".[0].number"])
        .current_dir(root)
        .output()?;

    let pr_num = String::from_utf8_lossy(&existing.stdout).trim().to_string();
    if !pr_num.is_empty() && pr_num != "null" {
        messages.push(format!("PR #{pr_num} already open for {branch}"));
        return Ok(());
    }

    let title_str = pr_title(id, title);
    let out = std::process::Command::new("gh")
        .args(["pr", "create", "--base", default_branch, "--head", branch,
               "--title", &title_str, "--body", body])
        .current_dir(root)
        .output()?;

    if out.status.success() {
        let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
        messages.push(format!("PR created: {url}"));
    } else {
        bail!("gh pr create failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

fn pr_title(id: &str, title: &str) -> String {
    let short_id = &id[..8.min(id.len())];
    if title.is_empty() {
        short_id.to_string()
    } else {
        format!("{short_id}: {title}")
    }
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
    fn pr_title_includes_short_id_prefix() {
        let id = "034ed345-apm-state-include-ticket-id-in-github-pr";
        assert_eq!(pr_title(id, "Fix the thing"), "034ed345: Fix the thing");
    }

    #[test]
    fn pr_title_empty_title_falls_back_to_short_id() {
        let id = "034ed345-apm-state-include-ticket-id-in-github-pr";
        assert_eq!(pr_title(id, ""), "034ed345");
    }

    #[test]
    fn pr_title_short_id_exactly_8_chars() {
        let id = "abcd1234efgh";
        assert_eq!(pr_title(id, "My ticket"), "abcd1234: My ticket");
    }
}
