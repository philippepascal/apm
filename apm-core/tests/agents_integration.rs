use apm_core::agents::{list_wrappers, scaffold_wrapper, test_wrapper, eject_wrapper};
use apm_core::config::Config;
use apm_core::wrapper::WrapperKind;

/// Write a minimal config.toml so Config::load succeeds in the temp dir.
fn write_minimal_config(root: &std::path::Path) {
    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir).unwrap();
    std::fs::write(
        apm_dir.join("config.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();
}

/// Create an executable shell script at `path`.
#[cfg(unix)]
fn make_executable(path: &std::path::Path, content: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, content).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

// --------------------------------------------------------------------------
// list tests
// --------------------------------------------------------------------------

#[test]
fn list_shows_builtin_claude() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);
    let config = Config::load(root).unwrap();

    let entries = list_wrappers(root, &config).unwrap();
    let claude = entries.iter().find(|e| e.name == "claude");
    assert!(claude.is_some(), "claude entry not found");
    assert!(
        matches!(claude.unwrap().kind, WrapperKind::Builtin(_)),
        "expected Builtin kind for claude"
    );
}

#[test]
fn list_shows_project_wrapper() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    // Create .apm/agents/my-wrapper/wrapper.sh (mode 0o755)
    let agent_dir = root.join(".apm").join("agents").join("my-wrapper");
    std::fs::create_dir_all(&agent_dir).unwrap();
    #[cfg(unix)]
    make_executable(&agent_dir.join("wrapper.sh"), "#!/bin/sh\nexit 0\n");
    #[cfg(not(unix))]
    std::fs::write(agent_dir.join("wrapper.sh"), "exit 0\n").unwrap();

    let config = Config::load(root).unwrap();
    let entries = list_wrappers(root, &config).unwrap();
    let mw = entries.iter().find(|e| e.name == "my-wrapper");
    assert!(mw.is_some(), "my-wrapper entry not found");
    assert!(
        matches!(mw.unwrap().kind, WrapperKind::Custom { .. }),
        "expected Custom kind for my-wrapper"
    );
}

// --------------------------------------------------------------------------
// scaffold tests
// --------------------------------------------------------------------------

#[test]
fn scaffold_creates_all_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    scaffold_wrapper(root, "test-wrap", false).unwrap();

    let agent_dir = root.join(".apm").join("agents").join("test-wrap");
    assert!(agent_dir.join("wrapper.sh").exists(), "wrapper.sh missing");
    assert!(agent_dir.join("manifest.toml").exists(), "manifest.toml missing");
    assert!(agent_dir.join("apm.worker.md").exists(), "apm.worker.md missing");
    assert!(agent_dir.join("apm.spec-writer.md").exists(), "apm.spec-writer.md missing");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(agent_dir.join("wrapper.sh")).unwrap();
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "wrapper.sh must be executable"
        );
    }
}

#[test]
fn scaffold_refuses_existing_dir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    scaffold_wrapper(root, "dup-wrap", false).unwrap();
    let err = scaffold_wrapper(root, "dup-wrap", false).unwrap_err();
    assert!(
        err.to_string().contains("--force"),
        "error must mention --force: {err}"
    );
}

#[test]
fn scaffold_force_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    scaffold_wrapper(root, "over-wrap", false).unwrap();
    // Second call with force = true must succeed
    scaffold_wrapper(root, "over-wrap", true).unwrap();
}

// --------------------------------------------------------------------------
// test_wrapper tests
// --------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn test_passes_for_good_script() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    let agent_dir = root.join(".apm").join("agents").join("test-ok");
    std::fs::create_dir_all(&agent_dir).unwrap();
    make_executable(
        &agent_dir.join("wrapper.sh"),
        "#!/bin/sh\nprintf '{\"type\":\"text\",\"text\":\"ok\"}\\n'\nexit 0\n",
    );

    let report = test_wrapper(root, "test-ok").unwrap();
    assert!(report.passed, "expected passed=true");
    assert!(report.canonical_events >= 1, "expected at least one canonical event");
    assert_eq!(report.exit_code, 0);
}

#[cfg(unix)]
#[test]
fn test_fails_for_nonzero_exit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    let agent_dir = root.join(".apm").join("agents").join("test-fail");
    std::fs::create_dir_all(&agent_dir).unwrap();
    make_executable(&agent_dir.join("wrapper.sh"), "#!/bin/sh\nexit 1\n");

    let report = test_wrapper(root, "test-fail").unwrap();
    assert!(!report.passed, "expected passed=false");
    assert_eq!(report.exit_code, 1);
}

#[test]
fn test_unknown_wrapper_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    let err = test_wrapper(root, "not-a-wrapper").unwrap_err();
    assert!(
        err.to_string().contains("not found") || err.to_string().contains("not-a-wrapper"),
        "error should mention wrapper name: {err}"
    );
}

// --------------------------------------------------------------------------
// eject tests
// --------------------------------------------------------------------------

#[test]
fn eject_claude_creates_script() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    eject_wrapper(root, "claude").unwrap();

    let script = root.join(".apm").join("agents").join("claude").join("wrapper.sh");
    assert!(script.exists(), "wrapper.sh missing after eject");
    let content = std::fs::read_to_string(&script).unwrap();
    assert!(
        content.contains("claude"),
        "ejected script must reference claude: {content}"
    );
    assert!(
        content.contains("output-format"),
        "ejected script must contain output-format: {content}"
    );
}

#[test]
fn eject_claude_creates_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    eject_wrapper(root, "claude").unwrap();

    let manifest = root
        .join(".apm")
        .join("agents")
        .join("claude")
        .join("manifest.toml");
    let content = std::fs::read_to_string(&manifest).unwrap();
    assert!(
        content.contains("contract_version = 1"),
        "manifest must contain contract_version = 1: {content}"
    );
    assert!(
        content.contains("canonical"),
        "manifest must contain canonical: {content}"
    );
}

#[test]
fn eject_refuses_existing_dir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    // Pre-create the directory
    std::fs::create_dir_all(root.join(".apm").join("agents").join("claude")).unwrap();

    let err = eject_wrapper(root, "claude").unwrap_err();
    assert!(
        err.to_string().contains("already exists"),
        "error should mention already exists: {err}"
    );
}

#[test]
fn eject_unknown_builtin_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    let err = eject_wrapper(root, "not-a-builtin").unwrap_err();
    assert!(
        err.to_string().contains("not a known built-in"),
        "error should mention not a known built-in: {err}"
    );
}

#[test]
fn eject_sets_execute_bit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_minimal_config(root);

    eject_wrapper(root, "claude").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = root.join(".apm").join("agents").join("claude").join("wrapper.sh");
        let meta = std::fs::metadata(&script).unwrap();
        assert!(
            meta.permissions().mode() & 0o111 != 0,
            "ejected wrapper.sh must be executable"
        );
    }
}
