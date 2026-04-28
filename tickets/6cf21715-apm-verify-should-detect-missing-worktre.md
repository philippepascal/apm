+++
id = "6cf21715"
title = "apm verify should detect missing worktree for active-state tickets"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6cf21715-apm-verify-should-detect-missing-worktre"
created_at = "2026-04-28T00:50:59.455196Z"
updated_at = "2026-04-28T06:09:14.625480Z"
+++

## Spec

### Problem

`apm verify` currently checks for unknown states, ID/filename mismatches, missing branches on active tickets, merged-but-open branches, and missing spec/history sections. It does not check whether a ticket's worktree directory actually exists on disk.

When a ticket is in `in_design` or `in_progress`, `apm start` has been called and a worktree should be present at `{worktrees_base}/{branch.replace("/", "-")}`. If that directory is deleted (e.g., the repo was re-cloned, the worktrees sibling directory was wiped, or the worktree was force-removed without resetting ticket state), the ticket becomes silently stuck: no agent can work on it and no tooling flags it.

Real incident: ticket ec5e9fe3 was in `in_progress`. `apm worktrees` listed an entry for it at `…/apm--worktrees/ticket-ec5e9fe3-…`. The directory did not exist on disk. `apm verify` ran cleanly and reported no issues.

The fix is to walk every non-terminal ticket whose state is in `{in_design, in_progress}`, compute its expected worktree path, and emit an issue if the directory is absent. `--fix` should not auto-recreate the missing worktree because recreation would silently discard any uncommitted work that may still exist in another clone — a human decision is required (re-provision via `apm start <id>`, or revert state to `ready`).

### Acceptance criteria

- [x] `apm verify` reports an issue for a ticket in `in_design` state whose branch's expected worktree directory does not exist on disk
- [x] `apm verify` reports an issue for a ticket in `in_progress` state whose branch's expected worktree directory does not exist on disk
- [x] The reported issue message for a missing worktree matches the format `#{id} [{state}]: worktree at <path> is missing`
- [x] `apm verify` does not report a worktree issue for a ticket in `in_design` or `in_progress` state when its worktree directory exists on disk
- [x] `apm verify` does not report a worktree issue for a ticket in `in_design` or `in_progress` when no `branch` field is set (the existing "state requires branch but none set" issue fires instead)
- [x] `apm verify` does not report a worktree issue for tickets in states outside `{in_design, in_progress}` (e.g., `specd`, `implemented`, `closed`) even when the computed path is absent
- [x] `apm verify --fix` does not auto-recreate missing worktrees; the issue is printed and the process exits non-zero, same as without `--fix`

### Out of scope

- Auto-recreating missing worktrees via `--fix`
- Detecting stale git worktree registrations (entries in `git worktree list` that point to deleted directories) — a separate concern
- Worktree checks for `implemented`, `blocked`, or any other state outside `{in_design, in_progress}`
- Recreating or repairing the underlying git metadata for the missing worktree

### Approach

**Files changed:**

1. `apm-core/src/verify.rs` — add the worktree-presence check
2. `apm/src/cmd/verify.rs` — update the call site to pass `root`
3. `apm-core/tests/verify.rs` — new test file covering the new check

---

**`apm-core/src/verify.rs`**

Add `root: &Path` as a new first parameter to `verify_tickets`. The full new signature:

```rust
pub fn verify_tickets(
    root: &Path,
    config: &Config,
    tickets: &[Ticket],
    merged: &HashSet<String>,
) -> Vec<String>
```

Before the per-ticket loop, compute the worktrees base path once:

```rust
let worktree_states: HashSet<&str> =
    ["in_design", "in_progress"].iter().copied().collect();
let main_root = crate::git_util::main_worktree_root(root)
    .unwrap_or_else(|| root.to_path_buf());
let worktrees_base = main_root.join(&config.worktrees.dir);
```

Inside the per-ticket loop, after the existing "state requires branch" check, add:

```rust
// in_design/in_progress with missing worktree directory.
if worktree_states.contains(fm.state.as_str()) {
    if let Some(branch) = &fm.branch {
        let wt_name = branch.replace('/', "-");
        let wt_path = worktrees_base.join(&wt_name);
        if !wt_path.is_dir() {
            issues.push(format!(
                "{prefix}: worktree at {} is missing",
                wt_path.display()
            ));
        }
    }
}
```

The check mirrors the path logic in `worktree::ensure_worktree` exactly: `branch.replace('/', "-")` joined onto `worktrees_base`.

---

**`apm/src/cmd/verify.rs`**

Update the single call to `verify_tickets` to pass `root` as the new first argument:

```rust
let issues = apm_core::verify::verify_tickets(root, &ctx.config, &ctx.tickets, &merged_set);
```

No other change needed; `--fix` / `apply_fixes` is unaffected.

---

**`apm-core/tests/verify.rs`** (new file)

Use the existing test pattern from `apm-core/tests/ticket_create.rs`: initialize a real git repo in a `TempDir`, write an `apm.toml` that sets `[worktrees] dir = "worktrees"` (inside the temp dir, so path assertions are predictable), then write ticket markdown files directly and call `verify_tickets`.

Three tests:

1. **`worktree_missing_in_design`** — ticket in `in_design`, branch set, worktree dir absent → issue fired with correct message.
2. **`worktree_present_no_issue`** — same ticket, but `std::fs::create_dir_all` the expected path first → no worktree issue.
3. **`worktree_check_skipped_for_other_states`** — ticket in `specd` with branch set, worktree absent → no worktree issue.

Because `main_worktree_root` runs `git worktree list --porcelain` in the temp dir (a real git repo), it returns the temp dir path, so `worktrees_base = temp_dir/worktrees` and the computed `wt_path` is fully inside the temp dir — no path-outside-root awkwardness.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:50Z | — | new | philippepascal |
| 2026-04-28T00:51Z | new | groomed | philippepascal |
| 2026-04-28T01:06Z | groomed | in_design | philippepascal |
| 2026-04-28T01:11Z | in_design | specd | claude-0428-0106-fd50 |
| 2026-04-28T06:00Z | specd | ready | philippepascal |
| 2026-04-28T06:09Z | ready | in_progress | philippepascal |
