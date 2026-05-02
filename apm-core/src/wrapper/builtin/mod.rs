pub mod claude;
pub mod mock_happy;
pub mod mock_sad;
pub mod mock_random;
pub mod debug;

pub use claude::ClaudeWrapper;
pub use mock_happy::MockHappyWrapper;
pub use mock_sad::MockSadWrapper;
pub use mock_random::MockRandomWrapper;
pub use debug::DebugWrapper;

use std::collections::HashMap;
use crate::config::{Config, TransitionConfig, StateConfig};
use crate::wrapper::WrapperContext;

pub(crate) fn load_transitions_with_outcomes(
    ctx: &WrapperContext,
) -> anyhow::Result<Vec<(TransitionConfig, StateConfig)>> {
    let config = Config::load(&ctx.root)?;
    let current = config.workflow.states.iter()
        .find(|s| s.id == ctx.current_state)
        .ok_or_else(|| anyhow::anyhow!("state '{}' not found in workflow", ctx.current_state))?;
    let state_map: HashMap<&str, &StateConfig> = config.workflow.states.iter()
        .map(|s| (s.id.as_str(), s))
        .collect();
    let mut result = Vec::new();
    for t in &current.transitions {
        if let Some(&target) = state_map.get(t.to.as_str()) {
            result.push((t.clone(), target.clone()));
        }
    }
    Ok(result)
}

pub(crate) fn is_impl_mode(transitions: &[(TransitionConfig, StateConfig)]) -> bool {
    use crate::config::CompletionStrategy;
    transitions.iter().any(|(t, _)| t.completion != CompletionStrategy::None)
}

pub(crate) fn happy_script(id: &str, target: &str, impl_mode: bool) -> String {
    if impl_mode {
        format!(
            r#"#!/bin/sh
set -e
APM="${{APM_BIN:?APM_BIN not set — see wrapper contract}}"
ID="{id}"
printf 'mock: placeholder implementation for ticket %s\n' "$ID" > mock-implementation.txt
git add mock-implementation.txt
git commit -m "mock: placeholder commit for ticket $ID"
printf '%s\n' '{{"type":"tool_use","id":"mock-1","name":"git_commit","input":{{}}}}'
printf '%s\n' '{{"type":"tool_use","id":"mock-2","name":"apm_state","input":{{}}}}'
"$APM" state "$ID" {target}
rm -f "$0"
"#
        )
    } else {
        format!(
            r#"#!/bin/sh
set -e
APM="${{APM_BIN:?APM_BIN not set — see wrapper contract}}"
ID="{id}"
"$APM" spec "$ID" --section "Problem" --set "Mock spec — no real problem analyzed."
printf '%s\n' "- [ ] Mock criterion 1" "- [ ] Mock criterion 2" > ".apm-mock-ac-$$.txt"
"$APM" spec "$ID" --section "Acceptance criteria" --set-file ".apm-mock-ac-$$.txt"
rm -f ".apm-mock-ac-$$.txt"
"$APM" spec "$ID" --section "Out of scope" --set "Nothing in scope for this mock run"
"$APM" spec "$ID" --section "Approach" --set "Mock approach — no real implementation analyzed."
"$APM" set "$ID" effort 1
"$APM" set "$ID" risk 1
printf '%s\n' '{{"type":"tool_use","id":"mock-1","name":"write_spec","input":{{}}}}'
printf '%s\n' '{{"type":"tool_use","id":"mock-2","name":"apm_state","input":{{}}}}'
"$APM" state "$ID" {target}
rm -f "$0"
"#
        )
    }
}

pub(crate) fn sad_script(id: &str, target: &str) -> String {
    format!(
        r#"#!/bin/sh
set -e
APM="${{APM_BIN:?APM_BIN not set — see wrapper contract}}"
ID="{id}"
"$APM" spec "$ID" --section "Problem" --set "Mock sad run — spec intentionally incomplete."
printf '%s\n' '{{"type":"tool_use","id":"mock-1","name":"write_partial_spec","input":{{}}}}'
"$APM" state "$ID" {target}
rm -f "$0"
"#
    )
}

pub(crate) fn seed_from_ctx(ctx: &WrapperContext) -> u64 {
    // Check ctx.options["seed"] first (set by [workers.options] seed = "...")
    if let Some(s) = ctx.options.get("seed").and_then(|s| s.parse().ok()) {
        return s;
    }
    // Fall back to APM_OPT_SEED env var (for test injection or external scripts)
    if let Some(s) = std::env::var("APM_OPT_SEED").ok().and_then(|s| s.parse().ok()) {
        return s;
    }
    // Fall back to time-based random
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
}

