+++
id = "8b8fd3a9"
title = "apm sync, when doing a push of main, doesn't display push/hook outputs"
state = "in_design"
priority = 0
effort = 1
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8b8fd3a9-apm-sync-when-doing-a-push-of-main-doesn"
created_at = "2026-06-13T18:33:03.661566Z"
updated_at = "2026-06-14T06:01:10.792960Z"
+++

## Spec

### Problem

`apm sync` calls `push_branch` (defined in `apm-core/src/git_util.rs`) when pushing the default branch or ahead ticket/epic branches. `push_branch` delegates to the internal `run()` helper, which uses `Command::output()` — this captures both stdout and stderr of the git process. On success, both streams are discarded; on failure, only the captured stderr is surfaced as an error string. Remote push hooks (e.g. CI gate hooks, lint hooks, custom server-side checks) emit their messages through the git process's stderr, so those messages are silently swallowed every time the push succeeds.

Users who have hooks installed on the remote see nothing — no confirmation the hook ran, no warnings or status lines the hook produced. The fix is to stream git's output directly to the terminal rather than capturing it.

### Acceptance criteria

- [ ] `apm sync` (when pushing the default branch) streams all git push output — including remote hook messages — to the terminal
- [ ] `apm sync` (when pushing ahead ticket/epic branches) streams all git push output to the terminal
- [ ] When a push fails, `apm sync` still prints a `warning: push failed` line and continues syncing other refs (current error-handling behaviour unchanged)
- [ ] `apm sync --quiet` still suppresses the APM-added confirmation line ("pushed main to origin") but does not suppress git's own output, including hook messages

### Out of scope

- `push_branch_tracking` (used by `apm state` ticket transitions) — same root cause but a different call site and user flow; separate ticket if desired
- Streaming output for `push_ticket_branches` (the `push_refs` path in `util.rs`) — it uses `run()` directly, not `push_branch`, and is a separate code path
- Suppressing git push progress output under `--quiet` — hook output is not APM output and should always be visible

### Approach

**File:** `apm-core/src/git_util.rs`

Replace the body of `push_branch` (line 967–969). Currently it delegates to `run()`, which calls `Command::output()` and discards all captured output on success. Change it to use `Command::status()` instead, which inherits the parent process's stdin/stdout/stderr and lets all git output (including remote hook stderr) flow directly to the terminal:

```rust
pub fn push_branch(root: &Path, branch: &str) -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["push", "origin", &format!("{branch}:{branch}")])
        .current_dir(root)
        .status()?;
    if !status.success() {
        anyhow::bail!("git push failed with exit code {}", status.code().unwrap_or(-1));
    }
    Ok(())
}
```

**Error message note:** With `.status()`, stderr is no longer captured, so the error string can no longer echo git's output. This is fine — git has already printed the error to the terminal. The `warning: push failed: …` line in `sync.rs` still appears as a structured APM-level signal that the push failed.

**All callers benefit automatically:**
- `apm/src/cmd/sync.rs` line 75 — default branch push (the primary case)
- `apm/src/cmd/sync.rs` line 92 — ticket/epic branch pushes
- `apm-core/src/git_util.rs` line 1256 — merge-then-push inside `state implemented`
- `apm-core/src/ticket/ticket_util.rs` line 365 — aggressive sync auto-close push

No changes to `sync.rs` are needed. The existing `if let Err(e) = git::push_branch(...)` / `eprintln!("warning: push failed: {e:#}")` pattern still works correctly.

No new tests are required — the change is a one-function substitution with no behavioural branches to cover. Existing tests do not mock git push and will continue to pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-13T18:33Z | — | new | philippepascal |
| 2026-06-14T05:57Z | new | groomed | philippepascal |
| 2026-06-14T05:57Z | groomed | in_design | philippepascal |