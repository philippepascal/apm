use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::wrapper::{self, Wrapper, WrapperContext, WrapperKind};
use crate::wrapper::custom::CustomWrapper;
use crate::config::Config;

pub struct WrapperEntry {
    pub name: String,
    pub kind: WrapperKind,
    pub parser: String,
    pub configured_as: Vec<String>,
}

#[derive(Debug)]
pub struct TestReport {
    pub exit_code: i32,
    pub canonical_events: usize,
    pub non_canonical_lines: usize,
    pub stderr_lines: usize,
    pub wall_millis: u64,
    pub passed: bool,
}

const MANIFEST_TEMPLATE: &str =
    "[wrapper]\ncontract_version = 1\nparser = \"canonical\"\n";

const WRAPPER_TEMPLATE: &str = r#"#!/usr/bin/env bash
# APM wrapper skeleton
#
# Environment variables provided by APM:
#   APM_AGENT_NAME          - name of this worker (from config)
#   APM_TICKET_ID           - 8-char hex ticket ID
#   APM_TICKET_BRANCH       - git branch for this ticket
#   APM_TICKET_WORKTREE     - absolute path to the ticket worktree
#   APM_SYSTEM_PROMPT_FILE  - path to a file containing the system prompt
#   APM_USER_MESSAGE_FILE   - path to a file containing the user message (ticket content)
#   APM_SKIP_PERMISSIONS    - "1" if --dangerously-skip-permissions should be passed; "0" otherwise
#   APM_PROFILE             - active worker profile name
#   APM_ROLE_PREFIX         - optional role label prepended to the worker identity
#   APM_WRAPPER_VERSION     - contract version this APM build implements (currently "1")
#   APM_BIN                 - absolute path to the running apm binary
#   APM_OPT_*               - key-value options from [workers.options] in config.toml
#
# Contract:
#   stdout  - emit JSONL events (one JSON object per line, each with a "type" key)
#   stderr  - free-form log output (not parsed by APM)
#   exit 0  - success; non-zero signals failure
#
set -euo pipefail

# Dump all APM_* env vars to stderr for debugging
env | grep '^APM_' >&2 || true

# Read inputs
SYSTEM_PROMPT="$(cat "$APM_SYSTEM_PROMPT_FILE")"
USER_MESSAGE="$(cat "$APM_USER_MESSAGE_FILE")"

# TODO: replace this printf with a real agent invocation that:
#   1. Sends SYSTEM_PROMPT + USER_MESSAGE to your AI tool
#   2. Emits JSONL events on stdout as the tool runs
printf '{"type":"text","text":"wrapper skeleton -- replace with real invocation"}\n'

# TODO: when the agent finishes, transition the ticket:
#   apm state "$APM_TICKET_ID" <target-state>

exit 0
"#;

const CLAUDE_EJECT_SCRIPT: &str = r#"#!/usr/bin/env bash
# Ejected from APM built-in: claude
set -euo pipefail

ARGS=(--print --output-format stream-json --verbose)

ARGS+=(--system-prompt "$(cat "$APM_SYSTEM_PROMPT_FILE")")

if [[ -n "${APM_OPT_MODEL:-}" ]]; then
    ARGS+=(--model "$APM_OPT_MODEL")
fi

if [[ "${APM_SKIP_PERMISSIONS:-0}" == "1" ]]; then
    ARGS+=(--dangerously-skip-permissions)
fi

exec claude "${ARGS[@]}" "$(cat "$APM_USER_MESSAGE_FILE")"
"#;

const DEFAULT_WORKER_MD: &str = include_str!("default/apm.worker.md");
const DEFAULT_SPEC_WRITER_MD: &str = include_str!("default/apm.spec-writer.md");

fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u16
}

pub fn list_wrappers(root: &Path, config: &Config) -> Result<Vec<WrapperEntry>> {
    let mut entries: Vec<WrapperEntry> = Vec::new();

    // Built-in entries
    for name in wrapper::list_builtin_names() {
        entries.push(WrapperEntry {
            name: name.to_string(),
            kind: WrapperKind::Builtin(name.to_string()),
            parser: "canonical".to_string(),
            configured_as: vec![],
        });
    }

    // Project entries from .apm/agents/
    let agents_dir = root.join(".apm").join("agents");
    if agents_dir.is_dir() {
        let rd = match std::fs::read_dir(&agents_dir) {
            Ok(rd) => rd,
            Err(_) => return Ok(entries),
        };
        let mut names: Vec<String> = rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        names.sort();

        for entry_name in names {
            if let Ok(Some(WrapperKind::Custom { script_path, manifest })) =
                wrapper::resolve_wrapper(root, &entry_name)
            {
                let parser = manifest
                    .as_ref()
                    .map(|m| m.parser.clone())
                    .unwrap_or_else(|| "canonical".to_string());
                entries.push(WrapperEntry {
                    name: entry_name,
                    kind: WrapperKind::Custom { script_path, manifest },
                    parser,
                    configured_as: vec![],
                });
            }
        }
    }

    // Configured marker: global [workers].agent plus per-profile [worker_profiles.*].agent.
    let global_agent = config.workers.agent.as_deref().unwrap_or("claude").to_string();
    for entry in &mut entries {
        if entry.name == global_agent {
            entry.configured_as.push("(configured)".to_string());
        }
        for (profile_name, profile) in &config.worker_profiles {
            if let Some(ref agent) = profile.agent {
                if entry.name == *agent {
                    entry.configured_as.push(format!("({profile_name})"));
                }
            }
        }
    }

    Ok(entries)
}

