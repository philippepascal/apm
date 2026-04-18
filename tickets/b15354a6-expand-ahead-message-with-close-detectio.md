+++
id = "b15354a6"
title = "Expand ahead message with close-detection context and surface in UI sync"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b15354a6-expand-ahead-message-with-close-detectio"
created_at = "2026-04-18T02:21:44.835172Z"
updated_at = "2026-04-18T02:29:02.208567Z"
+++

## Spec

### Problem

When `apm sync` detects that local `<default>` is ahead of `origin/<default>`, it prints a message that is accurate but silent about the most important consequence: **close detection is gated on origin visibility**. `apm sync` detects merged tickets by inspecting commits reachable from `origin/<default>`; unpushed local commits are invisible to that check. Users have hit this as a mystery — sync reports "ahead by 16 commits" and shows "no tickets to close", then immediately offers to close tickets after a `git push`. The causal link is missing from the message.

There is also a parity gap between the CLI and UI sync surfaces. The server handler (`apm-server/src/handlers/maintenance.rs`) discards warnings from `sync_non_checked_out_refs` entirely (the accumulator is named `_sync_warnings` and never read), and routes warnings from `sync_default_branch` to `eprintln!` (server stderr) rather than into the JSON `log` field. As a result, the UI sync modal never shows "main is ahead" or any ahead-of-origin messages for non-checked-out ticket/epic refs, even when those gaps are precisely what is blocking close detection. Users running the UI today get no signal that their local main is out of sync with origin.

### Acceptance criteria

- [ ] `MAIN_AHEAD` in `apm-core/src/sync_guidance.rs` includes a sentence explaining that merged tickets will not be detected as closeable until the user pushes
- [ ] When `apm sync` (CLI) runs and local default branch is ahead of origin, the expanded message appears on stderr
- [ ] When `POST /api/sync` runs and local default branch is ahead of origin, the `log` field in the JSON response contains the expanded `MAIN_AHEAD` message
- [ ] When `POST /api/sync` runs and one or more non-checked-out ticket or epic refs are ahead of origin, those `TICKET_OR_EPIC_AHEAD` messages appear in the `log` field (currently the warnings vector is discarded)
- [ ] The UI sync modal displays the "ahead" message when local main is ahead of origin
- [ ] The UI sync modal displays per-branch ahead warnings when non-checked-out ticket/epic refs are ahead of origin
- [ ] `apm sync` (CLI) behaviour for the happy path (no ahead condition) is unchanged

### Out of scope

- Automatically pushing to origin (sync still never pushes; the user must push explicitly)
- Changing the JSON response shape — `log` stays as a newline-joined string, `branches` and `closed` stay as integers
- Restructuring the sync modal UI beyond displaying the existing `log` field (no redesign)
- Surfacing other existing warnings (diverged, dirty-overlap) that are also currently lost in the server path — those are a separate concern
- Adding a `TICKET_OR_EPIC_AHEAD` close-detection note; ticket/epic branches being ahead of origin does not block close detection (only the default branch does), so the wording change there is out of scope

### Approach

Two files change; order does not matter.

**1. `apm-core/src/sync_guidance.rs` — expand `MAIN_AHEAD` wording**

Update the constant body at line 67 from:

```
<default> is ahead of <remote> by <count> <commits> — run `git push` when ready
```

to something like:

```
<default> is ahead of <remote> by <count> <commits>. Merged tickets will not be detected as closeable until you push — run `git push` when ready.
```

Exact wording is the implementer's call; the requirement is that the user learns *why* pushing matters (close detection), not just the bare fact of being ahead. The placeholder substitution mechanism is unchanged (`<default>`, `<remote>`, `<count>`, `<commits>` are replaced by the caller at the print site in `git_util.rs`).

`TICKET_OR_EPIC_AHEAD` (line 73) does not need a close-detection note — ticket/epic branches being ahead of origin does not block close detection.

**2. `apm-server/src/handlers/maintenance.rs` — surface warnings in the JSON `log`**

There are two separate bugs in `sync_handler` (lines 11-61):

**Bug A — `sync_non_checked_out_refs` warnings discarded (lines 23-25):**

Current:
```rust
let mut _sync_warnings: Vec<String> = Vec::new();
apm_core::git::sync_non_checked_out_refs(&root, &mut _sync_warnings);
log.push("synced non-checked-out refs".to_string());
```

Fix: use a real vector and extend `log` with it before the status line:
```rust
let mut ref_warnings: Vec<String> = Vec::new();
apm_core::git::sync_non_checked_out_refs(&root, &mut ref_warnings);
log.extend(ref_warnings);
log.push("synced non-checked-out refs".to_string());
```

**Bug B — `sync_default_branch` warnings sent to stderr only (lines 31-35):**

Current:
```rust
let mut sync_warnings: Vec<String> = Vec::new();
apm_core::git::sync_default_branch(&root, &config.project.default_branch, &mut sync_warnings);
for w in &sync_warnings {
    eprintln!("warning: {w}");
}
```

Fix: extend `log` with the warnings (keep or drop the `eprintln!` as preferred — dropping it avoids duplicate output):
```rust
let mut sync_warnings: Vec<String> = Vec::new();
apm_core::git::sync_default_branch(&root, &config.project.default_branch, &mut sync_warnings);
log.extend(sync_warnings);
```

No changes are required to `apm-ui/src/components/SyncModal.tsx` — it already renders the entire `log` string as pre-formatted text, so once the server includes the warnings in `log` they will appear automatically.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:29Z | groomed | in_design | philippepascal |