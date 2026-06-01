+++
id = "ab1eb252"
title = "Improve apm epic close UX: help text, auto-sync mergeable tickets, --merge/--pr/--auto"
state = "merge_failed"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab1eb252-improve-apm-epic-close-ux-help-text-auto"
created_at = "2026-05-30T18:53:24.160398Z"
updated_at = "2026-06-01T08:31:41.091077Z"
depends_on = ["e96593f5"]
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

- [x] `apm epic close --help` describes at least three operational stages: quiescence check, the already-merged branch-delete shortcut (no PR), and the default push-and-open-PR path.
- [x] `apm epic close --help` documents `--merge`, `--pr`, and `--auto` with one-line descriptions of each flag's semantics.
- [x] When the quiescence check finds blocking tickets whose branches are already merged into the epic branch or the default branch and have no live worker, the command lists them and prompts "Close N merged ticket(s)? [y/N]" on a TTY.
- [x] Accepting the auto-close prompt closes those tickets via `apm_core::ticket::close`; if no genuine blockers remain afterward, the command proceeds normally.
- [x] Tickets with a live `.apm-worker.pid` are never included in the auto-close prompt — they appear only in the genuine-blocker error message.
- [x] On a non-TTY, no prompt is shown; merged-but-unclosed tickets remain in the blocker error and the command exits non-zero.
- [x] `apm epic close <id> --merge` merges the epic branch into the default branch locally and creates no PR.
- [x] `apm epic close <id> --auto` merges locally when the merge would be clean; falls back to opening a PR when it would conflict.
- [x] Without a flag (or with `--pr`), the command pushes the epic branch and opens or updates a PR — identical to the current behavior.
- [x] `--merge`, `--pr`, and `--auto` are mutually exclusive; clap rejects combinations with a usage error.
- [x] The already-merged shortcut (delete branch, skip PR/merge) is preserved regardless of which flag is given.

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
- Add `#[command(long_about = "...")]` with a multi-line string covering four stages: (1) quiescence check — all epic tickets must be non-terminal; merged-but-unclosed tickets are offered for auto-close on TTY; (2) already-merged shortcut — if the epic branch has no commits ahead of default, the branch is deleted locally and remotely, no PR created; (3) `--pr` (default) — pushes the epic branch and creates or updates a GitHub PR targeting the default branch; (4) `--merge` / `--auto` — local merge alternatives. Follow the `long_about` pattern at line 782 of `main.rs`.
- Add all four flags. `--close-all` comes from e96593f5; `--merge/--pr/--auto` are new here:
  ```rust
  #[arg(long)]
  close_all: bool,
  #[arg(long, conflicts_with_all = ["pr", "auto_mode"])]
  merge: bool,
  #[arg(long, conflicts_with_all = ["merge", "auto_mode"])]
  pr: bool,
  #[arg(long = "auto", conflicts_with_all = ["merge", "pr"])]
  auto_mode: bool,
  ```
- Update the dispatch arm:
  ```rust
  Command::Epic { command: EpicCommand::Close { id, close_all, merge, pr, auto_mode } }
      => cmd::epic::run_close(&root, &id, close_all, merge, pr, auto_mode),
  ```

#### `apm-core/src/epic.rs` — Replace `non_terminal_epic_tickets` with `classify_epic_quiescence`

e96593f5 adds `EpicTicketInfo { id, state, title }` and `non_terminal_epic_tickets()`. ab1eb252 replaces `non_terminal_epic_tickets` with `classify_epic_quiescence`, which applies the same `!terminal` filtering but classifies results into three buckets. Delete `non_terminal_epic_tickets` — its only caller is `run_close`, which now calls `classify_epic_quiescence` instead.

```rust
pub struct EpicQuiescenceResult {
    pub unsafe_tickets: Vec<EpicTicketInfo>,   // blocked/question — must resolve manually
    pub auto_closeable: Vec<EpicTicketInfo>,   // safe non-terminal, branch merged, no live worker
    pub genuine_blockers: Vec<EpicTicketInfo>, // safe non-terminal, branch not merged or has live worker
}

pub fn classify_epic_quiescence(
    root: &Path,
    epic_id: &str,
    config: &crate::config::Config,
    worktrees: &[(std::path::PathBuf, String)],
    epic_branch: &str,
) -> anyhow::Result<EpicQuiescenceResult>
```

