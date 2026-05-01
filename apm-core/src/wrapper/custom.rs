use std::path::{Path, PathBuf};
use serde::Deserialize;
use anyhow::Context;
use super::{Wrapper, WrapperContext};

fn default_contract_version() -> u32 { 1 }
fn default_parser() -> String { "canonical".to_string() }

#[derive(Debug, Deserialize, Clone)]
pub struct Manifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "default_contract_version")]
    pub contract_version: u32,
    #[serde(default = "default_parser")]
    pub parser: String,
    #[serde(default)]
    pub parser_command: Option<String>,
}

pub enum WrapperKind {
    Custom { script_path: PathBuf, manifest: Option<Manifest> },
    Builtin(String),
}

pub struct CustomWrapper {
    pub script_path: PathBuf,
    pub manifest: Option<Manifest>,
}

impl Wrapper for CustomWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        // Layer 2 spawn-time safety net: check contract_version unconditionally.
        // Even if apm validate already passed, the manifest may have been edited
        // between validate and this spawn call.
        let version = self.manifest.as_ref().map(|m| m.contract_version).unwrap_or(1);
        if version > 1 {
            anyhow::bail!(
                "wrapper at '{}' declares contract_version = {}; \
                 this APM build supports version 1 only — upgrade APM",
                self.script_path.display(),
                version
            );
        }

        let apm_bin = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut cmd = std::process::Command::new(&self.script_path);

        set_apm_env(&mut cmd, ctx, &apm_bin);
        for (k, v) in &ctx.extra_env {
            cmd.env(k, v);
        }

        cmd.current_dir(&ctx.worktree_path);

        let log_file = std::fs::File::create(&ctx.log_path)?;
        let log_clone = log_file.try_clone()?;
        cmd.stdout(log_file);
        cmd.stderr(log_clone);

        use std::os::unix::process::CommandExt;
        cmd.process_group(0);

        Ok(cmd.spawn()?)
    }
}

fn set_apm_env(cmd: &mut std::process::Command, ctx: &WrapperContext, apm_bin: &str) {
    cmd.env("APM_AGENT_NAME", &ctx.worker_name);
    cmd.env("APM_TICKET_ID", &ctx.ticket_id);
    cmd.env("APM_TICKET_BRANCH", &ctx.ticket_branch);
    cmd.env("APM_TICKET_WORKTREE", ctx.worktree_path.to_string_lossy().as_ref());
    cmd.env("APM_SYSTEM_PROMPT_FILE", ctx.system_prompt_file.to_string_lossy().as_ref());
    cmd.env("APM_USER_MESSAGE_FILE", ctx.user_message_file.to_string_lossy().as_ref());
    cmd.env("APM_SKIP_PERMISSIONS", if ctx.skip_permissions { "1" } else { "0" });
    cmd.env("APM_PROFILE", &ctx.profile);
    if let Some(ref prefix) = ctx.role_prefix {
        cmd.env("APM_ROLE_PREFIX", prefix);
    }
    cmd.env("APM_WRAPPER_VERSION", "1");
    cmd.env("APM_BIN", apm_bin);
    for (k, v) in &ctx.options {
        let env_key = format!(
            "APM_OPT_{}",
            k.to_uppercase().replace('.', "_").replace('-', "_")
        );
        cmd.env(&env_key, v);
    }
}

pub(crate) fn find_script(root: &Path, name: &str) -> Option<PathBuf> {
    let dir = root.join(".apm").join("agents").join(name);
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let fname = path.file_name()?.to_str()?.to_owned();
            if !fname.starts_with("wrapper.") {
                return None;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let meta = path.metadata().ok()?;
                if meta.permissions().mode() & 0o111 == 0 {
                    return None;
                }
            }
            Some(path)
        })
        .collect();
    candidates.sort();
    candidates.into_iter().next()
}

pub(crate) fn parse_manifest(root: &Path, name: &str) -> anyhow::Result<Option<Manifest>> {
    let path = root.join(".apm").join("agents").join(name).join("manifest.toml");
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;

    #[derive(Deserialize)]
    struct ManifestFile { wrapper: Manifest }

    let file: ManifestFile = toml::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(file.wrapper))
}

