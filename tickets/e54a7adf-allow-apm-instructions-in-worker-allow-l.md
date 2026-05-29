+++
id = "e54a7adf"
title = "Allow apm instructions in worker allow-list and stop denial scanner mislabeling approvals/cancellations"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e54a7adf-allow-apm-instructions-in-worker-allow-l"
created_at = "2026-05-29T18:47:07.214401Z"
updated_at = "2026-05-29T20:15:46.129654Z"
+++

## Spec

### Problem

Two bugs exposed by a worker that died in progress. The root cause from the real `.apm-worker.log`: the worker ran `apm instructions`, which is absent from the allow-list, so Claude Code returned "This command requires approval". Because the worker had batched it in parallel with other tool calls, Claude cancelled the entire parallel batch, producing seven `<tool_use_error>Cancelled: parallel tool call Bash(apm instructions) errored</tool_use_error>` results. The denial scanner then misread both the cancellations and the approval prompts as genuine permission denials, sending diagnosis badly astray.

**Part A** is the root cause: `apm instructions` is missing from `APM_ALLOW_ENTRIES` and `APM_USER_ALLOW_ENTRIES` in `apm/src/cmd/init.rs`. Every other `apm` subcommand appears in both lists; `instructions` was overlooked. The fix is a one-line addition to each array, using the same `Bash(apm instructions*)` glob form (no space before `*`) that matches both the bare invocation and any future flags.

**Part B** is the mis-diagnosis amplifier: `scan_transcript` in `apm-core/src/denial.rs` classifies every `is_error` tool result that does not start with `"Exit code "` as a denial. That rule is too broad. Cancellations (`"Cancelled: parallel tool call … errored"`) are collateral damage when a parallel sibling fails; they are not denials. Approval prompts (`"This command requires approval"`) are a config-gap signal distinct from an outright deny phrase (`"cannot be auto-allowed"`, `"was blocked. For security"`, `"but you haven't granted it yet"`). Mixing all three into one bucket produced nine reported "denials" of which zero were genuine.

### Acceptance criteria

- [x] After `apm init --yes`, `.claude/settings.json` `permissions.allow` contains `"Bash(apm instructions*)"`
- [x] After `apm init --yes`, `~/.claude/settings.json` `permissions.allow` contains `"Bash(apm instructions*)"`
- [x] The integration test `init_yes_creates_settings_when_claude_dir_exists` asserts `"Bash(apm instructions*)"` is present in project settings
- [x] The integration test `init_yes_updates_user_settings` asserts `"Bash(apm instructions*)"` is present in user settings
- [x] `scan_transcript` returns `denial_count = 0` for a transcript whose only `is_error` results have content starting with `"Cancelled: parallel tool call"`
- [x] `scan_transcript` classifies a result with content `"This command requires approval"` as `DenialClass::RequiresApproval`, not as `ApmCommandDenial` or `UnknownPattern`
- [x] `scan_transcript` still classifies `"cannot be auto-allowed"` on a Bash `apm` command as `DenialClass::ApmCommandDenial`
- [x] `DenialClass::RequiresApproval` serialises as `"requires_approval"` in `summary.json`

### Out of scope

- Telling workers to skip `apm instructions` (its output is already baked into the system prompt by `build_system_prompt`; removing it from worker startup is a separate concern)
- Changing `collect_unique_apm_commands` to also surface `RequiresApproval` entries
- apm-server and apm-ui changes
- Any changes to how `DenialSummary` is displayed or consumed beyond accurate classification

### Approach

#### Part A — allow-list entries (`apm/src/cmd/init.rs`)

Add `"Bash(apm instructions*)"` to `APM_ALLOW_ENTRIES` alongside the other `apm` subcommands (e.g., after `"Bash(apm version*)"`). Do the same in `APM_USER_ALLOW_ENTRIES`.

Pattern rationale: `Bash(apm instructions*)` (no space before `*`) matches the bare `apm instructions` invocation and any future flags, consistent with how other zero-or-one-arg subcommands are written in the same list (`Bash(apm sync*)`, `Bash(apm version*)`, etc.). The broken form `Bash(apm instructions *)` (space before `*`) requires at least one argument and would miss the bare call.

Update two integration tests in `apm/tests/integration.rs`:
- `init_yes_creates_settings_when_claude_dir_exists` — add `assert!(entries.contains(&"Bash(apm instructions*)"), "Bash(apm instructions*) missing");`
- `init_yes_updates_user_settings` — add the same assertion checking the user settings entries

#### Part B — denial scanner (`apm-core/src/denial.rs`)

**Enum change:** add `RequiresApproval` to `DenialClass`. It serialises as `"requires_approval"` automatically via the existing `#[serde(rename_all = "snake_case")]` attribute.

**Scanner change:** in `scan_transcript` pass 2, after the `"Exit code "` early-continue and before calling `classify_denial`, add two short-circuit checks in order:

1. `content_str.contains("Cancelled: parallel tool call")` → `continue` (collateral cancellation; not a denial; drop entirely from the count)
2. `content_str.contains("requires approval")` → extract the command string the same way `classify_denial` does for the relevant tool (Bash: `input_obj["command"]`; others: serialise `input_obj`), push a `DenialEntry` with `classification: DenialClass::RequiresApproval`, then `continue`

`classify_denial` itself is unchanged; all three new paths are handled at the call site where `content_str` is already in scope.

**Unit tests:** add two inline tests to the existing `#[cfg(test)] mod tests` block (inline style, using `tempfile::tempdir()` and `std::fs::write`, consistent with `test_regular_error_not_classified_as_denial`):

- `test_cancelled_parallel_not_a_denial` — transcript with one Bash tool whose result content is `"Cancelled: parallel tool call Bash(apm instructions) errored"`; assert `denial_count == 0` and `denials.is_empty()`
- `test_requires_approval_classified_as_requires_approval` — transcript with a Bash tool (`apm instructions`) whose result content is `"This command requires approval"`; assert `denial_count == 1` and `denials[0].classification == DenialClass::RequiresApproval`

The existing `test_apm_command_denial` fixture (`transcript_apm_denial.jsonl`) uses content `"apm doesnotexist cannot be auto-allowed"` — it continues to pass unchanged, verifying genuine denials are still classified as `ApmCommandDenial`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T18:47Z | — | new | philippepascal |
| 2026-05-29T18:49Z | new | groomed | philippepascal |
| 2026-05-29T19:01Z | groomed | in_design | philippepascal |
| 2026-05-29T19:04Z | in_design | specd | claude |
| 2026-05-29T19:22Z | specd | ready | philippepascal |
| 2026-05-29T19:24Z | ready | in_progress | philippepascal |
| 2026-05-29T19:28Z | in_progress | implemented | claude |
| 2026-05-29T20:15Z | implemented | closed | philippepascal(apm-sync) |
