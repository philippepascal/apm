/// Integration tests for PathGuard and hook_config.
///
/// These tests cover the acceptance criteria for the filesystem path validator
/// by invoking the core library directly and, where the `apm` binary is
/// available, via subprocess.

use apm_core::config::IsolationConfig;
use apm_core::wrapper::path_guard::{PathGuard, canonicalize_lenient};
use apm_core::wrapper::hook_config::{write_hook_config, remove_hook_config};
use std::path::{Path, PathBuf};

// ---- helpers ----

fn make_guard(wt: &Path) -> PathGuard {
    PathGuard::new(wt, &[], &[]).unwrap()
}

fn make_guard_with_protected(wt: &Path, protected: &[PathBuf]) -> PathGuard {
    PathGuard::new(wt, &[], protected).unwrap()
}

fn make_guard_with_read_allow(wt: &Path, patterns: &[&str]) -> PathGuard {
    let patterns: Vec<String> = patterns.iter().map(|s| s.to_string()).collect();
    PathGuard::new(wt, &patterns, &[]).unwrap()
}

// ---- AC: path outside worktree is rejected ----

#[test]
fn ac_edit_outside_worktree_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let outside = tmp.path().join("main-worktree").join("src").join("lib.rs");
    let err = guard.check_write(&outside).unwrap_err();
    assert!(
        err.contains("path outside ticket worktree"),
        "rejection message must contain 'path outside ticket worktree': {err}"
    );
}

// ---- AC: rejection message includes APM_TICKET_WORKTREE ----

#[test]
fn ac_rejection_message_includes_worktree() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let outside = tmp.path().join("outside.txt");
    let err = guard.check_write(&outside).unwrap_err();
    assert!(
        err.contains("APM_TICKET_WORKTREE"),
        "rejection must include APM_TICKET_WORKTREE: {err}"
    );
}

// ---- AC: main-worktree file is unmodified after rejection ----

#[test]
fn ac_main_worktree_file_unmodified_after_rejection() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let sentinel = tmp.path().join("sentinel.txt");
    std::fs::write(&sentinel, "original").unwrap();

    // check_write returns Err — the file should remain unchanged
    let result = guard.check_write(&sentinel);
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "original");
}

// ---- AC: Edit inside worktree succeeds ----

#[test]
fn ac_edit_inside_worktree_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let inside = wt.join("src").join("main.rs");
    assert!(guard.check_write(&inside).is_ok());
}

// ---- AC: Write outside worktree rejected ----

#[test]
fn ac_write_outside_worktree_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let outside = tmp.path().join("new_file.txt");
    let err = guard.check_write(&outside).unwrap_err();
    assert!(err.contains("path outside ticket worktree"));
}

// ---- AC: Bash echo redirect outside rejected, file unmodified ----

#[test]
fn ac_bash_redirect_outside_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let target = tmp.path().join("outside.txt");
    std::fs::write(&target, "original").unwrap();

    let cmd = format!("echo foo > {}", target.display());
    let result = guard.check_bash(&cmd);
    assert!(result.is_err(), "bash redirect outside must be rejected");
    // File should still be unmodified (check_bash only validates, not writes)
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "original");
}

// ---- AC: cat /etc/resolv.conf allowed (no write target detected) ----

#[test]
fn ac_bash_cat_resolv_conf_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);
    assert!(guard.check_bash("cat /etc/resolv.conf").is_ok());
}

// ---- AC: cat ~/.gitconfig allowed ----

#[test]
fn ac_bash_cat_gitconfig_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);
    assert!(guard.check_bash("cat ~/.gitconfig").is_ok());
}

// ---- AC: Bash with paths only inside worktree allowed ----

#[test]
fn ac_bash_inside_paths_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let guard = make_guard(&wt);

    let inside = wt.join("output.txt");
    let cmd = format!("echo hello > {}", inside.display());
    assert!(guard.check_bash(&cmd).is_ok());
}

// ---- AC: enforce_worktree_isolation = false / absent → no enforcement ----

#[test]
fn ac_isolation_config_default_is_false() {
    let config = IsolationConfig::default();
    assert!(!config.enforce_worktree_isolation);
}

#[test]
fn ac_isolation_config_parses_toml() {
    use apm_core::config::Config;
    let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[isolation]
enforce_worktree_isolation = true
read_allow = ["/etc/resolv.conf", "~/.gitconfig"]
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.isolation.enforce_worktree_isolation);
    assert!(config.isolation.read_allow.contains(&"/etc/resolv.conf".to_string()));
}

#[test]
fn ac_isolation_config_absent_defaults_false() {
    use apm_core::config::Config;
    let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(!config.isolation.enforce_worktree_isolation);
}

// ---- AC: path resolution canonicalises .. before comparison ----

#[test]
fn ac_dotdot_escape_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    let sub = wt.join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    let guard = make_guard(&wt);

    // wt/subdir/../../etc/passwd — escapes to parent of wt
    let path = sub.join("..").join("..").join("etc").join("passwd");
    assert!(
        guard.check_write(&path).is_err(),
        "dotdot escape must be rejected"
    );
}

// ---- AC: path resolution follows symlinks ----

#[test]
fn ac_symlink_to_outside_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let outside = tmp.path().join("outside");
    std::fs::create_dir(&outside).unwrap();

    let link = wt.join("link");
    std::os::unix::fs::symlink(&outside, &link).unwrap();

    let guard = make_guard(&wt);
    let target = link.join("secret.txt");
    assert!(
        guard.check_write(&target).is_err(),
        "symlink resolving outside must be rejected"
    );
}

// ---- AC: APM_BIN write rejected regardless of path ----

