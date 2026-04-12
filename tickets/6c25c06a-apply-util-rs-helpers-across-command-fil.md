+++
id = "6c25c06a"
title = "Apply util.rs helpers across command files"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6c25c06a-apply-util-rs-helpers-across-command-fil"
created_at = "2026-04-12T09:02:44.386660Z"
updated_at = "2026-04-12T09:21:22.655145Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f"]
+++

## Spec

### Problem

After `apm/src/util.rs` is introduced by the prerequisite ticket (d3ebdc0f), six command files continue to inline the same boilerplate patterns rather than delegating to the shared helpers. The duplicated logic is:\n\n- **Aggressive fetch (fetch_all variant):** `if aggressive { if let Err(e) = git::fetch_all(root) { eprintln!("warning: fetch failed: {e:#}"); } }` — appears in `next.rs` and `sync.rs`\n- **Aggressive fetch (fetch_branch variant):** same structure but calls `git::fetch_branch(root, branch)` — appears in `assign.rs`, `show.rs`, `close.rs`, `spec.rs`\n- **Confirmation prompt:** inline stdin read with flush — appears in `assign.rs` and as a local helper function `prompt_close` in `sync.rs`\n\nThis duplication means any change to fetch-warning wording or prompt behaviour must be made in multiple places. Centralising the patterns into `util::fetch_if_aggressive`, `util::fetch_branch_if_aggressive`, and `util::prompt_yes_no` removes that maintenance burden.\n\n`new.rs` is listed in the ticket title but on inspection does not contain an inline fetch block — it computes the `aggressive` flag and forwards it to `ticket::create()` in apm-core. No change is needed there.

### Acceptance criteria

- [ ] `apm/src/util.rs` is declared as `pub mod util` in `apm/src/lib.rs`\n- [ ] `next.rs` no longer contains an inline `if aggressive { git::fetch_all … }` block\n- [ ] `sync.rs` no longer contains an inline `if !offline { … git::fetch_all … }` fetch block\n- [ ] `assign.rs` no longer contains an inline `if aggressive { git::fetch_branch … }` block\n- [ ] `show.rs` no longer contains an inline `if aggressive { git::fetch_branch … }` block\n- [ ] `close.rs` no longer contains an inline `if aggressive { … git::fetch_branch … }` block\n- [ ] `spec.rs` no longer contains an inline `if aggressive { git::fetch_branch … }` block\n- [ ] `assign.rs` no longer contains an inline stdin-prompt block for reassignment confirmation\n- [ ] `sync.rs` no longer contains the local `prompt_close` helper function\n- [ ] Each replaced call site compiles without warnings (`cargo build -p apm` succeeds)\n- [ ] `cargo test -p apm` passes with no regressions\n- [ ] Files that no longer use `std::io` directly have their now-unused `std::io` imports removed

### Out of scope

- Push-after-write patterns (`git::push_branch` blocks in `assign.rs` and `spec.rs`) — these are not fetch helpers and are not covered by `util.rs`\n- `clean.rs` and `epic.rs` — handled by separate tickets in this epic to avoid conflicts\n- `new.rs` — the aggressive flag is forwarded to `ticket::create()` in apm-core; there is no inline fetch block to remove\n- Adding new util helpers beyond the three already defined in d3ebdc0f\n- Changing the behaviour of aggressive fetch or the prompt (the wording in `sync.rs` currently reads "no remote configured?" — normalising it to the standard wording is an acceptable side-effect, not a goal)\n- Unit tests for the util helpers themselves (covered by d3ebdc0f)

### Approach

