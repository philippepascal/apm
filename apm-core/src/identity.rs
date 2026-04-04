use std::path::Path;

#[derive(serde::Deserialize, Default)]
struct LocalConfig {
    username: Option<String>,
}

pub fn resolve_current_user(root: &Path) -> String {
    let local_path = root.join(".apm").join("local.toml");
    if let Ok(contents) = std::fs::read_to_string(&local_path) {
        if let Ok(local) = toml::from_str::<LocalConfig>(&contents) {
            if let Some(ref u) = local.username {
                if !u.is_empty() {
                    return u.clone();
                }
            }
        }
    }
    "apm".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_current_user_absent() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(resolve_current_user(tmp.path()), "apm");
    }

    #[test]
    fn resolve_current_user_with_username() {
        let tmp = TempDir::new().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"alice\"\n").unwrap();
        assert_eq!(resolve_current_user(tmp.path()), "alice");
    }

    #[test]
    fn resolve_current_user_empty_username() {
        let tmp = TempDir::new().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"\"\n").unwrap();
        assert_eq!(resolve_current_user(tmp.path()), "apm");
    }
}