pub(crate) fn write_and_spawn_script(
    name: &str,
    script: &str,
    ctx: &WrapperContext,
) -> anyhow::Result<std::process::Child> {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    use crate::wrapper::CONTRACT_VERSION;

    // Write the script file
    let script_path = ctx.worktree_path.join(format!(".apm-mock-{name}-{:04x}.sh", super::rand_u16()));
    std::fs::write(&script_path, script)?;
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;

    // Determine APM_BIN: ctx.options["apm_bin"] for tests, else current_exe
    let apm_bin = ctx.options.get("apm_bin")
        .cloned()
        .unwrap_or_else(|| {
            std::env::current_exe()
                .and_then(|p| p.canonicalize())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

    let mut cmd = std::process::Command::new("/bin/sh");
    cmd.arg(&script_path);

    // Set APM contract env vars
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
    cmd.env("APM_BIN", &apm_bin);
    cmd.env("APM_PROJECT_ROOT", ctx.root.to_string_lossy().as_ref());

    // Forward options as APM_OPT_<KEY>
    for (k, v) in &ctx.options {
        let env_key = format!(
            "APM_OPT_{}",
            k.to_uppercase().replace('.', "_").replace('-', "_")
        );
        cmd.env(&env_key, v);
    }

    cmd.current_dir(&ctx.worktree_path);
    cmd.process_group(0);

    let log_file = std::fs::File::create(&ctx.log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);

    Ok(cmd.spawn()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_ctx_with_options(opts: HashMap<String, String>) -> WrapperContext {
        WrapperContext {
            worker_name: "test".into(),
            ticket_id: "t".into(),
            ticket_branch: "b".into(),
            worktree_path: PathBuf::from("/tmp"),
            system_prompt_file: PathBuf::from("/tmp/sys"),
            user_message_file: PathBuf::from("/tmp/msg"),
            skip_permissions: false,
            profile: "default".into(),
            role_prefix: None,
            options: opts,
            model: None,
            log_path: PathBuf::from("/tmp/log"),
            container: None,
            extra_env: HashMap::new(),
            root: PathBuf::from("/tmp"),
            keychain: HashMap::new(),
            current_state: "test".into(),
        }
    }

    #[test]
    fn seed_from_ctx_uses_explicit_option() {
        let mut opts = HashMap::new();
        opts.insert("seed".into(), "12345".into());
        let ctx = make_ctx_with_options(opts);
        assert_eq!(seed_from_ctx(&ctx), 12345);
    }

    #[test]
    fn seed_from_ctx_falls_back_when_no_option() {
        // Without an explicit seed and no APM_OPT_SEED env, returns
        // a time-based value — just assert that it doesn't panic.
        let ctx = make_ctx_with_options(HashMap::new());
        let _ = seed_from_ctx(&ctx);
    }

    #[test]
    fn happy_script_includes_target_state_and_id() {
        let s = happy_script("abc123", "implemented", true);
        assert!(s.contains("ID=\"abc123\""), "id must appear: {s}");
        assert!(s.contains("apm\" state \"$ID\" implemented") || s.contains("$APM\" state \"$ID\" implemented"),
            "target transition must appear: {s}");
    }

    #[test]
    fn happy_script_spec_mode_writes_spec_sections() {
        let s = happy_script("abc123", "specd", false);
        assert!(s.contains("--section \"Problem\""), "spec mode must populate Problem: {s}");
        assert!(s.contains("--section \"Acceptance criteria\""), "spec mode must populate AC: {s}");
    }

    #[test]
    fn happy_script_impl_mode_creates_commit() {
        let s = happy_script("abc123", "implemented", true);
        assert!(s.contains("git commit"), "impl mode must create commit: {s}");
    }

    #[test]
    fn sad_script_includes_target_state() {
        let s = sad_script("abc123", "blocked");
        assert!(s.contains("ID=\"abc123\""), "id must appear: {s}");
        assert!(s.contains("apm\" state \"$ID\" blocked") || s.contains("$APM\" state \"$ID\" blocked"),
            "sad target must appear: {s}");
    }

    fn make_transition(to: &str, completion: crate::config::CompletionStrategy) -> crate::config::TransitionConfig {
        crate::config::TransitionConfig {
            to: to.into(),
            trigger: "command:state".into(),
            label: String::new(),
            hint: String::new(),
            completion,
            focus_section: None,
            context_section: None,
            warning: None,
            on_failure: None,
            outcome: None,
            profile: None,
        }
    }

    fn make_state(id: &str) -> crate::config::StateConfig {
        crate::config::StateConfig {
            id: id.into(),
            label: id.into(),
            description: String::new(),
            actionable: vec![],
            terminal: false,
            worker_end: false,
            satisfies_deps: crate::config::SatisfiesDeps::Bool(false),
            dep_requires: None,
            transitions: vec![],
            instructions: None,
        }
    }

    #[test]
    fn is_impl_mode_true_when_any_completion_strategy() {
        use crate::config::CompletionStrategy;
        assert!(is_impl_mode(&[(make_transition("implemented", CompletionStrategy::Merge), make_state("implemented"))]));
    }

    #[test]
    fn is_impl_mode_false_when_all_none() {
        use crate::config::CompletionStrategy;
        assert!(!is_impl_mode(&[(make_transition("specd", CompletionStrategy::None), make_state("specd"))]));
    }
}
