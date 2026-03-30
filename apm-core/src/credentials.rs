pub fn resolve(name: &str, keychain_service: Option<&str>) -> anyhow::Result<String> {
    if let Ok(v) = std::env::var(name) {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(service) = keychain_service {
        let out = std::process::Command::new("security")
            .args(["find-generic-password", "-s", service, "-w"])
            .output()?;
        if out.status.success() {
            let val = String::from_utf8(out.stdout)?.trim().to_string();
            if !val.is_empty() {
                return Ok(val);
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = keychain_service;
    anyhow::bail!("credential {:?} not found in environment or keychain", name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn prefers_env_var() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("TEST_CRED_0038", "from-env");
        let result = resolve("TEST_CRED_0038", Some("some-service"));
        std::env::remove_var("TEST_CRED_0038");
        assert_eq!(result.unwrap(), "from-env");
    }

    #[test]
    fn empty_env_var_is_skipped() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("TEST_CRED_0038_EMPTY", "");
        let result = resolve("TEST_CRED_0038_EMPTY", None);
        std::env::remove_var("TEST_CRED_0038_EMPTY");
        assert!(result.is_err());
    }

    #[test]
    fn missing_credential_fails() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("TEST_CRED_0038_MISSING");
        let result = resolve("TEST_CRED_0038_MISSING", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TEST_CRED_0038_MISSING"));
    }
}