For each non-terminal ticket in the epic (same `!terminal` predicate as `non_terminal_epic_tickets`), classify in this order:

1. State is `blocked` or `question` → `unsafe_tickets`.
2. Has a live `.apm-worker.pid` (check via the `worktrees` list) → `genuine_blockers`.
3. Ticket branch merged into `epic_branch` OR into `config.project.default_branch` via `git_util::is_branch_merged_into` → `auto_closeable`.
4. Otherwise → `genuine_blockers`.

`epic_is_quiescent` (used by `run_refresh_epic`) remains untouched.

Port the three unit tests that e96593f5 adds for `non_terminal_epic_tickets` to cover `classify_epic_quiescence` instead:
- `classify_epic_quiescence_all_closed_returns_empty` — all buckets empty when all tickets are terminal
- `classify_epic_quiescence_ignores_other_epics` — tickets belonging to another epic are ignored
- `classify_epic_quiescence_three_buckets` — blocked → unsafe; safe with merged branch → auto_closeable; safe with unmerged branch → genuine_blockers

#### `apm/src/cmd/epic.rs` — Updated `run_close`

New signature:
```rust
pub fn run_close(root: &Path, id_arg: &str, close_all: bool, merge: bool, pr: bool, auto_mode: bool) -> Result<()>
```

Replace all three quiescence-related code blocks — the original `epic_is_quiescent` bail, the earlier re-check from the first draft of this spec, and the `non_terminal_epic_tickets` guard added by e96593f5 — with a single unified section:

```rust
let result = apm_core::epic::classify_epic_quiescence(
    root, epic_id, &config, &worktrees, &epic_branch,
)?;

// Unsafe tickets always block — no flag overrides this.
if !result.unsafe_tickets.is_empty() {
    let rows = result.unsafe_tickets.iter()
        .map(|t| format!("  {:<8}  {:<13}  {}", t.id, t.state, t.title))
        .collect::<Vec<_>>().join("\n");
    anyhow::bail!(
        "cannot close epic: the following tickets require manual resolution:\n{}\nResolve them manually, then retry.",
        rows
    );
}

// Merged but not yet closed: offer auto-close.
// --close-all closes without prompt; on TTY ask interactively; on non-TTY treat as blocker.
let mut remaining: Vec<&apm_core::epic::EpicTicketInfo> =
    result.genuine_blockers.iter().collect();
if !result.auto_closeable.is_empty() {
    let should_close = close_all || (std::io::stdout().is_terminal() && {
        let n = result.auto_closeable.len();
        println!("\nTickets merged but not yet closed ({n}):");
        for t in &result.auto_closeable {
            println!("  {}  {}", t.id, t.title);
        }
        crate::util::prompt_yes_no(&format!("\nClose {n} merged ticket(s)? [y/N] "))?
    });
    if should_close {
        let actor = format!("{}(apm-epic-close)", apm_core::config::resolve_caller_name());
        for t in &result.auto_closeable {
            match apm_core::ticket::close(root, &config, &t.id, None, &actor, false) {
                Ok(msgs) => msgs.iter().for_each(|m| println!("{m}")),
                Err(e) => eprintln!("warning: could not close {}: {e:#}", t.id),
            }
        }
    } else {
        // User declined or non-TTY — treat declined tickets as blockers.
        remaining.extend(result.auto_closeable.iter());
    }
}

// Genuine blockers (unmerged tickets, live-worker tickets, any declined auto-closeables).
if !remaining.is_empty() {
    if !close_all {
        let rows = remaining.iter()
            .map(|t| format!("  {:<8}  {:<13}  {}", t.id, t.state, t.title))
            .collect::<Vec<_>>().join("\n");
        anyhow::bail!(
            "epic has {} non-terminal ticket(s):\n{}\nRe-run with --close-all to cascade close, or close them manually first.",
            remaining.len(), rows
        );
    }
    let actor = format!("{}(apm-epic-close)", apm_core::config::resolve_caller_name());
    for t in &remaining {
        print!("closing ticket #{} ... ", t.id);
        apm_core::ticket::close(root, &config, &t.id, None, &actor, false)
            .with_context(|| format!("failed to close ticket #{}", t.id))?;
        println!("done");
    }
}
```

