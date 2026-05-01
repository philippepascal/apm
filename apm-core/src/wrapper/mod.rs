mod claude;
pub mod custom;
pub use claude::ClaudeWrapper;
pub use custom::{WrapperKind, Manifest};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct WrapperContext {
    pub worker_name: String,
    pub ticket_id: String,
    pub ticket_branch: String,
    pub worktree_path: PathBuf,
    pub system_prompt_file: PathBuf,
    pub user_message_file: PathBuf,
    pub skip_permissions: bool,
    pub profile: String,
    pub role_prefix: Option<String>,
    pub options: HashMap<String, String>,
    pub model: Option<String>,
    pub log_path: PathBuf,
    pub container: Option<String>,
    pub extra_env: HashMap<String, String>,
    pub root: PathBuf,
    pub keychain: HashMap<String, String>,
}

pub trait Wrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child>;
}

pub fn resolve_builtin(name: &str) -> Option<Box<dyn Wrapper>> {
    match name {
        "claude" => Some(Box::new(ClaudeWrapper)),
        _ => None,
    }
}

pub fn resolve_wrapper(root: &Path, name: &str) -> anyhow::Result<Option<WrapperKind>> {
    if let Some(script_path) = custom::find_script(root, name) {
        let manifest = custom::parse_manifest(root, name)?;
        return Ok(Some(WrapperKind::Custom { script_path, manifest }));
    }
    if resolve_builtin(name).is_some() {
        return Ok(Some(WrapperKind::Builtin(name.to_owned())));
    }
    Ok(None)
}

pub fn write_temp_file(prefix: &str, content: &str) -> anyhow::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("apm-{prefix}-{:04x}.txt", rand_u16()));
    std::fs::write(&path, content)?;
    Ok(path)
}

fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_builtin_claude_returns_some() {
        assert!(resolve_builtin("claude").is_some());
    }

    #[test]
    fn resolve_builtin_unknown_returns_none() {
        assert!(resolve_builtin("bogus").is_none());
        assert!(resolve_builtin("").is_none());
        assert!(resolve_builtin("mock-happy").is_none());
    }
}
