use std::path::Path;

#[derive(serde::Deserialize, Default)]
struct LocalIdentity {
    username: Option<String>,
}

pub fn resolve_current_user(root: &Path) -> String {
    let local_path = root.join(".apm").join("local.toml");
    if let Ok(contents) = std::fs::read_to_string(&local_path) {
        if let Ok(local) = toml::from_str::<LocalIdentity>(&contents) {
            if let Some(ref u) = local.username {
                if !u.is_empty() {
                    return u.clone();
                }
            }
        }
    }
    // Fall back to GitHub identity if git_host is configured
    if let Ok(cfg) = crate::config::Config::load(root) {
        if cfg.git_host.provider.as_deref() == Some("github") {
            if let Some(login) = crate::github::gh_username() {
                return login;
            }
        }
    }
    "apm".to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_current_user;

    #[test]
    fn returns_apm_when_file_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(resolve_current_user(dir.path()), "apm");
    }

    #[test]
    fn returns_username_when_present() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".apm")).unwrap();
        std::fs::write(dir.path().join(".apm/local.toml"), "username = \"alice\"\n").unwrap();
        assert_eq!(resolve_current_user(dir.path()), "alice");
    }

    #[test]
    fn returns_apm_when_username_key_absent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".apm")).unwrap();
        std::fs::write(dir.path().join(".apm/local.toml"), "[workers]\ncommand = \"claude\"\n").unwrap();
        assert_eq!(resolve_current_user(dir.path()), "apm");
    }

    #[test]
    fn returns_apm_when_username_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".apm")).unwrap();
        std::fs::write(dir.path().join(".apm/local.toml"), "username = \"\"\n").unwrap();
        assert_eq!(resolve_current_user(dir.path()), "apm");
    }
}
