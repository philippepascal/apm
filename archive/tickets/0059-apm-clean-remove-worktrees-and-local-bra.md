+++
id = 59
title = "apm clean: remove worktrees and local branches for closed tickets"
state = "closed"
priority = 3
effort = 2
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1430-main"
branch = "ticket/0059-apm-clean-remove-worktrees-and-local-bra"
created_at = "2026-03-29T19:12:18.328861Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

After tickets are closed and their PRs merged, the permanent git worktrees and local branch refs created by `apm start` / `apm worktrees --add` are never cleaned up. Over time this clutters `git worktree list` and `git branch --list ticket/*` with stale entries. There is no command to remove them in bulk.

### Acceptance criteria

- [x] `apm clean` iterates all tickets in terminal states (per `workflow.terminal_states` in `apm.toml`) and, for each: removes the permanent worktree if one exists, and deletes the local branch ref.
- [x] `--dry-run` prints what would be removed without modifying anything.
- [x] A ticket whose branch is not merged into the default branch is skipped with a warning (safety guard against premature cleanup).
- [x] A worktree with uncommitted changes (dirty index or working tree) is skipped with a warning and its local branch is left intact.
- [x] Each removed worktree or branch ref produces one line of output: `removed worktree <path>` or `removed branch <name>`.
- [x] If there is nothing to clean, prints `Nothing to clean.` and exits 0.

### Out of scope

- Deleting remote branches on origin.
- Cleaning worktrees for non-ticket branches.
- Cleaning tickets in non-terminal states.
- Any interactive confirmation prompt (safety is handled by the merge check and uncommitted-changes check).

### Approach

Add `apm/src/cmd/clean.rs` with a single `pub fn run(root: &Path, dry_run: bool) -> Result<()>`.

Steps inside `run`:

1. Load config (`Config::load`) to get `workflow.terminal_states` and the default branch (`repos.code[0].default_branch`).
2. Load all tickets (`ticket::load_all_from_git`).
3. Get the set of branches merged into the default branch (`git::merged_into_main`).
4. For each ticket whose state is in `terminal_states`:
   a. Derive the branch name from `frontmatter.branch` or `git::branch_name_from_path`.
   b. Skip if the branch is not in the merged set, printing `warning: <branch> not merged — skipping`.
   c. Check for a permanent worktree via `git::find_worktree_for_branch`.
   d. If a worktree exists: run `git -C <wt_path> status --porcelain`; if output is non-empty, print `warning: <path> has uncommitted changes — skipping` and continue to the next ticket.
   e. If `--dry-run`: print `would remove worktree <path>` and `would remove branch <branch>`; otherwise call `git::remove_worktree` then delete the local ref with `git branch -d <branch>` via `Command`.
   f. Print one confirmation line per action taken (`removed worktree <path>`, `removed branch <branch>`).
5. Wire up in `main.rs`: add a `Clean { #[arg(long)] dry_run: bool }` variant to the `Command` enum and dispatch to `cmd::clean::run`.

The local branch deletion uses `git branch -d` (safe delete — refuses if unmerged); the merged-into-main pre-check above means this should always succeed. If `git branch -d` fails, surface the error and continue to the next ticket rather than aborting the whole run.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:56Z | new | in_design | claude-spec-59 |
| 2026-03-29T22:58Z | in_design | specd | claude-spec-59 |
| 2026-03-29T23:16Z | specd | ready | apm |
| 2026-03-29T23:36Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:39Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:48Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |