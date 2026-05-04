use std::io::Write;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use anyhow::Context;
use super::{Wrapper, WrapperContext, CONTRACT_VERSION};

#[derive(Debug, Clone, PartialEq)]
enum ParserStrategy {
    Canonical,
    External,
}

impl ParserStrategy {
    fn from_manifest(m: Option<&Manifest>) -> Self {
        match m.and_then(|m| Some(m.parser.as_str())) {
            Some("external") => Self::External,
            _ => Self::Canonical,
        }
    }
}

/// Locate an executable binary by name or absolute path.
/// For absolute paths: checks the file exists.
/// For relative names: walks PATH entries and returns the first executable match.
fn find_binary(cmd: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(cmd);
    if p.is_absolute() {
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
        anyhow::bail!("parser binary not found: {}", cmd);
    }
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if !candidate.is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = candidate.metadata() {
                if meta.permissions().mode() & 0o111 == 0 {
                    continue;
                }
            }
        }
        return Ok(candidate);
    }
    anyhow::bail!("parser binary not found: {}", cmd);
}

fn default_contract_version() -> u32 { CONTRACT_VERSION }
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
    /// When true, APM installs a `PreToolUse` hook that blocks writes outside
    /// `APM_TICKET_WORKTREE`. Only applies to `parser = "canonical"` wrappers.
    #[serde(default)]
    pub enforce_worktree_isolation: bool,
}

pub enum WrapperKind {
    Custom { script_path: PathBuf, manifest: Option<Manifest> },
    Builtin(String),
}

pub struct CustomWrapper {
    pub script_path: PathBuf,
    pub manifest: Option<Manifest>,
}

fn check_contract_version(declared: u32, apm_version: u32, log_path: &Path) -> anyhow::Result<()> {
    match declared.cmp(&apm_version) {
        std::cmp::Ordering::Greater => anyhow::bail!(
            "wrapper targets contract version {} but this APM build supports up to \
             version {}; upgrade APM",
            declared,
            apm_version,
        ),
        std::cmp::Ordering::Less => {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(log_path)
            {
                let _ = writeln!(
                    f,
                    "[apm] warning: wrapper targets contract version {} but this APM \
                     build is version {}; the wrapper may not use newer env vars",
                    declared, apm_version,
                );
            }
        }
        std::cmp::Ordering::Equal => {}
    }
    Ok(())
}

