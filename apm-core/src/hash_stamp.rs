use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

/// Hash the two APM config files in a fixed order. A missing file contributes
/// zero bytes (not an error). Returns the SHA-256 digest as a lowercase hex string.
pub fn config_hash(root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    for filename in ["config.toml", "workflow.toml"] {
        let path = root.join(".apm").join(filename);
        if let Ok(bytes) = std::fs::read(&path) {
            hasher.update(&bytes);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Read the stored stamp. Returns the trimmed content on success, None if absent
/// or unreadable.
pub fn read_stamp(root: &Path) -> Option<String> {
    let path = root.join(".apm").join(".validate-stamp");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Write `hash` to `.apm/.validate-stamp`, first ensuring `.apm/.gitignore`
/// contains `.validate-stamp` (append-only, idempotent).
pub fn write_stamp(root: &Path, hash: &str) -> Result<()> {
    let apm_dir = root.join(".apm");
    let gitignore_path = apm_dir.join(".gitignore");

    let entry = ".validate-stamp";
    match std::fs::read_to_string(&gitignore_path) {
        Ok(existing) => {
            if !existing.lines().any(|l| l.trim() == entry) {
                let mut f = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&gitignore_path)?;
                writeln!(f, "{entry}")?;
            }
        }
        Err(_) => {
            std::fs::write(&gitignore_path, format!("{entry}\n"))?;
        }
    }

    let stamp_path = apm_dir.join(".validate-stamp");
    std::fs::write(&stamp_path, format!("{hash}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_apm_dir(tmp: &TempDir) {
        std::fs::create_dir_all(tmp.path().join(".apm")).unwrap();
    }

    #[test]
    fn hash_is_deterministic() {
        let tmp = TempDir::new().unwrap();
        make_apm_dir(&tmp);
        std::fs::write(
            tmp.path().join(".apm").join("config.toml"),
            b"[project]\nname = \"test\"\n",
        )
        .unwrap();
        let h1 = config_hash(tmp.path()).unwrap();
        let h2 = config_hash(tmp.path()).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_changes_on_file_mutation() {
        let tmp = TempDir::new().unwrap();
        make_apm_dir(&tmp);
        let config_path = tmp.path().join(".apm").join("config.toml");
        std::fs::write(&config_path, b"[project]\nname = \"test\"\n").unwrap();
        let h1 = config_hash(tmp.path()).unwrap();
        std::fs::write(&config_path, b"[project]\nname = \"changed\"\n").unwrap();
        let h2 = config_hash(tmp.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn missing_files_are_stable() {
        let tmp = TempDir::new().unwrap();
        make_apm_dir(&tmp);
        let h1 = config_hash(tmp.path()).unwrap();
        let h2 = config_hash(tmp.path()).unwrap();
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }

    #[test]
    fn stamp_round_trip() {
        let tmp = TempDir::new().unwrap();
        make_apm_dir(&tmp);
        let hash = "abcdef1234567890";
        write_stamp(tmp.path(), hash).unwrap();
        let read_back = read_stamp(tmp.path()).unwrap();
        assert_eq!(read_back, hash);
    }
}
