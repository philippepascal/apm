+++
id = "29cc63a1"
title = "Pre-merge leak detection: refuse apm state implemented when main has uncommitted overlap"
state = "implemented"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29cc63a1-pre-merge-leak-detection-refuse-apm-stat"
created_at = "2026-05-01T02:30:13.061854Z"
updated_at = "2026-05-02T19:07:33.018481Z"
+++

## Spec

### Problem

When a worker writes to the main worktree (intentional leak or bug), the bad change sits there until someone notices via `git status` or fails an `apm state implemented` merge. The deferred enforcement piece from ticket 498febe0's spec is what closes this gap.

**Incident pattern:**
1. Worker spawns into its ticket worktree.
2. Worker (despite path-discipline guidance in apm.worker.md) issues a tool call with an absolute path pointing at the main worktree.
3. The call may succeed (if the file is in the project's allowlist or the worker was spawned with -P) or fail (default permission denial). When it succeeds, the change is silent.
4. Later, when the supervisor runs `apm state X implemented`, the merge of the ticket branch into main aborts because the main worktree has uncommitted changes that would be overwritten — but the error message is git's stock "Aborting" which doesn't point at the worker that caused it.
5. Cleanup requires the supervisor to identify the leaked file, decide whether to commit/discard, and re-attempt the merge.

**This ticket adds a pre-merge check that catches the leak earlier with a clearer diagnostic.**

**Reference:** ticket 498febe0's spec (already implemented) explicitly listed this as out of scope ("a defensive check in apm state implemented that fails fast when the main worktree is dirty for files the ticket changed"). Now is the time.

**Should land after the wrapper epic (4312fbd4)** so the wrapper-side path validator (separate ticket) and this check are layered together.

**Scope:**
- In `apm-core/src/state.rs`, before the merge attempt in the `Merge` and `PrOrEpicMerge` completion strategies:
  - Compute the set of files modified on the ticket branch since its merge-base with the target (main, or the epic branch).
  - Run `git status --porcelain` on the target worktree.
  - If any of the modified files appear in the status output as uncommitted: refuse the transition with a clear diagnostic naming each leaked file, the ticket id, and a pointer to the worker's transcript at `<worktree>/.apm-worker.log`.
  - On clean: proceed with the merge as today.
- The check is informational — does not modify the working tree or revert changes.
- New error message format:
  ```
  cannot complete <transition>: main worktree has uncommitted changes to files this ticket also modified:
    apm-ui/src/components/foo.tsx
    .apm/config.toml
  This usually means a worker leaked edits outside its worktree.
  Inspect the worker's transcript: <ticket-worktree>/.apm-worker.log
  Then either commit/restore the leaked files in main and re-run apm state <id> implemented, or run apm verify to investigate.
  ```

**Out of scope:**
- Auto-recovering the leak (move uncommitted changes to a stash, etc.). The supervisor decides; this ticket only surfaces.
- Pre-spawn checks (the leak hasn't happened yet).
- Wrapper-layer interception of tool calls (separate ticket).

**Acceptance pointers:**
- Integration test: simulate a leak by creating an uncommitted edit in the main worktree on a file the ticket branch also modified. `apm state X implemented` exits non-zero with the new diagnostic. The exit text names the leaked file. The ticket state remains at `in_progress` (no transition occurred).
- Integration test: clean main worktree → `apm state X implemented` proceeds normally.
- Integration test: the `Pr` and `None` completion strategies (no merge attempted) are unaffected.

### Acceptance criteria

- [x] `apm state X implemented` exits non-zero and prints a diagnostic when the merge-target worktree has uncommitted changes to at least one file that also appears in `git diff --name-only <merge-base>..<ticket-branch>`
- [x] The diagnostic names every overlapping file (one per line, indented)
- [x] The diagnostic includes the ticket id
- [x] The diagnostic includes the path `<ticket-worktree>/.apm-worker.log` (or a generic placeholder when the worktree is not found)
- [x] When the check fires, the ticket state on the branch remains unchanged (no `implemented` commit, no `on_failure` rollback commit)
- [x] `apm state X implemented` succeeds normally when the merge-target worktree has no uncommitted changes
- [x] `apm state X implemented` succeeds normally when the merge-target worktree has uncommitted changes to files that are NOT on the ticket branch (no false positives)
- [x] The check runs for the `Merge` completion strategy (direct merge to `target_branch` or `default_branch`)
- [x] The check runs for the `PrOrEpicMerge` completion strategy when `target_branch` is set (epic-branch merge path)
- [x] `apm state X implemented` is unaffected when the completion strategy is `Pr`, `Pull`, or `None` (no merge is attempted; no check runs)
- [x] When `check_leaked_files` cannot resolve the merge-base (e.g. no shared history), it returns an empty list and the transition is not blocked
- [x] When the merge-target worktree does not exist on disk yet, the check returns empty and does not block the transition
- [x] `git status --porcelain` entries with `R` or `C` in the X column (staged renames/copies) in the target worktree are skipped during dirty-file enumeration and are never reported as leaks (known limitation: a leaked file staged as a rename in the target worktree will not be detected)
- [x] An untracked file (`??` prefix in `git status --porcelain`) in the target worktree is included in the dirty set; if the ticket branch also added that same file, it is reported as a potential leak

### Out of scope

- Auto-recovery of leaked edits (stashing, reverting, or moving changes on behalf of the user)
- Pre-spawn checks to prevent workers from leaking in the first place
- Wrapper-layer tool-call interception (separate ticket in epic 4312fbd4)
- Checking for leaks in worktrees other than the merge target (e.g. other ticket worktrees)
- The `Pull` completion strategy (pulls upstream into ticket branch; no merge of ticket into target)
- The `Pr` strategy's no-merge path (no worktree merge is attempted)
- Detecting leaks that have already been committed to the main branch (those cause a normal merge conflict, not a worker leak)
- `apm verify` command (referenced in the diagnostic as a follow-up tool; implementation is out of scope here)

### Approach

Two files change: `apm-core/src/git_util.rs` (new function) and `apm-core/src/state.rs` (call site + branch-computation move). Four integration tests are added.

#### New function: `git_util::check_leaked_files`

Signature — no `Config` parameter; the function needs only the repo root and branch names:

```rust
/// Returns the list of files that are both modified on `ticket_branch`
/// (since its merge-base with `target_branch`) AND dirty (uncommitted) in the
/// target worktree.  Returns an empty Vec when the check cannot be performed
/// (no shared history, target worktree not found on disk).
pub fn check_leaked_files(
    root: &Path,
    ticket_branch: &str,
    target_branch: &str,
) -> Result<Vec<String>> {
    // 1. Resolve the target worktree directory.
    let current = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();

    let merge_dir = if current_branch == target_branch {
        root.to_path_buf()
    } else {
        match crate::worktree::find_worktree_for_branch(root, target_branch) {
            Some(p) => p,
            None => return Ok(vec![]),  // target worktree absent -> cannot be dirty
        }
    };

    // 2. Compute merge-base between target and ticket.
    let base = match merge_base(root, target_branch, ticket_branch) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Ok(vec![]),  // no shared history -> don't block
    };
    if base.is_empty() {
        return Ok(vec![]);
    }

    // 3. Files touched by the ticket branch since the merge-base (includes newly
    //    added files, which appear as untracked in the target if leaked).
    let diff_out = Command::new("git")
        .args(["diff", "--name-only", &base, ticket_branch])
        .current_dir(root)
        .output()?;
    let ticket_files: std::collections::HashSet<String> =
        String::from_utf8_lossy(&diff_out.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

    // 4. Dirty files in the target worktree.
    //    Porcelain v1 format: "XY <path>" -- path starts at column 3.
    //    "??" (untracked) entries are intentionally included: a file added by the
    //    ticket branch that sits untracked in the target is a genuine leak signal.
    //    "R " and "C " (staged rename/copy) entries are skipped: their line format
    //    is "XY orig -> dest", so col-3 slicing produces "orig -> dest", not a
    //    matchable path.  Known limitation: leaks of staged-renamed files are not
    //    detected.
    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&merge_dir)
        .output()?;
    let dirty_files: std::collections::HashSet<String> =
        String::from_utf8_lossy(&status_out.stdout)
            .lines()
            .filter_map(|line| {
                if line.len() < 3 {
                    return None;
                }
                let x = line.as_bytes()[0] as char;
                let y = line.as_bytes()[1] as char;
                // Skip rename/copy entries: cannot be parsed with a simple col-3 slice.
                if x == 'R' || x == 'C' || y == 'R' || y == 'C' {
                    return None;
                }
                Some(line[3..].to_string())
            })
            .collect();

    // 5. Intersection, sorted for stable output.
    let mut overlap: Vec<String> = ticket_files
        .intersection(&dirty_files)
        .cloned()
        .collect();
    overlap.sort();
    Ok(overlap)
}
```

#### Changes to `apm-core/src/state.rs`

**Step 1 — move `branch` computation earlier.**

The five lines that compute `branch` (currently lines 128–133, after the `match new_state` block) must move to just before that block. No logic change — only reordering.

**Step 2 — add leak check inside the `"implemented"` arm.**

After the existing acceptance-criteria check (currently lines 100–108), insert:

```rust
// Pre-merge leak detection: refuse if the target worktree has uncommitted
// overlap with files this ticket modified.
let should_check = match &completion {
    CompletionStrategy::Merge => true,
    CompletionStrategy::PrOrEpicMerge => t.frontmatter.target_branch.is_some(),
    _ => false,
};
if should_check {
    let merge_target = t.frontmatter.target_branch.as_deref()
        .unwrap_or(config.project.default_branch.as_str());
    let leaked = git::check_leaked_files(root, &branch, merge_target)?;
    if !leaked.is_empty() {
        let file_list = leaked
            .iter()
            .map(|f| format!("  {f}"))
            .collect::<Vec<_>>()
            .join("\n");
        let log_hint = crate::worktree::find_worktree_for_branch(root, &branch)
            .map(|p| p.join(".apm-worker.log").to_string_lossy().into_owned())
            .unwrap_or_else(|| "<ticket-worktree>/.apm-worker.log".to_string());
        bail!(
            "cannot complete {}: the target worktree has uncommitted changes \
             to files this ticket also modified:\n{}\n\
             This usually means a worker leaked edits outside its worktree.\n\
             Inspect the worker's transcript: {}\n\
             Then either commit/restore the leaked files and re-run \
             `apm state {} implemented`, or run `apm verify` to investigate.",
            new_state, file_list, log_hint, id
        );
    }
}
```

Placing the check inside `"implemented"` (before the state mutation) ensures the ticket stays unchanged if the check fires — no `implemented` commit lands, no `on_failure` rollback is needed.

#### Tests (add to `apm/tests/integration.rs`)

Use the existing `setup_merge()` helper (already configures `merge` completion strategy, `in_progress → implemented` transition).

**`state_implemented_refuses_when_main_dirty_overlap`**
1. `setup_merge()` → repo root `p`.
2. Create and commit `src/foo.rs` on `main`.
3. Create ticket branch; check it out; modify `src/foo.rs`; commit.
4. Switch back to `main`.
5. Modify `src/foo.rs` in the working tree — do NOT commit (simulates leaked edit).
6. Create a ticket file in `in_progress` state on the ticket branch.
7. Run `apm state <id> implemented`; assert exit code is non-zero; assert output contains `src/foo.rs`; assert ticket state is still `in_progress`.

**`state_implemented_proceeds_when_main_clean`**
1. `setup_merge()`.
2. Commit `src/foo.rs` on `main`.
3. Create ticket branch; modify `src/foo.rs`; commit; check all AC boxes.
4. Run `apm state <id> implemented`; assert success.

**`state_implemented_proceeds_when_dirty_no_overlap`**
1. `setup_merge()`.
2. Commit `src/foo.rs` and `src/bar.rs` on `main`.
3. Ticket branch modifies only `src/foo.rs`.
4. `src/bar.rs` is left dirty (uncommitted) in `main` — no overlap with ticket.
5. Run `apm state <id> implemented`; assert success (no false positive).

**`state_implemented_refuses_when_main_has_untracked_overlap`**
1. `setup_merge()`.
2. On `main`, do NOT create `src/new.rs` (file does not exist there yet).
3. Create ticket branch; add `src/new.rs` as a new file; commit.
4. Switch back to `main`; create `src/new.rs` in the working tree without staging it (simulates a worker dropping a new file in main).
5. Run `apm state <id> implemented`; assert exit code is non-zero; assert output contains `src/new.rs`.

Add to `apm-core/src/git_util.rs` (inside the `#[cfg(test)]` block) a unit test for `check_leaked_files` covering: overlap case, no-overlap case, and untracked-file overlap case. Use the existing `git_init()` / `git_cmd()` helpers.

#### No changes required

`apm/src/cmd/state.rs`, `apm-core/src/worktree.rs`, `apm-core/src/config.rs`, and the existing `on_failure` error-handling paths (those remain for actual merge conflicts; the leak check short-circuits before the merge starts).

### New function: `git_util::check_leaked_files`

Add to `apm-core/src/git_util.rs`:

```rust
/// Returns the list of files that are both modified on `ticket_branch`
/// (since its merge-base with `target_branch`) AND dirty (uncommitted) in the
/// target worktree.  Returns an empty Vec when the check cannot be performed
/// (no shared history, target worktree not found on disk).
pub fn check_leaked_files(
    root: &Path,
    config: &Config,
    ticket_branch: &str,
    target_branch: &str,
) -> Result<Vec<String>> {
    // 1. Resolve the target worktree directory — same logic as merge_into_default
    //    but without creating the worktree (no creation = no side effects).
    let current = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();

    let merge_dir = if current_branch == target_branch {
        root.to_path_buf()
    } else {
        match crate::worktree::find_worktree_for_branch(root, target_branch) {
            Some(p) => p,
            None => return Ok(vec![]),  // target worktree absent → cannot be dirty
        }
    };

    // 2. Compute merge-base between target and ticket.
    let base = match merge_base(root, target_branch, ticket_branch) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Ok(vec![]),  // no shared history → don't block
    };
    if base.is_empty() {
        return Ok(vec![]);
    }

    // 3. Files touched by the ticket branch since the merge-base.
    let diff_out = Command::new("git")
        .args(["diff", "--name-only", &base, ticket_branch])
        .current_dir(root)
        .output()?;
    let ticket_files: std::collections::HashSet<String> =
        String::from_utf8_lossy(&diff_out.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

    // 4. Dirty files in the target worktree (staged, unstaged, untracked).
    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&merge_dir)
        .output()?;
    let dirty_files: std::collections::HashSet<String> =
        String::from_utf8_lossy(&status_out.stdout)
            .lines()
            .filter_map(|line| {
                // porcelain format: "XY <path>" — path starts at col 3
                if line.len() > 3 { Some(line[3..].to_string()) } else { None }
            })
            .collect();

    // 5. Intersection, sorted for stable output.
    let mut overlap: Vec<String> = ticket_files
        .intersection(&dirty_files)
        .cloned()
        .collect();
    overlap.sort();
    Ok(overlap)
}
```

The `Config` parameter is needed only to satisfy a possible future signature extension; the current implementation uses only `root` and the git commands. If it turns out `Config` is truly unnecessary, remove it and update the call sites.

---

### State machine changes: `apm-core/src/state.rs`

**Step 1 — move `branch` computation earlier.**

Lines 128-133 currently compute `branch` after the `match new_state` block. Move those five lines to just before the `match new_state.as_str()` block (i.e., before line 84). No logic change — only position.

**Step 2 — add leak check inside the `"implemented"` arm.**

After the existing acceptance-criteria check (lines 100-108), insert:

```rust
// Pre-merge leak detection: refuse if the target worktree has uncommitted
// overlap with files this ticket modified.
let should_check = match &completion {
    CompletionStrategy::Merge => true,
    CompletionStrategy::PrOrEpicMerge => t.frontmatter.target_branch.is_some(),
    _ => false,
};
if should_check {
    let merge_target = t.frontmatter.target_branch.as_deref()
        .unwrap_or(config.project.default_branch.as_str());
    let leaked = git::check_leaked_files(root, &config, &branch, merge_target)?;
    if !leaked.is_empty() {
        let file_list = leaked
            .iter()
            .map(|f| format!("  {f}"))
            .collect::<Vec<_>>()
            .join("\n");
        let log_hint = crate::worktree::find_worktree_for_branch(root, &branch)
            .map(|p| p.join(".apm-worker.log").to_string_lossy().into_owned())
            .unwrap_or_else(|| "<ticket-worktree>/.apm-worker.log".to_string());
        bail!(
            "cannot complete {new_state}: the target worktree has uncommitted changes \
             to files this ticket also modified:\n{file_list}\n\
             This usually means a worker leaked edits outside its worktree.\n\
             Inspect the worker's transcript: {log_hint}\n\
             Then either commit/restore the leaked files and re-run \
             `apm state {id} implemented`, or run `apm verify` to investigate."
        );
    }
}
```

Placing the check inside `"implemented"` (before the state mutation at line 115) ensures the ticket stays unchanged if the check fails — no `implemented` commit, no rollback commit needed.

---

### Tests

Add three `#[test]` functions to `apm/tests/integration.rs`, using an extended version of the existing `setup_merge()` helper (which already configures the `merge` completion strategy and the `in_progress → implemented` transition).

**`state_implemented_refuses_when_main_dirty_overlap`**
1. `setup_merge()` → repo root `p`.
2. Create and commit `src/foo.rs` on `main`.
3. Create ticket branch `ticket/abc-test`; check it out; modify `src/foo.rs`; commit.
4. Switch back to `main`.
5. Modify `src/foo.rs` in the working tree — do NOT commit (simulates leaked edit).
6. Create a ticket file in `in_progress` state on the ticket branch using `apm state`.
7. Run `apm state <id> implemented`; assert exit code is non-zero; assert stderr/stdout contains `src/foo.rs`; assert the ticket state is still `in_progress`.

**`state_implemented_proceeds_when_main_clean`**
1. `setup_merge()`.
2. Commit `src/foo.rs` on `main`.
3. Create ticket branch; modify `src/foo.rs`; commit; check all AC boxes.
4. Run `apm state <id> implemented`; assert success.

**`state_implemented_proceeds_when_dirty_no_overlap`**
1. `setup_merge()`.
2. Commit `src/foo.rs` and `src/bar.rs` on `main`.
3. Ticket branch modifies only `src/foo.rs`.
4. `src/bar.rs` is modified uncommitted in `main` (no overlap with ticket).
5. Run `apm state <id> implemented`; assert success (no false positive).

Add to `apm-core/src/git_util.rs` (inside the `#[cfg(test)]` block) a unit test for `check_leaked_files` covering the overlap and no-overlap cases, using the existing `git_init()` / `git_cmd()` helpers already present in that file.

---

### No changes required to

- `apm/src/cmd/state.rs` (thin CLI wrapper; unchanged)
- `apm-core/src/worktree.rs` (only `find_worktree_for_branch` is called; already public)
- `apm-core/src/config.rs` (no new config keys)
- The `Merge` / `PrOrEpicMerge` error-handling paths (those remain for actual merge failures; the leak check is a pre-flight that short-circuits before the merge even starts)

### Open questions


### Amendment requests

- [x] Drop the `Config` parameter from `check_leaked_files`. The Approach itself admits "the `Config` parameter is needed only to satisfy a possible future signature extension; the current implementation uses only `root` and the git commands." That is speculative-design — narrow the signature to `(root, ticket_branch, target_branch)`. If a future caller needs config, add it then.

- [x] Porcelain path slicing is wrong for renames. `line[3..]` works for `M `, `??`, etc., but `R ` and `C ` entries use `R  old -> new` — slicing at column 3 captures `old -> new` as one path string and silently misses leaks of renamed files. Either filter out R/C entries (acceptable simplification with an AC noting the limitation) or parse the rename arrow. Pick one and state it in the spec; do not leave the bug latent.

- [x] Untracked-file overlap semantics are ambiguous. Step 4 includes `??` lines in `dirty_files`, but step 3's diff against merge-base only enumerates tracked files. Pin the behaviour with an AC, e.g. "untracked file in target worktree that the ticket branch added is reported as a leak", and a corresponding test — or explicitly exclude `??` from the dirty set with an AC noting the gap.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:30Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:14Z | groomed | in_design | philippepascal |
| 2026-05-02T03:21Z | in_design | specd | claude-0502-0314-7430 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T07:50Z | ammend | in_design | philippepascal |
| 2026-05-02T07:55Z | in_design | specd | claude-0502-0750-45f8 |
| 2026-05-02T18:21Z | specd | ready | philippepascal |
| 2026-05-02T18:53Z | ready | in_progress | philippepascal |
| 2026-05-02T19:07Z | in_progress | implemented | claude-0502-1853-5758 |
