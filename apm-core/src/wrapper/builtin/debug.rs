use super::write_and_spawn_script;
use crate::wrapper::{Wrapper, WrapperContext};

pub struct DebugWrapper;

impl Wrapper for DebugWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        let script = r#"#!/bin/sh
env | grep '^APM_' >&2
printf '\n=== SYSTEM PROMPT ===\n' >&2
cat "$APM_SYSTEM_PROMPT_FILE" >&2
printf '\n=== USER MESSAGE ===\n' >&2
cat "$APM_USER_MESSAGE_FILE" >&2
printf '{"type":"tool_use","id":"debug-1","name":"noop","input":{}}\n'
rm -f "$0"
"#;
        write_and_spawn_script("debug", script, ctx)
    }
}
