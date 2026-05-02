use super::write_and_spawn_script;
use crate::wrapper::{Wrapper, WrapperContext};

pub(crate) const DEBUG_SCRIPT: &str = r#"#!/bin/sh
env | grep '^APM_' >&2
printf '\n=== SYSTEM PROMPT ===\n' >&2
cat "$APM_SYSTEM_PROMPT_FILE" >&2
printf '\n=== USER MESSAGE ===\n' >&2
cat "$APM_USER_MESSAGE_FILE" >&2
printf '{"type":"tool_use","id":"debug-1","name":"noop","input":{}}\n'
rm -f "$0"
"#;

pub struct DebugWrapper;

impl Wrapper for DebugWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        write_and_spawn_script("debug", DEBUG_SCRIPT, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::DEBUG_SCRIPT;

    #[test]
    fn script_dumps_apm_env_to_stderr() {
        assert!(
            DEBUG_SCRIPT.contains("env | grep '^APM_' >&2"),
            "script must redirect APM_ env vars to stderr"
        );
    }

    #[test]
    fn script_emits_one_canonical_event() {
        let canonical_lines: Vec<&str> = DEBUG_SCRIPT
            .lines()
            .filter(|l| l.starts_with("printf '{") && !l.contains(">&2"))
            .collect();
        assert_eq!(canonical_lines.len(), 1, "expected exactly one stdout JSONL line: {canonical_lines:?}");
        assert!(canonical_lines[0].contains("\"type\":\"tool_use\""), "canonical event must have type=tool_use");
    }

    #[test]
    fn script_outputs_system_prompt_and_user_message_to_stderr() {
        assert!(DEBUG_SCRIPT.contains("cat \"$APM_SYSTEM_PROMPT_FILE\" >&2"));
        assert!(DEBUG_SCRIPT.contains("cat \"$APM_USER_MESSAGE_FILE\" >&2"));
    }

    #[test]
    fn script_self_cleans_up() {
        assert!(DEBUG_SCRIPT.contains("rm -f \"$0\""), "script must remove itself after running");
    }
}