impl Wrapper for CustomWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        // Layer 2 spawn-time safety net: check contract_version unconditionally.
        // Even if apm validate already passed, the manifest may have been edited
        // between validate and this spawn call.
        let declared = self.manifest.as_ref().map_or(1, |m| m.contract_version);
        check_contract_version(declared, CONTRACT_VERSION, &ctx.log_path)
            .map_err(|e| anyhow::anyhow!("wrapper '{}': {}", self.script_path.display(), e))?;

        let apm_bin = super::resolve_apm_cli_bin();

        // Write the path-guard hook for canonical wrappers that request isolation.
        let enforce = self.manifest.as_ref().map_or(false, |m| m.enforce_worktree_isolation);
        let strategy = ParserStrategy::from_manifest(self.manifest.as_ref());
        if enforce && strategy == ParserStrategy::Canonical {
            crate::wrapper::hook_config::write_hook_config(&ctx.worktree_path, &apm_bin)?;
        }

        let mut cmd = std::process::Command::new(&self.script_path);

        set_apm_env(&mut cmd, ctx, &apm_bin);
        for (k, v) in &ctx.extra_env {
            cmd.env(k, v);
        }
        cmd.current_dir(&ctx.worktree_path);

        #[cfg(unix)]
        use std::os::unix::process::CommandExt;

        match strategy {
            ParserStrategy::Canonical => {
                let log_file = std::fs::File::create(&ctx.log_path)?;
                let log_clone = log_file.try_clone()?;
                cmd.stdout(log_file);
                cmd.stderr(log_clone);
                #[cfg(unix)]
                cmd.process_group(0);
                Ok(cmd.spawn()?)
            }
            ParserStrategy::External => {
                let manifest_path = self.script_path
                    .parent()
                    .map(|p| p.join("manifest.toml"))
                    .unwrap_or_else(|| PathBuf::from("manifest.toml"));

                // Require parser_command
                let parser_cmd_str = self.manifest.as_ref()
                    .and_then(|m| m.parser_command.as_deref())
                    .ok_or_else(|| anyhow::anyhow!(
                        "{}: parser = \"external\" but parser_command is not set",
                        manifest_path.display()
                    ))?
                    .to_owned();

                // Validate binary is findable before spawning any process
                let parser_bin = find_binary(&parser_cmd_str)?;

                // Open log file; clone for each stream that writes to it:
                // 1. wrapper.stderr, 2. parser.stdout, 3. parser.stderr
                let log_file_wrapper_stderr = std::fs::File::create(&ctx.log_path)?;
                let log_file_parser_stdout = log_file_wrapper_stderr.try_clone()?;
                let log_file_parser_stderr = log_file_wrapper_stderr.try_clone()?;

                use std::process::Stdio;

                // Spawn wrapper: stdout piped to feed parser stdin; stderr directly to log
                cmd.stdout(Stdio::piped());
                cmd.stderr(log_file_wrapper_stderr);
                #[cfg(unix)]
                cmd.process_group(0);
                let mut wrapper_child = cmd.spawn()?;

                let wrapper_stdout = wrapper_child.stdout.take()
                    .ok_or_else(|| anyhow::anyhow!("failed to capture wrapper stdout pipe"))?;

                // Reap wrapper in background thread; append diagnostic exit line to log
                let log_path_clone = ctx.log_path.clone();
                std::thread::spawn(move || {
                    let status = wrapper_child.wait();
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&log_path_clone)
                    {
                        let status_str = match status {
                            Ok(s) => format!("{s}"),
                            Err(e) => format!("error: {e}"),
                        };
                        let _ = writeln!(f, "[apm] wrapper exited: {status_str}");
                    }
                });

                // Spawn parser: stdin = wrapper stdout pipe; stdout/stderr -> log
                let mut parser_cmd = std::process::Command::new(&parser_bin);
                parser_cmd.stdin(Stdio::from(wrapper_stdout));
                parser_cmd.stdout(log_file_parser_stdout);
                parser_cmd.stderr(log_file_parser_stderr);
                parser_cmd.current_dir(&ctx.worktree_path);
                #[cfg(unix)]
                parser_cmd.process_group(0);

                Ok(parser_cmd.spawn()?)
            }
        }
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
    cmd.env("APM_WRAPPER_VERSION", CONTRACT_VERSION.to_string());
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

