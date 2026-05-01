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
