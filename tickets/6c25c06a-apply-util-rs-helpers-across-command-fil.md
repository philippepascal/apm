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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:21Z | groomed | in_design | philippepascal |