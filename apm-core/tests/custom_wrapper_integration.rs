use apm_core::wrapper::{WrapperContext, WrapperKind, Wrapper};
use apm_core::wrapper::custom::CustomWrapper;
use std::collections::HashMap;

#[cfg(unix)]
#[test]
fn integration_echo_test_wrapper() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create fixture: .apm/agents/echo-test/wrapper.sh
    let agent_dir = root.join(".apm").join("agents").join("echo-test");
    std::fs::create_dir_all(&agent_dir).unwrap();

    let script_path = agent_dir.join("wrapper.sh");
    std::fs::write(
        &script_path,
        "#!/bin/sh\nprintf '{\"type\":\"result\",\"text\":\"hello\"}\\n'\nexit 0\n",
    ).unwrap();
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Temp worktree and log file
    let wt = tempfile::tempdir().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let log_path = log_dir.path().join("worker.log");

    // resolve_wrapper returns Custom variant
    let kind = apm_core::wrapper::resolve_wrapper(root, "echo-test")
        .expect("resolve_wrapper should not error")
        .expect("echo-test should be found");
    assert!(matches!(kind, WrapperKind::Custom { .. }), "expected Custom variant, got Builtin");

    // Build a minimal WrapperContext
    let sys_file = apm_core::wrapper::write_temp_file("sys", "system prompt").unwrap();
    let msg_file = apm_core::wrapper::write_temp_file("msg", "ticket content").unwrap();

    let ctx = WrapperContext {
        worker_name: "test-worker".to_string(),
        ticket_id: "echo-test-id".to_string(),
        ticket_branch: "ticket/echo-test-id".to_string(),
        worktree_path: wt.path().to_path_buf(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions: false,
        profile: "default".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
    };

    // Spawn custom wrapper and wait
    let (script, manifest) = match kind {
        WrapperKind::Custom { script_path, manifest } => (script_path, manifest),
        WrapperKind::Builtin(_) => panic!("expected Custom"),
    };
    let wrapper = CustomWrapper { script_path: script, manifest };
    let mut child = wrapper.spawn(&ctx).expect("spawn should succeed");
    let status = child.wait().expect("wait should succeed");
    assert!(status.success(), "wrapper should exit 0; got: {status}");

    // Log file should contain the emitted JSONL line
    let log_content = std::fs::read_to_string(&log_path)
        .expect("log file should exist after wrapper exits");
    assert!(
        log_content.contains(r#"{"type":"result","text":"hello"}"#),
        "log file must contain the emitted JSONL line; got:\n{log_content}"
    );

    let _ = std::fs::remove_file(&sys_file);
    let _ = std::fs::remove_file(&msg_file);
}

#[cfg(unix)]
#[test]
fn spawn_matching_contract_succeeds() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create fixture: .apm/agents/v1-agent/wrapper.sh + manifest.toml declaring version 1
    let agent_dir = root.join(".apm").join("agents").join("v1-agent");
    std::fs::create_dir_all(&agent_dir).unwrap();

    let script_path = agent_dir.join("wrapper.sh");
    std::fs::write(&script_path, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    std::fs::write(
        agent_dir.join("manifest.toml"),
        "[wrapper]\ncontract_version = 1\n",
    ).unwrap();

    let wt = tempfile::tempdir().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let log_path = log_dir.path().join("worker.log");

    let kind = apm_core::wrapper::resolve_wrapper(root, "v1-agent")
        .expect("resolve_wrapper should not error")
        .expect("v1-agent should be found");

    let (script, manifest) = match kind {
        WrapperKind::Custom { script_path, manifest } => (script_path, manifest),
        WrapperKind::Builtin(_) => panic!("expected Custom"),
    };
    let wrapper = CustomWrapper { script_path: script, manifest };

    let ctx = WrapperContext {
        worker_name: "v1-agent".to_string(),
        ticket_id: "v1-test".to_string(),
        ticket_branch: "ticket/v1-test".to_string(),
        worktree_path: wt.path().to_path_buf(),
        system_prompt_file: wt.path().join("sys.txt"),
        user_message_file: wt.path().join("msg.txt"),
        skip_permissions: false,
        profile: "default".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
    };

    let mut child = wrapper.spawn(&ctx).expect("spawn should succeed for contract_version = 1");
    let status = child.wait().expect("wait should succeed");
    assert!(status.success(), "wrapper should exit 0; got: {status}");

    // No warning line should appear in the log for matching versions
    let log_content = std::fs::read_to_string(&log_path).unwrap_or_default();
    assert!(
        !log_content.contains("warning"),
        "log must not contain any warning for matching contract_version: {log_content}"
    );
}

#[cfg(unix)]
#[test]
fn integration_canonical_mode() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let agent_dir = root.join(".apm").join("agents").join("canonical-test");
    std::fs::create_dir_all(&agent_dir).unwrap();

    let script_path = agent_dir.join("wrapper.sh");
    std::fs::write(
        &script_path,
        "#!/bin/sh\nprintf '{\"type\":\"result\",\"text\":\"canonical-ok\"}\\n'\nexit 0\n",
    ).unwrap();
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let wt = tempfile::tempdir().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let log_path = log_dir.path().join("worker.log");

    let kind = apm_core::wrapper::resolve_wrapper(root, "canonical-test")
        .expect("resolve_wrapper should not error")
        .expect("canonical-test should be found");

    let (script, manifest) = match kind {
        WrapperKind::Custom { script_path, manifest } => (script_path, manifest),
        WrapperKind::Builtin(_) => panic!("expected Custom"),
    };
    let wrapper = CustomWrapper { script_path: script, manifest };

    let ctx = WrapperContext {
        worker_name: "canonical-test".to_string(),
        ticket_id: "canonical-id".to_string(),
        ticket_branch: "ticket/canonical-id".to_string(),
        worktree_path: wt.path().to_path_buf(),
        system_prompt_file: wt.path().join("sys.txt"),
        user_message_file: wt.path().join("msg.txt"),
        skip_permissions: false,
        profile: "default".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
    };

    let mut child = wrapper.spawn(&ctx).expect("spawn should succeed for canonical mode");
    let status = child.wait().expect("wait should succeed");
    assert!(status.success(), "wrapper should exit 0; got: {status}");

    let log_content = std::fs::read_to_string(&log_path)
        .expect("log file should exist after wrapper exits");
    assert!(
        log_content.contains(r#"{"type":"result","text":"canonical-ok"}"#),
        "log must contain the emitted JSONL line; got:\n{log_content}"
    );
}

#[cfg(unix)]
#[test]
fn integration_external_parser_pipe() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let agent_dir = root.join(".apm").join("agents").join("pipe-test");
    std::fs::create_dir_all(&agent_dir).unwrap();

    // Wrapper emits non-JSONL text on stdout
    let wrapper_script = agent_dir.join("wrapper.sh");
    std::fs::write(
        &wrapper_script,
        "#!/bin/sh\nprintf 'raw-output-line\\n'\nexit 0\n",
    ).unwrap();
    std::fs::set_permissions(&wrapper_script, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Parser reads each line from stdin and wraps it as JSON
    let parser_script = dir.path().join("parser.sh");
    std::fs::write(
        &parser_script,
        "#!/bin/sh\nwhile IFS= read -r line; do\n  printf '{\"type\":\"parsed\",\"content\":\"%s\"}\\n' \"$line\"\ndone\n",
    ).unwrap();
    std::fs::set_permissions(&parser_script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let parser_absolute = parser_script.to_string_lossy().to_string();

    // Write manifest with parser = "external" and absolute path to parser
    std::fs::write(
        agent_dir.join("manifest.toml"),
        format!(
            "[wrapper]\ncontract_version = 1\nparser = \"external\"\nparser_command = \"{}\"\n",
            parser_absolute.replace('\\', "\\\\")
        ),
    ).unwrap();

    let wt = tempfile::tempdir().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let log_path = log_dir.path().join("worker.log");

    let kind = apm_core::wrapper::resolve_wrapper(root, "pipe-test")
        .expect("resolve_wrapper should not error")
        .expect("pipe-test should be found");

    let (script, manifest) = match kind {
        WrapperKind::Custom { script_path, manifest } => (script_path, manifest),
        WrapperKind::Builtin(_) => panic!("expected Custom"),
    };
    let wrapper = CustomWrapper { script_path: script, manifest };

    let ctx = WrapperContext {
        worker_name: "pipe-test".to_string(),
        ticket_id: "pipe-id".to_string(),
        ticket_branch: "ticket/pipe-id".to_string(),
        worktree_path: wt.path().to_path_buf(),
        system_prompt_file: wt.path().join("sys.txt"),
        user_message_file: wt.path().join("msg.txt"),
        skip_permissions: false,
        profile: "default".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
    };

    let mut parser_child = wrapper.spawn(&ctx).expect("spawn should succeed for external mode");
    let status = parser_child.wait().expect("wait on parser child should succeed");
    assert!(status.success(), "parser should exit 0; got: {status}");

    let log_content = std::fs::read_to_string(&log_path)
        .expect("log file should exist after parser exits");
    assert!(
        log_content.contains("raw-output-line"),
        "log must contain input text wrapped in JSON; got:\n{log_content}"
    );
    assert!(
        log_content.contains(r#""type":"parsed""#),
        "log must contain parsed JSON object; got:\n{log_content}"
    );
}

#[cfg(unix)]
#[test]
fn spawn_future_contract_rejected() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create fixture: .apm/agents/future-agent/wrapper.sh + manifest.toml declaring version 2
    let agent_dir = root.join(".apm").join("agents").join("future-agent");
    std::fs::create_dir_all(&agent_dir).unwrap();

    let script_path = agent_dir.join("wrapper.sh");
    std::fs::write(&script_path, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();

    std::fs::write(
        agent_dir.join("manifest.toml"),
        "[wrapper]\ncontract_version = 2\n",
    ).unwrap();

    let wt = tempfile::tempdir().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let log_path = log_dir.path().join("worker.log");

    let kind = apm_core::wrapper::resolve_wrapper(root, "future-agent")
        .expect("resolve_wrapper should not error")
        .expect("future-agent should be found");

    let (script, manifest) = match kind {
        WrapperKind::Custom { script_path, manifest } => (script_path, manifest),
        WrapperKind::Builtin(_) => panic!("expected Custom"),
    };
    let wrapper = CustomWrapper { script_path: script, manifest };

    let ctx = WrapperContext {
        worker_name: "future-agent".to_string(),
        ticket_id: "future-test".to_string(),
        ticket_branch: "ticket/future-test".to_string(),
        worktree_path: wt.path().to_path_buf(),
        system_prompt_file: wt.path().join("sys.txt"),
        user_message_file: wt.path().join("msg.txt"),
        skip_permissions: false,
        profile: "default".to_string(),
        role_prefix: None,
        options: HashMap::new(),
        model: None,
        log_path: log_path.clone(),
        container: None,
        extra_env: HashMap::new(),
        root: root.to_path_buf(),
        keychain: HashMap::new(),
    };

    let result = wrapper.spawn(&ctx);
    assert!(result.is_err(), "spawn must return Err for contract_version = 2");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("upgrade APM"), "error must mention 'upgrade APM': {msg}");
}
