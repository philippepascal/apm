+++
id = "d3ebdc0f"
title = "Create util.rs with shared CLI helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3ebdc0f-create-util-rs-with-shared-cli-helpers"
created_at = "2026-04-12T09:02:33.251574Z"
updated_at = "2026-04-12T09:09:29.821591Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

Several boilerplate patterns are copy-pasted across the `apm/src/cmd/` command files with no shared home:

1. **Aggressive fetch check** – 6 command files (`assign.rs`, `show.rs`, `next.rs`, `close.rs`, `spec.rs`, `sync.rs`) each inline the same two-line block: compute `aggressive`, call `git::fetch_all` or `git::fetch_branch`, and emit a warning on failure. `ctx.rs` already encapsulates the `fetch_all` variant for commands that use `CmdContext::load()`, but the `fetch_branch` variant (used when a specific branch is known) is still duplicated across four files.

2. **Fetch/push warning strings** – The string `"warning: fetch failed: {e:#}"` appears verbatim in 5 files; a one-character typo fix would require touching all five. (`sync.rs` has a slightly different message with an extra hint, which is an accidental divergence.)

3. **Confirmation prompt** – `assign.rs`, `sync.rs`, and `clean.rs` each re-implement the same `print! / flush / read_line / trim / eq_ignore_ascii_case("y")` sequence. `clean.rs` uses it three times internally.

There is no `util.rs` module today. Creating one with `fetch_if_aggressive`, `fetch_branch_if_aggressive`, and `prompt_yes_no` would eliminate all three duplication classes and give future commands a single place to reach for these primitives.

### Acceptance criteria

- [ ] `apm/src/util.rs` exists and is declared as `pub mod util;` in `apm/src/lib.rs`
- [ ] `util::fetch_if_aggressive(root: &Path, aggressive: bool)` calls `git::fetch_all(root)` when `aggressive` is true and emits `"warning: fetch failed: {e:#}"` on error
- [ ] `util::fetch_branch_if_aggressive(root: &Path, branch: &str, aggressive: bool)` calls `git::fetch_branch(root, branch)` when `aggressive` is true and emits `"warning: fetch failed: {e:#}"` on error
- [ ] `util::prompt_yes_no(prompt: &str) -> io::Result<bool>` prints the prompt to stdout, flushes, reads one line from stdin, and returns `true` if and only if the trimmed input equals `"y"` (case-insensitive)
- [ ] `next.rs` and `sync.rs` (fetch-all users) replace their inline fetch block with a call to `util::fetch_if_aggressive`
- [ ] `assign.rs`, `show.rs`, `close.rs`, and `spec.rs` (fetch-branch users) replace their inline fetch block with a call to `util::fetch_branch_if_aggressive`
- [ ] `ctx.rs`'s `CmdContext::load` replaces its inline fetch block with `util::fetch_if_aggressive`
- [ ] `assign.rs`, `sync.rs`, and `clean.rs` replace their inline confirmation sequences with calls to `util::prompt_yes_no`
- [ ] The literal string `"warning: fetch failed: {e:#}"` appears in exactly one place in the codebase (`util.rs`)
- [ ] `cargo test` passes with no regressions after all replacements

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
| 2026-04-12T09:08Z | new | groomed | apm |
| 2026-04-12T09:09Z | groomed | in_design | philippepascal |