pub(crate) fn manifest_unknown_keys(root: &Path, name: &str) -> anyhow::Result<Vec<String>> {
    let path = root.join(".apm").join("agents").join(name).join("manifest.toml");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let table: toml::Value = content.parse::<toml::Value>()
        .with_context(|| format!("parsing {}", path.display()))?;
    let known = ["name", "contract_version", "parser", "parser_command"];
    let unknown = match table.get("wrapper").and_then(|v| v.as_table()) {
        Some(t) => t.keys()
            .filter(|k| !known.contains(&k.as_str()))
            .cloned()
            .collect(),
        None => vec![],
    };
    Ok(unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_ctx(wt: &std::path::Path, log: &std::path::Path) -> WrapperContext {
        WrapperContext {
            worker_name: "test-worker".to_string(),
            ticket_id: "test-id".to_string(),
            ticket_branch: "ticket/test-id".to_string(),
            worktree_path: wt.to_path_buf(),
            system_prompt_file: wt.join("sys.txt"),
            user_message_file: wt.join("msg.txt"),
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path: log.to_path_buf(),
            container: None,
            extra_env: HashMap::new(),
            root: wt.to_path_buf(),
            keychain: HashMap::new(),
        }
    }

    fn make_executable(path: &std::path::Path, content: &str) {
        std::fs::write(path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    // --- resolve_wrapper tests (via wrapper::resolve_wrapper) ---

    #[test]
    fn resolve_wrapper_custom_shadows_builtin() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("claude");
        std::fs::create_dir_all(&agent_dir).unwrap();
        make_executable(&agent_dir.join("wrapper.sh"), "#!/bin/sh\nexit 0\n");

        let result = crate::wrapper::resolve_wrapper(root, "claude").unwrap();
        assert!(matches!(result, Some(WrapperKind::Custom { .. })), "expected Custom variant");
    }

    #[test]
    fn resolve_wrapper_fallback_to_builtin() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // No .apm/agents/claude/ dir

        let result = crate::wrapper::resolve_wrapper(root, "claude").unwrap();
        assert!(matches!(result, Some(WrapperKind::Builtin(ref n)) if n == "claude"),
            "expected Builtin(claude)");
    }

    #[test]
    fn resolve_wrapper_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // "bogus-agent" is neither a builtin nor a custom script

        let result = crate::wrapper::resolve_wrapper(root, "bogus-agent").unwrap();
        assert!(result.is_none(), "expected None");
    }

    #[test]
    fn resolve_wrapper_nonexecutable_invisible() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("claude");
        std::fs::create_dir_all(&agent_dir).unwrap();

        // Write non-executable wrapper.sh
        let script = agent_dir.join("wrapper.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        // Non-executable script is invisible; falls through to builtin
        let result = crate::wrapper::resolve_wrapper(root, "claude").unwrap();
        assert!(matches!(result, Some(WrapperKind::Builtin(ref n)) if n == "claude"),
            "non-executable script should be invisible; expected fallback to Builtin(claude)");
    }

    // --- manifest tests ---

    #[test]
    fn manifest_parse_valid() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
        std::fs::create_dir_all(&agent_dir).unwrap();
        std::fs::write(agent_dir.join("manifest.toml"),
            "[wrapper]\nname = \"my-wrapper\"\ncontract_version = 1\nparser = \"canonical\"\n"
        ).unwrap();

        let m = parse_manifest(root, "my-wrapper").unwrap().unwrap();
        assert_eq!(m.contract_version, 1);
        assert_eq!(m.parser, "canonical");
        assert_eq!(m.name.as_deref(), Some("my-wrapper"));
        assert!(m.parser_command.is_none());
    }

    #[test]
    fn manifest_parse_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
        std::fs::create_dir_all(&agent_dir).unwrap();
        std::fs::write(agent_dir.join("manifest.toml"), "[wrapper]\n").unwrap();

        let m = parse_manifest(root, "my-wrapper").unwrap().unwrap();
        assert_eq!(m.contract_version, 1);
        assert_eq!(m.parser, "canonical");
        assert!(m.parser_command.is_none());
    }

    #[test]
    fn manifest_parse_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
        std::fs::create_dir_all(&agent_dir).unwrap();
        std::fs::write(agent_dir.join("manifest.toml"), "[[[\nbad toml\n").unwrap();

        assert!(parse_manifest(root, "my-wrapper").is_err(), "expected parse error");
    }

    #[test]
    fn manifest_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
        std::fs::create_dir_all(&agent_dir).unwrap();
        // No manifest.toml

        assert!(parse_manifest(root, "my-wrapper").unwrap().is_none());
    }

    #[test]
    fn manifest_unknown_keys_detected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
        std::fs::create_dir_all(&agent_dir).unwrap();
        std::fs::write(agent_dir.join("manifest.toml"),
            "[wrapper]\ncontract_version = 1\nunknown_key = \"foo\"\n"
        ).unwrap();

        let unknown = manifest_unknown_keys(root, "my-wrapper").unwrap();
        assert!(unknown.contains(&"unknown_key".to_string()),
            "expected unknown_key in {unknown:?}");
    }

    #[test]
    fn spawn_rejects_contract_version_gt_1() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();

        // Create a script (won't be reached due to early bail)
        let script = wt.path().join("wrapper.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        let manifest = Manifest {
            name: None,
            contract_version: 2,
            parser: "canonical".to_string(),
            parser_command: None,
        };

        let wrapper = CustomWrapper {
            script_path: script,
            manifest: Some(manifest),
        };

        let ctx = make_ctx(wt.path(), &log_dir.path().join("worker.log"));
        let err = wrapper.spawn(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("upgrade APM"),
            "error message must mention 'upgrade APM': {msg}");
    }
}
