use std::path::Path;
use serde_json::Value;

/// Write a `PreToolUse` hook entry to `<worktree>/.claude/settings.json` that
/// intercepts `Edit`, `Write`, and `Bash` tool calls by running `apm path-guard`.
///
/// The function is idempotent: if an entry with the same command already exists,
/// it is not duplicated.
pub fn write_hook_config(worktree: &Path, apm_bin: &str) -> anyhow::Result<()> {
    let claude_dir = worktree.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let settings_path = claude_dir.join("settings.json");
    let content = std::fs::read_to_string(&settings_path).unwrap_or_else(|_| "{}".to_string());
    let mut settings: Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

    // Ensure settings is an object
    if !settings.is_object() {
        settings = serde_json::json!({});
    }

    // Navigate to hooks -> PreToolUse, creating as needed
    let hooks = settings
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }

    let pre_tool_use = hooks
        .as_object_mut()
        .unwrap()
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));

    if !pre_tool_use.is_array() {
        *pre_tool_use = serde_json::json!([]);
    }

    let hook_command = format!("{} path-guard", apm_bin);

    // Check for an existing entry with this command (idempotent)
    let already_present = pre_tool_use.as_array().unwrap().iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.ends_with("apm path-guard"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    if !already_present {
        pre_tool_use.as_array_mut().unwrap().push(serde_json::json!({
            "matcher": "Edit|Write|Bash",
            "hooks": [{"type": "command", "command": hook_command}]
        }));
    }

    let json_str = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, json_str)?;

    Ok(())
}

/// Remove the `apm path-guard` hook entry from `<worktree>/.claude/settings.json`.
///
/// Called after the worker process exits to avoid leaving stale hooks in long-lived
/// worktrees. If the file does not exist, this is a no-op.
pub fn remove_hook_config(worktree: &Path) -> anyhow::Result<()> {
    let settings_path = worktree.join(".claude").join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)?;
    let mut settings: Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

    if let Some(pre_tool_use) = settings
        .get_mut("hooks")
        .and_then(|h| h.get_mut("PreToolUse"))
        .and_then(|p| p.as_array_mut())
    {
        pre_tool_use.retain(|entry| {
            !entry
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.ends_with("apm path-guard"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });
    }

    let json_str = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, json_str)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_hook_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        write_hook_config(tmp.path(), "/usr/bin/apm").unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        assert!(settings_path.exists());
        let content = std::fs::read_to_string(&settings_path).unwrap();
        assert!(content.contains("apm path-guard"));
        assert!(content.contains("PreToolUse"));
        assert!(content.contains("Edit|Write|Bash"));
    }

    #[test]
    fn write_hook_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        write_hook_config(tmp.path(), "/usr/bin/apm").unwrap();
        write_hook_config(tmp.path(), "/usr/bin/apm").unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 1, "hook entry should not be duplicated");
    }

    #[test]
    fn write_hook_preserves_existing_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let existing = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Edit", "hooks": [{"type": "command", "command": "other-hook"}]}
                ]
            }
        });
        std::fs::write(&settings_path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        write_hook_config(tmp.path(), "/usr/bin/apm").unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 2, "existing entry must be preserved");
        assert!(content.contains("other-hook"));
        assert!(content.contains("apm path-guard"));
    }

    #[test]
    fn remove_hook_removes_entry() {
        let tmp = tempfile::tempdir().unwrap();
        write_hook_config(tmp.path(), "/usr/bin/apm").unwrap();
        remove_hook_config(tmp.path()).unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 0, "hook entry should be removed");
    }

    #[test]
    fn remove_hook_noop_when_no_file() {
        let tmp = tempfile::tempdir().unwrap();
        // Should not error when settings.json doesn't exist
        remove_hook_config(tmp.path()).unwrap();
    }

    #[test]
    fn remove_hook_preserves_other_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        let existing = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"matcher": "Edit", "hooks": [{"type": "command", "command": "other-hook"}]},
                    {"matcher": "Edit|Write|Bash", "hooks": [{"type": "command", "command": "/usr/bin/apm path-guard"}]}
                ]
            }
        });
        std::fs::write(&settings_path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

        remove_hook_config(tmp.path()).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        let arr = v["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 1, "other entry must be preserved");
        assert!(content.contains("other-hook"));
        assert!(!content.contains("apm path-guard"));
    }
}
