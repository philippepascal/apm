+++
id = "d3ebdc0f"
title = "Create util.rs with shared CLI helpers"
state = "implemented"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3ebdc0f-create-util-rs-with-shared-cli-helpers"
created_at = "2026-04-12T09:02:33.251574Z"
updated_at = "2026-04-12T10:36:02.874767Z"
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

- [x] `apm/src/util.rs` exists and is declared as `pub mod util;` in `apm/src/lib.rs`
- [x] `util::fetch_if_aggressive(root: &Path, aggressive: bool)` calls `git::fetch_all(root)` when `aggressive` is true and emits `"warning: fetch failed: {e:#}"` on error
- [x] `util::fetch_branch_if_aggressive(root: &Path, branch: &str, aggressive: bool)` calls `git::fetch_branch(root, branch)` when `aggressive` is true and emits `"warning: fetch failed: {e:#}"` on error
- [x] `util::prompt_yes_no(prompt: &str) -> io::Result<bool>` prints the prompt to stdout, flushes, reads one line from stdin, and returns `true` if and only if the trimmed input equals `"y"` (case-insensitive)
- [x] `next.rs` and `sync.rs` (fetch-all users) replace their inline fetch block with a call to `util::fetch_if_aggressive`
- [x] `assign.rs`, `show.rs`, `close.rs`, and `spec.rs` (fetch-branch users) replace their inline fetch block with a call to `util::fetch_branch_if_aggressive`
- [x] `ctx.rs`'s `CmdContext::load` replaces its inline fetch block with `util::fetch_if_aggressive`
- [x] `assign.rs`, `sync.rs`, and `clean.rs` replace their inline confirmation sequences with calls to `util::prompt_yes_no`
- [x] The literal string `"warning: fetch failed: {e:#}"` appears in exactly one place in the codebase (`util.rs`)
- [x] `cargo test` passes with no regressions after all replacements

### Out of scope

- Push-warning deduplication (`"warning: push failed: {e:#}"` in `assign.rs` and `spec.rs`) — separate concern, low frequency
- Changing any user-visible behaviour or output beyond normalising the one divergent fetch warning in `sync.rs`
- Adding terminal-detection logic to `prompt_yes_no` — `clean.rs` has an `is_terminal()` guard in one call site; that guard stays inline in `clean.rs`
- Restructuring `CmdContext` beyond the single fetch-block swap
- Extracting other helpers not listed in the Approach (e.g. root detection, config loading)

### Approach

**1. Create `apm/src/util.rs`**

Add the following three public functions:

```rust
use std::io::{self, BufRead, Write};
use std::path::Path;
use apm_core::git;

/// Run `git fetch --all` when `aggressive` is true; emit a warning on failure.
pub fn fetch_if_aggressive(root: &Path, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Run `git fetch <branch>` when `aggressive` is true; emit a warning on failure.
pub fn fetch_branch_if_aggressive(root: &Path, branch: &str, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_branch(root, branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Print `prompt`, flush stdout, read one line, return true iff the answer is "y".
pub fn prompt_yes_no(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}
```

**2. Declare the module in `apm/src/lib.rs`**

Add `pub mod util;` alongside the existing module declarations.

**3. Update `ctx.rs`**

In `CmdContext::load`, replace:
```rust
if aggressive {
    if let Err(e) = git::fetch_all(root) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
```
with:
```rust
crate::util::fetch_if_aggressive(root, aggressive);
```
Remove the now-unused `git` import if it is only referenced there.

**4. Update fetch-all callers: `next.rs`, `sync.rs`**

Replace the inline block in each file with:
```rust
crate::util::fetch_if_aggressive(root, aggressive);
```
`sync.rs`'s slightly different warning string (`"warning: fetch failed (no remote configured?): {e:#}"`) is normalised to the standard string. Remove any `use` imports that become unused.

**5. Update fetch-branch callers: `assign.rs`, `show.rs`, `close.rs`, `spec.rs`**

Each of these computes `aggressive` then calls `git::fetch_branch`. Replace with:
```rust
crate::util::fetch_branch_if_aggressive(root, &branch, aggressive);
```
`spec.rs` calls this pattern twice (once per sub-command that does a fetch); replace both.

**6. Update confirmation-prompt callers: `assign.rs`, `sync.rs`, `clean.rs`**

Replace each `print! / flush / read_line / trim / eq_ignore_ascii_case` block with:
```rust
crate::util::prompt_yes_no("...prompt text...")?
```
`clean.rs` has three such blocks. One of them is guarded by an `is_terminal()` check — leave that guard in place; only replace the inner prompt sequence. Remove unused `use std::io` items if they become unnecessary after the swap.

**Order**: create util.rs → update lib.rs → ctx.rs → next.rs / sync.rs → assign.rs / show.rs / close.rs / spec.rs → clean.rs. Run `cargo build` after each file to catch import issues early.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:08Z | new | groomed | apm |
| 2026-04-12T09:09Z | groomed | in_design | philippepascal |
| 2026-04-12T09:12Z | in_design | specd | claude-0412-0909-d5f8 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T10:27Z | ready | in_progress | philippepascal |
| 2026-04-12T10:36Z | in_progress | implemented | claude-0412-1027-85e0 |
