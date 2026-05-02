use apm_core::wrapper::path_guard::{PathGuard, canonicalize_lenient};
use std::io::Read;
use std::path::{Path, PathBuf};

/// `apm path-guard` — PreToolUse hook called by Claude Code before every
/// Edit, Write, and Bash tool invocation.
///
/// Reads a JSON payload from stdin:
/// ```json
/// {"tool_name": "Edit", "tool_input": {"file_path": "/some/path"}}
/// ```
///
/// Exit codes:
/// - 0: tool call allowed
/// - 2: tool call blocked; rejection message printed to stdout
pub fn run() {
    let mut stdin_buf = String::new();
    if std::io::stdin().read_to_string(&mut stdin_buf).is_err() {
        std::process::exit(0);
    }

    let payload: serde_json::Value = match serde_json::from_str(&stdin_buf) {
        Ok(v) => v,
        Err(_) => {
            // Malformed JSON — allow (do not block on parse failure)
            std::process::exit(0);
        }
    };

    let tool_name = match payload.get("tool_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => std::process::exit(0),
    };

    // Only intercept Edit, Write, and Bash
    if !matches!(tool_name, "Edit" | "Write" | "Bash") {
        std::process::exit(0);
    }

    // Read required environment variables
    let worktree_str = match std::env::var("APM_TICKET_WORKTREE") {
        Ok(v) if !v.is_empty() => v,
        _ => std::process::exit(0), // not running inside an APM worker — allow
    };
    let worktree = Path::new(&worktree_str);

    let apm_bin = std::env::var("APM_BIN").unwrap_or_default();
    let sys_file = std::env::var("APM_SYSTEM_PROMPT_FILE").unwrap_or_default();
    let msg_file = std::env::var("APM_USER_MESSAGE_FILE").unwrap_or_default();

    // Load IsolationConfig by walking up from the worktree
    let isolation = load_isolation_config(worktree);

    // Build PathGuard
    let mut write_protected: Vec<PathBuf> = Vec::new();
    if !apm_bin.is_empty() {
        write_protected.push(canonicalize_lenient(Path::new(&apm_bin)));
    }
    if !sys_file.is_empty() {
        write_protected.push(canonicalize_lenient(Path::new(&sys_file)));
    }
    if !msg_file.is_empty() {
        write_protected.push(canonicalize_lenient(Path::new(&msg_file)));
    }

    let guard = match PathGuard::new(worktree, &isolation.read_allow, &write_protected) {
        Ok(g) => g,
        Err(_) => std::process::exit(0), // construction failure — allow
    };

    let tool_input = match payload.get("tool_input") {
        Some(v) => v,
        None => std::process::exit(0),
    };

    let result = match tool_name {
        "Edit" | "Write" => {
            let file_path = match tool_input.get("file_path").and_then(|v| v.as_str()) {
                Some(p) => p,
                None => std::process::exit(0),
            };
            guard.check_write(Path::new(file_path))
        }
        "Bash" => {
            let command = match tool_input.get("command").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => std::process::exit(0),
            };
            guard.check_bash(command)
        }
        _ => std::process::exit(0),
    };

    match result {
        Ok(()) => std::process::exit(0),
        Err(msg) => {
            #[allow(clippy::print_stdout)]
            {
                println!("{}", msg);
            }
            std::process::exit(2);
        }
    }
}

/// Walk upward from `worktree` to find `.apm/config.toml` and return the
/// `IsolationConfig`. Falls back to defaults if not found.
fn load_isolation_config(worktree: &Path) -> apm_core::config::IsolationConfig {
    let mut dir = worktree;
    loop {
        let config_path = dir.join(".apm").join("config.toml");
        if config_path.exists() {
            if let Ok(config) = apm_core::config::Config::load(dir) {
                return config.isolation;
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    apm_core::config::IsolationConfig::default()
}
