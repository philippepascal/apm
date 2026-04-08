+++
id = "47d4695f"
title = "apm new: accept --section/--set flags to pre-populate spec sections"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "40799"
branch = "ticket/47d4695f-apm-new-accept-section-set-flags-to-pre-"
created_at = "2026-03-31T00:05:27.351459Z"
updated_at = "2026-03-31T05:04:58.353822Z"
+++

## Spec

### Problem

apm new accepts --no-edit to skip the interactive editor, but agents cannot pre-populate spec sections in a single command. Without section content, the ticket is created empty in `new` state and immediately eligible for pickup by a running `apm work` daemon — a worker may start writing the spec before the creating agent has a chance to fill it in.

Interactive users avoid this because the editor opens synchronously during `apm new`, keeping the ticket in a transient state until they save and close. Agents have no equivalent: they must create the ticket first, then make separate `apm spec` calls — a window where the ticket is vulnerable to premature worker pickup.

The fix is to allow `--section`/`--set` pairs on `apm new`, with the same API as `apm spec`. Sections are written into the ticket file before the first commit, so the ticket never exists in an empty `new` state.

Example:

```
apm new --no-edit "title" \
  --section Problem --set "What is broken..." \
  --section "Acceptance criteria" --set "- [ ] ..." \
  --section "Out of scope" --set "..." \
  --section Approach --set "..."
```

The ticket is created fully specd in a single atomic command.

### Acceptance criteria

- [x] `apm new --no-edit "title" --section Problem --set "text"` creates a ticket with the Problem section pre-populated in the initial git commit
- [x] Multiple `--section`/`--set` pairs apply all named sections atomically before the first commit
- [x] Mismatched pair counts (e.g. two `--section` flags but one `--set` flag, or vice versa) return a clear error
- [x] `--set` without `--section` returns an error consistent with `apm spec` behaviour
- [x] Section names are validated with the same rules as `apm spec` (known built-in names, or config-defined sections when `[ticket.sections]` is non-empty)
- [x] The ticket git history never contains an intermediate empty-section commit when sections are provided at creation time
- [x] All existing `apm new` flags (`--no-edit`, `--side-note`, `--context`, `--context-section`) continue to work unchanged
- [x] `cargo test --workspace` passes, including a new integration test for multi-section pre-population

### Out of scope

- `--mark` flag support on `apm new` (only `--set` is needed for creation)
- Changing spec validation behaviour or the `specd` transition rules
- Support for `--section`/`--set` on the interactive editor path (only applies with `--no-edit`)

### Approach

Add `section: Vec<String>` and `set: Vec<String>` args to the `New` subcommand in `main.rs`. Validate at the start of `cmd::new::run` that the two vecs have equal length (error if not), and that `--set` is not present without `--section`. Pass the pairs as `Vec<(String, String)>` into `ticket::create` in `apm-core/src/ticket.rs`.

Inside `ticket::create`, after the body template is built (and after the existing `--context` substitution), iterate over the section/value pairs and apply each one using `spec::set_section` (for built-in structured sections) or `spec::set_section_body` (for custom sections), using the same dispatch logic that `cmd::spec::run` already uses. Serialize the final body and pass it to `commit_to_branch` as normal — no extra commit is needed.

Add an integration test in `apm/tests/integration.rs` that calls `apm new --no-edit` with two `--section`/`--set` pairs and asserts that `git show` on the branch at HEAD contains the expected section content.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T00:05Z | — | new | apm |
| 2026-03-31T00:05Z | new | in_design | apm |
| 2026-03-31T04:35Z | in_design | new | apm |
| 2026-03-31T04:36Z | new | in_design | philippepascal |
| 2026-03-31T04:39Z | in_design | specd | claude-0330-0000-w47f |
| 2026-03-31T04:44Z | specd | ready | apm |
| 2026-03-31T04:45Z | ready | in_progress | philippepascal |
| 2026-03-31T04:56Z | in_progress | implemented | claude-0330-1445-w47d |
| 2026-03-31T05:01Z | implemented | accepted | apm-sync |
| 2026-03-31T05:04Z | accepted | closed | apm-sync |