pub fn scaffold_wrapper(root: &Path, name: &str, force: bool) -> Result<()> {
    let dir = root.join(".apm").join("agents").join(name);
    if dir.exists() && !force {
        anyhow::bail!(".apm/agents/{name}/ already exists; use --force to overwrite");
    }
    std::fs::create_dir_all(&dir)?;

    // Write wrapper.sh
    let wrapper_path = dir.join("wrapper.sh");
    std::fs::write(&wrapper_path, WRAPPER_TEMPLATE)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Write manifest.toml
    std::fs::write(dir.join("manifest.toml"), MANIFEST_TEMPLATE)?;

    // Write apm.worker.md
    let worker_md = std::fs::read_to_string(root.join(".apm").join("apm.worker.md"))
        .unwrap_or_else(|_| DEFAULT_WORKER_MD.to_string());
    std::fs::write(dir.join("apm.worker.md"), &worker_md)?;

    // Write apm.spec-writer.md
    let spec_writer_md =
        std::fs::read_to_string(root.join(".apm").join("apm.spec-writer.md"))
            .unwrap_or_else(|_| DEFAULT_SPEC_WRITER_MD.to_string());
    std::fs::write(dir.join("apm.spec-writer.md"), &spec_writer_md)?;

    Ok(())
}

pub fn test_wrapper(root: &Path, name: &str) -> Result<TestReport> {
    let kind = wrapper::resolve_wrapper(root, name)?.ok_or_else(|| {
        anyhow::anyhow!(
            "agent '{}' not found: checked built-ins and .apm/agents/{}/",
            name,
            name
        )
    })?;

    let tmp: PathBuf =
        std::env::temp_dir().join(format!("apm-agents-test-{:04x}", rand_u16()));
    std::fs::create_dir_all(&tmp)?;

    let sys_file = tmp.join("system.txt");
    let msg_file = tmp.join("message.txt");
    let log_path = tmp.join("wrapper.log");

    std::fs::write(&sys_file, "You are a test agent.")?;
    std::fs::write(&msg_file, "Test run -- apm agents test.")?;

    let ctx = WrapperContext {
        worker_name: "agents-test".to_string(),
        ticket_id: "00000000".to_string(),
        ticket_branch: "test/agents-test".to_string(),
        worktree_path: tmp.clone(),
        system_prompt_file: sys_file,
        user_message_file: msg_file,
        skip_permissions: false,
        profile: "test".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
        current_state: "test".to_string(),
    };

    let start = std::time::Instant::now();
    let mut child = match kind {
        WrapperKind::Custom { script_path, manifest } => {
            CustomWrapper { script_path, manifest }.spawn(&ctx)?
        }
        WrapperKind::Builtin(n) => {
            wrapper::resolve_builtin(&n)
                .expect("registered builtin")
                .spawn(&ctx)?
        }
    };

    let status = child.wait()?;
    let wall_millis = start.elapsed().as_millis() as u64;
    let exit_code = status.code().unwrap_or(-1);

    // Classify log lines
    let log_content = std::fs::read_to_string(&log_path).unwrap_or_default();
    let mut canonical_events = 0usize;
    let mut non_canonical_lines = 0usize;
    let mut stderr_lines = 0usize;

    for line in log_content.lines() {
        if line.is_empty() {
            continue;
        }
        if line.starts_with("APM_") {
            stderr_lines += 1;
        } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            if val.get("type").is_some() {
                canonical_events += 1;
            } else {
                non_canonical_lines += 1;
            }
        } else {
            non_canonical_lines += 1;
        }
    }

    let passed = status.success() && canonical_events >= 1;
    let report = TestReport {
        exit_code,
        canonical_events,
        non_canonical_lines,
        stderr_lines,
        wall_millis,
        passed,
    };

    let _ = std::fs::remove_dir_all(&tmp);

    Ok(report)
}

pub fn eject_wrapper(root: &Path, name: &str) -> Result<()> {
    if wrapper::resolve_builtin(name).is_none() {
        anyhow::bail!(
            "'{}' is not a known built-in; run apm agents list to see available wrappers",
            name
        );
    }

    let dir = root.join(".apm").join("agents").join(name);
    if dir.exists() {
        anyhow::bail!(".apm/agents/{name}/ already exists; delete it first to eject again");
    }

    std::fs::create_dir_all(&dir)?;

    let script_content = match name {
        "claude" => CLAUDE_EJECT_SCRIPT,
        other => anyhow::bail!("eject not yet implemented for built-in {}", other),
    };
    let script_path = dir.join("wrapper.sh");
    std::fs::write(&script_path, script_content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Write manifest.toml — intentionally the same template as scaffold_wrapper:
    // recognised as v1-canonical by 2c32a282's manifest parser and 2e772eab's version check,
    // so the ejected script requires no extra setup.
    std::fs::write(dir.join("manifest.toml"), MANIFEST_TEMPLATE)?;

    Ok(())
}