All changes are in `apm/src/`. The prerequisite ticket (d3ebdc0f) must be merged first so `util.rs` and its three helpers exist.\n\n**1. `apm/src/lib.rs`** — add `pub mod util;` alongside the existing mod declarations.\n\n**2. `apm/src/cmd/next.rs`** — replace the inline fetch block:\n```rust\n// before\nif aggressive {\n    if let Err(e) = git::fetch_all(root) {\n        eprintln!("warning: fetch failed: {e:#}");\n    }\n}\n// after\ncrate::util::fetch_if_aggressive(root, aggressive);\n```\nRemove any `use std::io` that becomes unused.\n\n**3. `apm/src/cmd/show.rs`** — replace the inline fetch_branch block:\n```rust\n// before\nif aggressive {\n    if let Err(e) = git::fetch_branch(root, &branch) {\n        eprintln!("warning: fetch failed: {e:#}");\n    }\n}\n// after\ncrate::util::fetch_branch_if_aggressive(root, &branch, aggressive);\n```\n\n**4. `apm/src/cmd/close.rs`** — the branch is wrapped in `Option`; unwrap before passing:\n```rust\n// before\nif aggressive {\n    if let Some(ref b) = branch {\n        if let Err(e) = git::fetch_branch(root, b) {\n            eprintln!("warning: fetch failed: {e:#}");\n        }\n    }\n}\n// after\nif let Some(ref b) = branch {\n    crate::util::fetch_branch_if_aggressive(root, b, aggressive);\n}\n```\n\n**5. `apm/src/cmd/spec.rs`** — one inline fetch_branch block (the push block is out of scope):\n```rust\n// before\nif aggressive {\n    if let Err(e) = git::fetch_branch(root, &branch) {\n        eprintln!("warning: fetch failed: {e:#}");\n    }\n}\n// after\ncrate::util::fetch_branch_if_aggressive(root, &branch, aggressive);\n```\n\n**6. `apm/src/cmd/assign.rs`** — two replacements:\n\na) Inline fetch_branch block (same pattern as show.rs):\n```rust\ncrate::util::fetch_branch_if_aggressive(root, &branch, aggressive);\n```\n\nb) Inline confirmation prompt (lines ~46-54). The `confirm_override` short-circuit is kept; only the `None` arm changes:\n```rust\n// before (None arm)\nprint!("Ticket {id} is currently owned by {current_owner}. Reassign to {username}? [y/N] ");\nio::stdout().flush()?;\nlet mut line = String::new();\nio::stdin().lock().read_line(&mut line)?;\nline.trim().eq_ignore_ascii_case("y")\n// after (None arm)\ncrate::util::prompt_yes_no(\n    &format!("Ticket {id} is currently owned by {current_owner}. Reassign to {username}? [y/N] ")\n)?\n```\nRemove `use std::io::{self, Write, BufRead}` if nothing else in the file uses it.\n\n**7. `apm/src/cmd/sync.rs`** — two replacements:\n\na) Inline fetch_all block. The existing block wraps fetch in a broader `if !offline` and also calls `git::sync_local_ticket_refs`. Only the fetch-and-warn sub-block moves to the helper; the `sync_local_ticket_refs` call stays:\n```rust\n// before\nif !offline {\n    match git::fetch_all(root) {\n        Ok(_) => { git::sync_local_ticket_refs(root, &mut sync_warnings); }\n        Err(e) => { eprintln!("warning: fetch failed (no remote configured?): {e:#}"); }\n    }\n}\n// after\ncrate::util::fetch_if_aggressive(root, !offline);\nif !offline {\n    git::sync_local_ticket_refs(root, &mut sync_warnings);\n}\n```\n(The error message is slightly normalised — this is acceptable.)\n\nb) Delete the local `fn prompt_close` helper and replace its single call site:\n```rust\n// before\nif prompt_close(&candidates)? { … }\n// after\nif crate::util::prompt_yes_no("\nClose all? [y/N] ")? { … }\n```\nRemove `use std::io::{self, BufRead, Write}` if nothing else in the file uses it after both replacements.\n\n**Verification:** run `cargo build -p apm` and `cargo test -p apm` from the repo root; expect zero errors and no new warnings.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:21Z | groomed | in_design | philippepascal |