When `close_all` is set, `should_close` is always `true` for `auto_closeable` tickets (closed without prompt) and `remaining` holds only `genuine_blockers`, which are then cascade-closed. This is e96593f5's `--close-all` semantics applied uniformly across all safe non-terminal tickets.

**Steps 5-6 — flag-conditional merge or PR** (unchanged from previous draft):

After the already-merged shortcut (unchanged), replace the existing push-and-PR block:

```rust
let do_merge = merge || (auto_mode && {
    let s = apm_core::epic::merge_tree_status(root, default_branch, &epic_branch)?;
    s.clean
});

if do_merge {
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
    apm_core::git::push_branch_tracking(root, &epic_branch)?;
    let mut messages = vec![];
    apm_core::github::gh_pr_create_or_update(
        root, &epic_branch, default_branch, epic_id, &pr_title,
        &format!("Epic: {epic_branch}"), &mut messages,
    )?;
    for m in &messages { println!("{m}"); }
}
```

When `--merge` is given and `merge_ref` encounters a conflict it returns `None` (already aborts the merge). When `--auto` is given and the merge would conflict, `do_merge` is `false`, so `merge_ref` is never called and the PR path runs.

#### Integration tests

Reuse the four integration tests e96593f5 adds to `apm/tests/integration.rs`, adjusting the single assertion that previously referenced `non_terminal_epic_tickets` to match the new three-bucket behavior:
- `epic_close_no_flag_bails_on_non_terminal_ticket` — ticket with unmerged branch → `genuine_blockers` → bails without `--close-all`
- `epic_close_all_bails_on_blocked_ticket` — blocked ticket → `unsafe_tickets` → bails even with `--close-all`
- `epic_close_all_bails_on_mixed_blocked_and_safe` — unsafe check fires before cascade attempt

Add from ab1eb252:
- `epic_close_auto_close_non_tty` — epic with one safe ticket whose branch is merged; run `run_close` with `stdout` not a TTY (default in test harness); assert `Err` and that the message lists the merged ticket as a blocker (auto-close was not offered)

### Open questions


### Amendment requests

- [x] Reconcile with sibling ticket e96593f5 (now a dependency). e96593f5 introduces a non-terminal-ticket guard in run_close that fires before any auto-close logic. Specify how ab1eb252's auto-sync flow fits on top: when e96593f5's check finds non-terminal tickets AND all of them are merged-and-eligible-for-close AND the supervisor is on a TTY, prompt for auto-close instead of failing. When non-eligible tickets exist (blocked, question, etc.), e96593f5's failure remains the outcome. Also refactor classify_epic_quiescence (the proposed new helper) to share logic with epic_is_quiescent rather than duplicating, so a future change to the blocker definition does not need to be made in two places.

### Code review


### Merge notes

merge conflict — resolve manually and push: 

## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T18:53Z | — | new | philippepascal |
| 2026-05-30T18:57Z | new | groomed | philippepascal |
| 2026-05-30T19:01Z | groomed | in_design | philippepascal |
| 2026-05-30T19:09Z | in_design | specd | claude |
| 2026-06-01T03:06Z | specd | ammend | philippepascal |
| 2026-06-01T07:06Z | ammend | in_design | philippepascal |
| 2026-06-01T07:14Z | in_design | specd | claude |
| 2026-06-01T07:36Z | specd | ready | philippepascal |
| 2026-06-01T08:21Z | ready | in_progress | philippepascal |
| 2026-06-01T08:31Z | in_progress | implemented | claude |
| 2026-06-01T08:31Z | implemented | merge_failed | claude |
