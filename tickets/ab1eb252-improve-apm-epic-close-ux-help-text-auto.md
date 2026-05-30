+++
id = "ab1eb252"
title = "Improve apm epic close UX: help text, auto-sync mergeable tickets, --merge/--pr/--auto"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab1eb252-improve-apm-epic-close-ux-help-text-auto"
created_at = "2026-05-30T18:53:24.160398Z"
updated_at = "2026-05-30T19:09:12.793651Z"
+++

## Spec

### Problem

Three improvements to apm epic close (apm/src/cmd/epic.rs::run_close, lines 73-132):

1. Help text should briefly list the high-level steps the command performs: quiescence check, push epic branch, create or update PR, with a note that the branch is just deleted (no PR) when it is already merged into default. Today the help is one sentence and users do not know what the command is about to do.

2. When the quiescence check fails because tickets in the epic are still in non-closed states, the command should not just bail with the blocker list. It should detect tickets whose branches are already merged into the epic branch or the default branch and offer to close them automatically, the same way apm sync already prompts to close merged tickets. Tickets that genuinely need manual attention should still be listed as blockers; tickets that are merely waiting for the closing transition should be offered for auto-close.

3. Add --merge, --pr, and --auto flags mirroring the pattern already used by apm epic refresh (run_refresh_epic). Semantics:
   --merge does a working-tree merge of the epic branch into default and skips PR creation
   --pr (the current default behaviour) pushes the epic branch and opens or updates a PR
   --auto merges when the merge would be clean; falls back to opening a PR when it would conflict
The current default (push + open PR) should remain the default when no flag is given.

Reference: run_refresh_epic in apm/src/cmd/epic.rs already implements the --merge/--pr/--auto pattern and the merge_tree_status helper that distinguishes clean vs conflicted merges. The new flags on run_close should reuse the same helpers, not duplicate the logic.

### Acceptance criteria

- [ ] `apm epic close --help` describes at least three operational stages: quiescence check, the already-merged branch-delete shortcut (no PR), and the default push-and-open-PR path.
- [ ] `apm epic close --help` documents `--merge`, `--pr`, and `--auto` with one-line descriptions of each flag's semantics.
- [ ] When the quiescence check finds blocking tickets whose branches are already merged into the epic branch or the default branch and have no live worker, the command lists them and prompts "Close N merged ticket(s)? [y/N]" on a TTY.
- [ ] Accepting the auto-close prompt closes those tickets via `apm_core::ticket::close`; if no genuine blockers remain afterward, the command proceeds normally.
- [ ] Tickets with a live `.apm-worker.pid` are never included in the auto-close prompt — they appear only in the genuine-blocker error message.
- [ ] On a non-TTY, no prompt is shown; merged-but-unclosed tickets remain in the blocker error and the command exits non-zero.
- [ ] `apm epic close <id> --merge` merges the epic branch into the default branch locally and creates no PR.
- [ ] `apm epic close <id> --auto` merges locally when the merge would be clean; falls back to opening a PR when it would conflict.
- [ ] Without a flag (or with `--pr`), the command pushes the epic branch and opens or updates a PR — identical to the current behavior.
- [ ] `--merge`, `--pr`, and `--auto` are mutually exclusive; clap rejects combinations with a usage error.
- [ ] The already-merged shortcut (delete branch, skip PR/merge) is preserved regardless of which flag is given.

### Out of scope

- A `--yes` flag to bypass the auto-close prompt in non-TTY / scripted mode
- Changes to `apm sync`, `apm epic refresh`, or any other subcommand
- Modifying the quiescence definition (which states or conditions block)
- Auto-pushing the default branch after a `--merge` close
- Web UI changes (`apm-server` / `apm-ui`)

### Approach

#### `apm/src/main.rs` — CLI wiring

Update `EpicCommand::Close`:

- Replace the one-line `/// Open a PR…` doc comment with a short summary line (e.g. "Merge the epic branch into default or open a PR for it").
- Add `#[command(long_about = "...")]` with a multi-line string that walks through the four stages the command performs:
  1. Quiescence check — all epic tickets must be closed or otherwise quiescent (no live workers, no tickets stuck in in-progress states). Merged-but-unclosed tickets are offered for auto-close interactively.
  2. Already-merged shortcut — if the epic branch has no commits ahead of the default branch, the branch is deleted locally and remotely without creating a PR.
  3. `--pr` (default) — pushes the epic branch and creates or updates a GitHub PR targeting the default branch.
  4. `--merge` — merges the epic branch into the default branch in a local working tree; no PR is created. `--auto` merges locally when clean and falls back to `--pr` when there would be conflicts.
  Follow the existing `long_about` pattern used by the `Spec` command at line 782 of `main.rs`.
- Add three bool fields to the `Close` variant, mirroring `RefreshEpic`:
  ```rust
  #[arg(long, conflicts_with_all = ["pr", "auto_mode"])]
  merge: bool,
  #[arg(long, conflicts_with_all = ["merge", "auto_mode"])]
  pr: bool,
  #[arg(long = "auto", conflicts_with_all = ["merge", "pr"])]
  auto_mode: bool,
  ```
- Update the dispatch arm:
  ```rust
  Command::Epic { command: EpicCommand::Close { id, merge, pr, auto_mode } }
      => cmd::epic::run_close(&root, &id, merge, pr, auto_mode),
  ```

#### `apm-core/src/epic.rs` — Classification function

Add alongside `epic_is_quiescent` (keep that function unchanged — `run_refresh_epic` still calls it):