pub fn manifest_unknown_keys(root: &Path, name: &str) -> anyhow::Result<Vec<String>> {
    let path = root.join(".apm").join("agents").join(name).join("manifest.toml");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let table: toml::Value = content.parse::<toml::Value>()
        .with_context(|| format!("parsing {}", path.display()))?;
    let known = ["name", "contract_version", "parser", "parser_command", "enforce_worktree_isolation"];
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
            current_state: "test".to_string(),
            command: None,
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

    // --- check_contract_version unit tests ---

    #[test]
    fn check_version_equal() {
        let log_dir = tempfile::tempdir().unwrap();
        let log_path = log_dir.path().join("worker.log");
        assert!(check_contract_version(1, 1, &log_path).is_ok());
        // No log file created for equal versions
        assert!(!log_path.exists() || std::fs::read_to_string(&log_path).unwrap().is_empty());
    }

    #[test]
    fn check_version_older_writes_warning() {
        let log_dir = tempfile::tempdir().unwrap();
        let log_path = log_dir.path().join("worker.log");
        // declared=1 is older than apm_version=2 → warning, Ok
        let result = check_contract_version(1, 2, &log_path);
        assert!(result.is_ok(), "expected Ok for older version");
        let content = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert!(content.contains("warning"), "log must contain 'warning': {content}");
        assert!(content.contains('1'), "log must contain declared version 1: {content}");
        assert!(content.contains('2'), "log must contain apm version 2: {content}");
    }

    #[test]
    fn check_version_too_high_returns_err() {
        let log_dir = tempfile::tempdir().unwrap();
        let log_path = log_dir.path().join("worker.log");
        let result = check_contract_version(2, 1, &log_path);
        assert!(result.is_err(), "expected Err for version > apm");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("upgrade APM"), "error must mention 'upgrade APM': {msg}");
        assert!(msg.contains('2'), "error must mention declared version 2: {msg}");
        assert!(msg.contains('1'), "error must mention apm version 1: {msg}");
    }

    #[test]
    fn default_contract_version_tracks_apm_version() {
        // Ensures that bumping CONTRACT_VERSION also updates the manifest serde
        // default, so older manifests don't silently parse with a stale version.
        assert_eq!(default_contract_version(), CONTRACT_VERSION);
    }

    // --- ParserStrategy tests ---

    #[test]
    fn parser_strategy_defaults_to_canonical() {
        assert_eq!(ParserStrategy::from_manifest(None), ParserStrategy::Canonical);
    }

    #[test]
    fn parser_strategy_explicit_canonical() {
        let m = Manifest {
            name: None,
            contract_version: 1,
            parser: "canonical".to_string(),
            parser_command: None,
            enforce_worktree_isolation: false,
        };
        assert_eq!(ParserStrategy::from_manifest(Some(&m)), ParserStrategy::Canonical);
    }

    #[test]
    fn parser_strategy_external() {
        let m = Manifest {
            name: None,
            contract_version: 1,
            parser: "external".to_string(),
            parser_command: Some("my-parser".to_string()),
            enforce_worktree_isolation: false,
        };
        assert_eq!(ParserStrategy::from_manifest(Some(&m)), ParserStrategy::External);
    }

    #[test]
    fn parser_strategy_unknown_falls_back_to_canonical() {
        let m = Manifest {
            name: None,
            contract_version: 1,
            parser: "foobar".to_string(),
            parser_command: None,
            enforce_worktree_isolation: false,
        };
        assert_eq!(ParserStrategy::from_manifest(Some(&m)), ParserStrategy::Canonical);
    }

    #[test]
    fn spawn_external_missing_parser_command() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let log_path = log_dir.path().join("worker.log");

        let script = wt.path().join("wrapper.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        let manifest = Manifest {
            name: None,
            contract_version: 1,
            parser: "external".to_string(),
            parser_command: None,
            enforce_worktree_isolation: false,
        };
        let wrapper = CustomWrapper {
            script_path: script,
            manifest: Some(manifest),
        };

        let ctx = make_ctx(wt.path(), &log_path);
        let err = wrapper.spawn(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("parser_command"), "error must mention parser_command: {msg}");
        assert!(msg.contains("not set"), "error must mention 'not set': {msg}");
    }

    #[test]
    fn spawn_external_binary_not_found() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let log_path = log_dir.path().join("worker.log");

        let script = wt.path().join("wrapper.sh");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        let manifest = Manifest {
            name: None,
            contract_version: 1,
            parser: "external".to_string(),
            parser_command: Some("nonexistent-binary-xyzzy-2803".to_string()),
            enforce_worktree_isolation: false,
        };
        let wrapper = CustomWrapper {
            script_path: script,
            manifest: Some(manifest),
        };

        let ctx = make_ctx(wt.path(), &log_path);
        let err = wrapper.spawn(&ctx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("nonexistent-binary-xyzzy-2803"),
            "error must name the missing binary: {msg}"
        );
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
            enforce_worktree_isolation: false,
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
