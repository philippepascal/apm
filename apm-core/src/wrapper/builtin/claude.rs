use std::os::unix::process::CommandExt;
use crate::wrapper::{Wrapper, WrapperContext, CONTRACT_VERSION};

pub struct ClaudeWrapper;

impl Wrapper for ClaudeWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        let sys = std::fs::read_to_string(&ctx.system_prompt_file)?;
        let msg = std::fs::read_to_string(&ctx.user_message_file)?;

        let apm_bin = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        match &ctx.container {
            None => spawn_local(ctx, &sys, &msg, &apm_bin),
            Some(image) => spawn_container(ctx, image, &sys, &msg, &apm_bin),
        }
    }
}

pub(crate) fn build_claude_args(model: Option<&str>, skip_permissions: bool, sys: &str, msg: &str) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--print".into(),
        "--output-format".into(),
        "stream-json".into(),
        "--verbose".into(),
        "--system-prompt".into(),
        sys.into(),
    ];
    if let Some(m) = model {
        args.push("--model".into());
        args.push(m.into());
    }
    if skip_permissions {
        args.push("--dangerously-skip-permissions".into());
    }
    args.push(msg.into());
    args
}

fn spawn_local(
    ctx: &WrapperContext,
    sys: &str,
    msg: &str,
    apm_bin: &str,
) -> anyhow::Result<std::process::Child> {
    let mut cmd = std::process::Command::new("claude");
    cmd.args(build_claude_args(ctx.model.as_deref(), ctx.skip_permissions, sys, msg));

    set_apm_env(&mut cmd, ctx, apm_bin);
    for (k, v) in &ctx.extra_env {
        cmd.env(k, v);
    }

    cmd.current_dir(&ctx.worktree_path);

    let log_file = std::fs::File::create(&ctx.log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);
    cmd.process_group(0);

    Ok(cmd.spawn()?)
}

fn spawn_container(
    ctx: &WrapperContext,
    image: &str,
    sys: &str,
    msg: &str,
    apm_bin: &str,
) -> anyhow::Result<std::process::Child> {
    let api_key = crate::credentials::resolve(
        "ANTHROPIC_API_KEY",
        ctx.keychain.get("ANTHROPIC_API_KEY").map(|s| s.as_str()),
    )?;

    let author_name = std::env::var("GIT_AUTHOR_NAME")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| crate::git_util::git_config_get(&ctx.root, "user.name"))
        .unwrap_or_default();
    let author_email = std::env::var("GIT_AUTHOR_EMAIL")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| crate::git_util::git_config_get(&ctx.root, "user.email"))
        .unwrap_or_default();
    let committer_name = std::env::var("GIT_COMMITTER_NAME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| author_name.clone());
    let committer_email = std::env::var("GIT_COMMITTER_EMAIL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| author_email.clone());

    let mut cmd = std::process::Command::new("docker");
    cmd.arg("run");
    cmd.arg("--rm");
    cmd.args(["--volume", &format!("{}:/workspace", ctx.worktree_path.display())]);
    cmd.args(["--workdir", "/workspace"]);
    cmd.args(["--env", &format!("ANTHROPIC_API_KEY={api_key}")]);
    if !author_name.is_empty() {
        cmd.args(["--env", &format!("GIT_AUTHOR_NAME={author_name}")]);
    }
    if !author_email.is_empty() {
        cmd.args(["--env", &format!("GIT_AUTHOR_EMAIL={author_email}")]);
    }
    if !committer_name.is_empty() {
        cmd.args(["--env", &format!("GIT_COMMITTER_NAME={committer_name}")]);
    }
    if !committer_email.is_empty() {
        cmd.args(["--env", &format!("GIT_COMMITTER_EMAIL={committer_email}")]);
    }

    let skip_perm_val = if ctx.skip_permissions { "1" } else { "0" };
    let worktree_str = ctx.worktree_path.to_string_lossy();
    let sys_file_str = ctx.system_prompt_file.to_string_lossy();
    let msg_file_str = ctx.user_message_file.to_string_lossy();
    let contract_version_str = CONTRACT_VERSION.to_string();

    let apm_env_pairs: &[(&str, &str)] = &[
        ("APM_AGENT_NAME", &ctx.worker_name),
        ("APM_TICKET_ID", &ctx.ticket_id),
        ("APM_TICKET_BRANCH", &ctx.ticket_branch),
        ("APM_TICKET_WORKTREE", &worktree_str),
        ("APM_SYSTEM_PROMPT_FILE", &sys_file_str),
        ("APM_USER_MESSAGE_FILE", &msg_file_str),
        ("APM_SKIP_PERMISSIONS", skip_perm_val),
        ("APM_PROFILE", &ctx.profile),
        ("APM_WRAPPER_VERSION", &contract_version_str),
        ("APM_BIN", apm_bin),
    ];
    for (k, v) in apm_env_pairs {
        cmd.args(["--env", &format!("{k}={v}")]);
    }
    if let Some(ref prefix) = ctx.role_prefix {
        cmd.args(["--env", &format!("APM_ROLE_PREFIX={prefix}")]);
    }
    for (k, v) in &ctx.extra_env {
        cmd.args(["--env", &format!("{k}={v}")]);
    }
    // APM_OPT_<KEY> for each option entry
    for (k, v) in &ctx.options {
        let env_key = format!(
            "APM_OPT_{}",
            k.to_uppercase().replace('.', "_").replace('-', "_")
        );
        cmd.args(["--env", &format!("{env_key}={v}")]);
    }

    cmd.arg(image);
    cmd.arg("claude");
    cmd.args(build_claude_args(ctx.model.as_deref(), ctx.skip_permissions, sys, msg));

    let log_file = std::fs::File::create(&ctx.log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);
    cmd.process_group(0);

    Ok(cmd.spawn()?)
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
    // APM_OPT_<KEY> for each option entry
    for (k, v) in &ctx.options {
        let env_key = format!(
            "APM_OPT_{}",
            k.to_uppercase().replace('.', "_").replace('-', "_")
        );
        cmd.env(&env_key, v);
    }
}

#[cfg(test)]
mod tests {
    use super::build_claude_args;

    #[test]
    fn args_include_model_flag_when_set() {
        let args = build_claude_args(Some("sonnet"), false, "sys", "msg");
        let pos = args.iter().position(|a| a == "--model").expect("--model flag must be in argv");
        assert_eq!(args.get(pos + 1).map(String::as_str), Some("sonnet"), "value must follow --model");
    }

    #[test]
    fn args_omit_model_flag_when_unset() {
        let args = build_claude_args(None, false, "sys", "msg");
        assert!(!args.iter().any(|a| a == "--model"), "--model must be absent when no model configured: {args:?}");
    }

    #[test]
    fn args_include_skip_permissions_when_set() {
        let args = build_claude_args(None, true, "sys", "msg");
        assert!(args.iter().any(|a| a == "--dangerously-skip-permissions"), "{args:?}");
    }

    #[test]
    fn args_msg_is_last() {
        let args = build_claude_args(Some("opus"), true, "sys", "the-message");
        assert_eq!(args.last().map(String::as_str), Some("the-message"));
    }
}
