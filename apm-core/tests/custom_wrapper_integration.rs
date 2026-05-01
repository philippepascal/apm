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