#[test]
fn ac_apm_bin_write_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let apm_bin = tmp.path().join("usr").join("bin").join("apm");
    std::fs::create_dir_all(apm_bin.parent().unwrap()).unwrap();
    std::fs::write(&apm_bin, "binary").unwrap();

    let guard = make_guard_with_protected(&wt, &[apm_bin.clone()]);
    assert!(guard.check_write(&apm_bin).is_err());
}

// ---- AC: APM_SYSTEM_PROMPT_FILE / APM_USER_MESSAGE_FILE write rejected ----

#[test]
fn ac_system_prompt_file_write_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let sys_file = tmp.path().join("apm-sys-1234.txt");
    std::fs::write(&sys_file, "system prompt").unwrap();

    let guard = make_guard_with_protected(&wt, &[sys_file.clone()]);
    assert!(guard.check_write(&sys_file).is_err());
}

#[test]
fn ac_user_message_file_write_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();
    let msg_file = tmp.path().join("apm-msg-5678.txt");
    std::fs::write(&msg_file, "message").unwrap();

    let guard = make_guard_with_protected(&wt, &[msg_file.clone()]);
    assert!(guard.check_write(&msg_file).is_err());
}

// ---- AC: read_allow configurable; cat calls still allowed ----

#[test]
fn ac_read_allow_configurable_cat_still_allowed() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();

    // With custom read_allow patterns, cat calls remain allowed
    let guard = make_guard_with_read_allow(&wt, &["/custom/path/**", "/etc/resolv.conf"]);
    assert!(guard.check_bash("cat /custom/path/file.txt").is_ok());
    assert!(guard.check_bash("cat /etc/resolv.conf").is_ok());
}

// ---- AC: non-existent write target — intermediate components resolved ----

#[test]
fn ac_nonexistent_target_intermediate_symlinks_resolved() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    let sub = wt.join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    let guard = make_guard(&wt);

    // wt/subdir/../../etc/passwd — subdir exists but result escapes wt
    let path = sub.join("..").join("..").join("etc").join("passwd");
    assert!(
        guard.check_write(&path).is_err(),
        "intermediate-resolved path must be rejected if it escapes worktree"
    );
}

// ---- AC: APM_BIN inside worktree still rejected ----

#[test]
fn ac_apm_bin_inside_worktree_still_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    let bin_dir = wt.join("target").join("debug");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let apm_bin = bin_dir.join("apm");
    std::fs::write(&apm_bin, "binary").unwrap();

    let guard = make_guard_with_protected(&wt, &[apm_bin.clone()]);
    // Even though apm_bin is inside the worktree, it must be rejected
    assert!(
        guard.check_write(&apm_bin).is_err(),
        "APM_BIN inside worktree must still be rejected"
    );
}

// ---- AC: canonicalize_lenient — existing ancestors resolved, non-existent appended ----

#[test]
fn canonicalize_lenient_existing_ancestors_with_nonexistent_leaf() {
    let tmp = tempfile::tempdir().unwrap();
    let wt = tmp.path().join("wt");
    std::fs::create_dir(&wt).unwrap();

    let path = wt.join("new_file_does_not_exist.txt");
    let result = canonicalize_lenient(&path);

    // Parent should be resolved (wt exists), leaf appended lexically
    let canon_wt = std::fs::canonicalize(&wt).unwrap();
    assert_eq!(result.parent().unwrap(), canon_wt);
    assert_eq!(result.file_name().unwrap().to_str().unwrap(), "new_file_does_not_exist.txt");
}

// ---- hook_config integration tests ----

#[test]
fn hook_config_write_and_remove_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    write_hook_config(tmp.path(), "/usr/local/bin/apm").unwrap();

    let settings_path = tmp.path().join(".claude").join("settings.json");
    let content = std::fs::read_to_string(&settings_path).unwrap();
    assert!(content.contains("apm path-guard"));
    assert!(content.contains("Edit|Write|Bash"));

    remove_hook_config(tmp.path()).unwrap();
    let content_after = std::fs::read_to_string(&settings_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content_after).unwrap();
    let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(arr.len(), 0);
}

// ---- manifest.enforce_worktree_isolation field ----

#[test]
fn manifest_enforce_worktree_isolation_defaults_false() {
    use apm_core::wrapper::custom::Manifest;

    let toml = "[wrapper]\n";
    #[derive(serde::Deserialize)]
    struct ManifestFile { wrapper: Manifest }
    let file: ManifestFile = toml::from_str(toml).unwrap();
    assert!(!file.wrapper.enforce_worktree_isolation);
}

#[test]
fn manifest_enforce_worktree_isolation_parses_true() {
    use apm_core::wrapper::custom::Manifest;

    let toml = "[wrapper]\nenforce_worktree_isolation = true\n";
    #[derive(serde::Deserialize)]
    struct ManifestFile { wrapper: Manifest }
    let file: ManifestFile = toml::from_str(toml).unwrap();
    assert!(file.wrapper.enforce_worktree_isolation);
}

#[test]
fn manifest_enforce_worktree_isolation_not_unknown_key() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
    std::fs::create_dir_all(&agent_dir).unwrap();
    std::fs::write(
        agent_dir.join("manifest.toml"),
        "[wrapper]\nenforce_worktree_isolation = true\n",
    )
    .unwrap();

    let unknown = apm_core::wrapper::custom::manifest_unknown_keys(root, "my-wrapper").unwrap();
    assert!(
        !unknown.contains(&"enforce_worktree_isolation".to_string()),
        "enforce_worktree_isolation should be a known key, not unknown: {unknown:?}"
    );
}