```rust
pub struct EpicQuiescenceResult {
    pub auto_closeable: Vec<crate::ticket::Ticket>,
    pub genuine_blockers: Vec<String>,   // same display format as existing blockers
}

pub fn classify_epic_quiescence(
    root: &Path,
    epic_id: &str,
    config: &crate::config::Config,
    worktrees: &[(std::path::PathBuf, String)],
    epic_branch: &str,
) -> anyhow::Result<EpicQuiescenceResult>
```

Logic (mirrors `epic_is_quiescent`'s per-ticket iteration):
1. Load all tickets for the epic.
2. For each ticket, apply the same `has_reached_impl && !terminal` guard — skip tickets that are not blocking.
3. For each blocking ticket:
   - If it has a live `.apm-worker.pid` → `genuine_blockers`.
   - Else, resolve the ticket branch (`frontmatter.branch` or `ticket_fmt::branch_name_from_path`). If the branch is merged into `epic_branch` OR into `config.project.default_branch` via `git_util::is_branch_merged_into` → `auto_closeable`.
   - Otherwise → `genuine_blockers`.
4. Return `EpicQuiescenceResult`.

#### `apm/src/cmd/epic.rs` — Update `run_close`

New signature:
```rust
pub fn run_close(root: &Path, id_arg: &str, merge: bool, pr: bool, auto_mode: bool) -> Result<()>
```

**Step 3 — quiescence check with auto-close offer:**

Replace the existing `epic_is_quiescent` bail block with:

```rust
let result = apm_core::epic::classify_epic_quiescence(
    root, epic_id, &config, &worktrees, &epic_branch,
)?;

if !result.auto_closeable.is_empty() && std::io::stdout().is_terminal() {
    let n = result.auto_closeable.len();
    println!("\nTickets merged but not yet closed ({n}):");
    for t in &result.auto_closeable {
        println!("  {}  {}", t.frontmatter.id, t.frontmatter.title);
    }
    if crate::util::prompt_yes_no(&format!("\nClose {n} merged ticket(s)? [y/N] "))? {
        let caller = apm_core::config::resolve_caller_name();
        let actor = format!("{}(apm-epic-close)", caller);
        for t in &result.auto_closeable {
            match apm_core::ticket::close(root, &config, &t.frontmatter.id, None, &actor, false) {
                Ok(msgs) => msgs.iter().for_each(|m| println!("{m}")),
                Err(e) => eprintln!("warning: could not close {}: {e:#}", t.frontmatter.id),
            }
        }
    }
}

// Re-check after any auto-closes to get the definitive blocker list.
let worktrees = apm_core::worktree::list_ticket_worktrees(root)?;
let blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?;
if !blockers.is_empty() {
    anyhow::bail!(
        "cannot close epic: the following tickets are not quiescent:\n{}",
        blockers.join("\n")
    );
}
```

**Steps 5-6 — flag-conditional merge or PR:**

After the already-merged shortcut (which stays unchanged), replace the existing push-and-PR block:

```rust
let do_merge = merge || (auto_mode && {
    let s = apm_core::epic::merge_tree_status(root, default_branch, &epic_branch)?;
    s.clean
});

if do_merge {
    // Find the main worktree root; it must be on the default branch.
    let main_root = apm_core::git_util::main_worktree_root(root)
        .unwrap_or_else(|| root.to_path_buf());
    let head = apm_core::git_util::run(&main_root, &["symbolic-ref", "--short", "HEAD"])
        .unwrap_or_default();
    if head.trim() != default_branch {
        anyhow::bail!(
            "cannot merge: main worktree is on '{}', not '{default_branch}'. \
             Check out {default_branch} first, or use --pr.",
            head.trim()
        );
    }
    let mut messages = vec![];
    match apm_core::git_util::merge_ref(&main_root, &epic_branch, &mut messages) {
        Some(msg) => {
            for m in &messages { println!("{m}"); }
            println!("{msg}");
        }
        None => anyhow::bail!(
            "merge conflict — resolve manually after checking out {default_branch}, \
             or use --pr to open a PR instead"
        ),
    }
} else {
    // push + PR: existing code unchanged
    apm_core::git::push_branch_tracking(root, &epic_branch)?;
    let mut messages = vec![];
    apm_core::github::gh_pr_create_or_update(
        root, &epic_branch, default_branch, epic_id, &pr_title,
        &format!("Epic: {epic_branch}"), &mut messages,
    )?;
    for m in &messages { println!("{m}"); }
}
```

Note: when `--merge` is given and `merge_ref` encounters a conflict, it returns `None` (it already aborts the merge). When `--auto` is given and the merge would conflict, `do_merge` is `false`, so `merge_ref` is never called and the PR path runs instead.

**Tests to add:**

- Unit test in `apm-core/src/epic.rs`: `classify_epic_quiescence_separates_merged_from_active` — one ticket whose branch is merged into the epic branch (auto-closeable) and one in `in_progress` with no merge (genuine blocker); assert each appears in the correct bucket.
- Integration test in `apm/tests/integration.rs`: `epic_close_auto_closes_merged_tickets` — set up an epic with one implemented ticket whose branch is merged into the epic branch; run `apm epic close` in non-TTY mode and assert it exits with a blocker message (since no prompt). Add a separate test variant that mocks TTY acceptance if feasible, or document that the auto-close path is covered by the unit test.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T18:53Z | — | new | philippepascal |
| 2026-05-30T18:57Z | new | groomed | philippepascal |
| 2026-05-30T19:01Z | groomed | in_design | philippepascal |