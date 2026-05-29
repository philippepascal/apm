+++
id = "e54a7adf"
title = "Allow apm instructions in worker allow-list and stop denial scanner mislabeling approvals/cancellations"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e54a7adf-allow-apm-instructions-in-worker-allow-l"
created_at = "2026-05-29T18:47:07.214401Z"
updated_at = "2026-05-29T19:01:35.312689Z"
+++

## Spec

### Problem

Two bugs exposed by a worker that died in progress. The root cause from the real `.apm-worker.log`: the worker ran `apm instructions`, which is absent from the allow-list, so Claude Code returned "This command requires approval". Because the worker had batched it in parallel with other tool calls, Claude cancelled the entire parallel batch, producing seven `<tool_use_error>Cancelled: parallel tool call Bash(apm instructions) errored</tool_use_error>` results. The denial scanner then misread both the cancellations and the approval prompts as genuine permission denials, sending diagnosis badly astray.

**Part A** is the root cause: `apm instructions` is missing from `APM_ALLOW_ENTRIES` and `APM_USER_ALLOW_ENTRIES` in `apm/src/cmd/init.rs`. Every other `apm` subcommand appears in both lists; `instructions` was overlooked. The fix is a one-line addition to each array, using the same `Bash(apm instructions*)` glob form (no space before `*`) that matches both the bare invocation and any future flags.

**Part B** is the mis-diagnosis amplifier: `scan_transcript` in `apm-core/src/denial.rs` classifies every `is_error` tool result that does not start with `"Exit code "` as a denial. That rule is too broad. Cancellations (`"Cancelled: parallel tool call … errored"`) are collateral damage when a parallel sibling fails; they are not denials. Approval prompts (`"This command requires approval"`) are a config-gap signal distinct from an outright deny phrase (`"cannot be auto-allowed"`, `"was blocked. For security"`, `"but you haven't granted it yet"`). Mixing all three into one bucket produced nine reported "denials" of which zero were genuine.

### Acceptance criteria

- [ ] After `apm init --yes`, `.claude/settings.json` `permissions.allow` contains `"Bash(apm instructions*)"`
- [ ] After `apm init --yes`, `~/.claude/settings.json` `permissions.allow` contains `"Bash(apm instructions*)"`
- [ ] The integration test `init_yes_creates_settings_when_claude_dir_exists` asserts `"Bash(apm instructions*)"` is present in project settings
- [ ] The integration test `init_yes_updates_user_settings` asserts `"Bash(apm instructions*)"` is present in user settings
- [ ] `scan_transcript` returns `denial_count = 0` for a transcript whose only `is_error` results have content starting with `"Cancelled: parallel tool call"`
- [ ] `scan_transcript` classifies a result with content `"This command requires approval"` as `DenialClass::RequiresApproval`, not as `ApmCommandDenial` or `UnknownPattern`
- [ ] `scan_transcript` still classifies `"cannot be auto-allowed"` on a Bash `apm` command as `DenialClass::ApmCommandDenial`
- [ ] `DenialClass::RequiresApproval` serialises as `"requires_approval"` in `summary.json`

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T18:47Z | — | new | philippepascal |
| 2026-05-29T18:49Z | new | groomed | philippepascal |
| 2026-05-29T19:01Z | groomed | in_design | philippepascal |