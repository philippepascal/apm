+++
id = "9f4869d6"
title = "refactor: move review transition and body manipulation logic into apm-core"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "40953"
branch = "ticket/9f4869d6-refactor-move-review-transition-and-body"
created_at = "2026-03-30T14:27:50.402284Z"
updated_at = "2026-03-30T18:08:54.636067Z"
+++

## Spec

### Problem

review.rs (321 lines) mixes editor orchestration — a CLI concern — with document-manipulation logic that belongs in apm-core:

- split_body: splits a ticket body into editable spec and preserved history
- extract_spec: strips the editor-header from a saved temp file to recover the spec text
- manual_transitions (aka available_transitions): reads the config to determine which transitions a supervisor can trigger manually (filters out event: auto-triggers)
- normalise_amendment_checkboxes: rewrites plain - bullets in ### Amendment requests to - [ ] checkboxes when a ticket is transitioning to ammend state

Currently all four live in apm/src/cmd/review.rs alongside editor temp-file management, the $VISUAL/$EDITOR/vi invocation, and the interactive stdin prompt. This coupling means apm-serve — which will let a supervisor approve or request amendments via a web UI without a local editor — cannot reuse the logic without depending on the CLI crate.

Moving the document-manipulation functions into apm_core::review gives apm-serve (and tests) a stable, editor-free API surface. The CLI keeps open_editor, build_header, and prompt_transition; it calls into apm_core::review for everything else.

### Acceptance criteria

- [x] `apm_core::review` is a public module exported from `apm-core/src/lib.rs`
- [x] `apm_core::review::split_body` splits a body at `\n## History` (or `## History` at line start) into a `(spec, history)` tuple
- [x] `apm_core::review::split_body` returns `(full_body, "")` when no History section is present
- [x] `apm_core::review::extract_spec` returns everything after the sentinel line when the sentinel is present
- [x] `apm_core::review::extract_spec` strips leading `# ` comment lines as fallback when the sentinel was deleted
- [x] `apm_core::review::available_transitions` returns only transitions whose `trigger` does not start with `event:`
- [x] `apm_core::review::available_transitions` falls back to all non-terminal, non-current states when no explicit transitions are configured for a state
- [x] `apm_core::review::normalize_amendments` converts plain `- ` bullet lines inside `### Amendment requests` to `- [ ] ` checkboxes
- [x] `apm_core::review::normalize_amendments` leaves `- [ ]`, `- [x]`, and `- [X]` lines unchanged
- [x] `apm_core::review::normalize_amendments` leaves lines outside `### Amendment requests` unchanged
- [x] `apm_core::review::apply_review` returns `new_spec` trimmed of trailing whitespace concatenated with `history_section`
- [x] `apm/src/cmd/review.rs` imports and delegates to `apm_core::review` for all five moved functions; no duplicate implementations remain in the CLI crate
- [x] `apm/src/cmd/review.rs` retains `open_editor`, `build_header`, and `prompt_transition` exclusively
- [x] `cargo test --workspace` passes with no regressions
- [x] Each moved function has at least one unit test in `apm-core/src/review.rs`

### Out of scope

- Changing the behaviour of any moved function — this is a pure structural refactor
- Moving `build_header`, `open_editor`, or `prompt_transition` (these are CLI-only concerns)
- Adding a public `ReviewTransition` type; callers receive `(to, label, hint)` tuples and can wrap them locally
- `apm-serve` integration — this ticket only creates the library surface; wiring it into a web handler is a separate ticket

### Approach

1. **Create `apm-core/src/review.rs`** with five public functions:
   - `split_body(body: &str) -> (String, String)` — moved verbatim from `review.rs`
   - `extract_spec(content: &str) -> String` — moved verbatim; keep the `SENTINEL` constant here too
   - `available_transitions(config: &Config, current_state: &str) -> Vec<(String, String, String)>` — refactored from `manual_transitions`; returns `(to, label, hint)` tuples so the CLI needs no exported struct
   - `normalize_amendments(spec: String) -> String` — moved verbatim from `normalise_amendment_checkboxes` (renamed to drop British spelling)
   - `apply_review(new_spec: &str, history_section: &str) -> String` — thin helper: `format!("{}{}", new_spec.trim_end(), history_section)`

2. **Expose the module**: add `pub mod review;` to `apm-core/src/lib.rs`.

3. **Update `apm/src/cmd/review.rs`**:
   - Remove the four functions that moved and the `TransitionOption` struct
   - Keep `TransitionOption` as a private CLI struct built from the tuples returned by `available_transitions`
   - Replace all call sites to use the `apm_core::review::*` equivalents
   - Remove the local `SENTINEL` constant; import it from `apm_core::review::SENTINEL` (make it `pub const`)

4. **Add unit tests** in `apm-core/src/review.rs` (inline `#[cfg(test)]` block) covering the happy path and key edge cases for each of the five functions.

5. **Run `cargo test --workspace`** and confirm no regressions before committing.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:35Z | new | in_design | philippepascal |
| 2026-03-30T16:40Z | in_design | specd | claude-0330-1700-sp01 |
| 2026-03-30T17:00Z | specd | ready | philippepascal |
| 2026-03-30T17:24Z | ready | in_progress | philippepascal |
| 2026-03-30T17:29Z | in_progress | implemented | claude-0330-1800-wk01 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |