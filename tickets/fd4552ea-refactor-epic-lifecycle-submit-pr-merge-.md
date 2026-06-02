+++
id = "fd4552ea"
title = "Refactor epic lifecycle: submit (PR/merge) vs close (cleanup); sync surfaces hints"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fd4552ea-refactor-epic-lifecycle-submit-pr-merge-"
created_at = "2026-06-02T06:05:04.230173Z"
updated_at = "2026-06-02T06:17:44.647585Z"
+++

## Spec

### Problem

GOAL: split today's apm epic close into two distinct, single-purpose commands and add passive detection to apm sync so the supervisor's mental model matches the ticket lifecycle.

CURRENT PROBLEM: apm epic close does two completely different things depending on the epic's state:
- If not merged into main: pushes the branch and opens a PR (or merges, with --merge / --auto flags)
- If already merged: deletes the local branch and skips the PR

The supervisor has to run the same command twice (once to open the PR, once to clean up after the merge). This is inconsistent with how tickets work — for tickets, the supervisor runs apm state implemented to open a PR, then apm sync (passive) detects the merge and offers to close the ticket. For epics, there is no passive detection; the supervisor has to remember to re-run apm epic close.

Two observations sharpened the design:
- In the syn project, an empty epic (72294403) was squash-merged on GitHub. apm epic close on a second invocation failed with No commits between main and epic (gh rejected the PR creation); apm clean --epics said Nothing to clean. The supervisor had to hand-delete the branch and worktree. The naive git log --oneline main..branch check that both apm clean and apm epic close used misses squash merges; sync already has the right detection but its scope did not cover epic branches.
- Conceptually, the two phases of today's apm epic close are different actions with different verbs. Submitting an epic for merge is a creation action (pushes commits, opens a PR). Closing the epic is a cleanup action (deletes the branch, removes the worktree). Calling them both close conflates them.

NEW MODEL:

apm epic submit <id> [--pr | --merge | --auto]
- Single phase. Pushes the epic branch, opens or updates a PR (or merges, with --merge).
- Idempotent. Running on an already-submitted epic updates the existing PR (gh_pr_create_or_update already does this).
- --pr (default): push and open PR
- --merge: do a working-tree merge of the epic into main, push main
- --auto: merge when clean, fall back to PR when the merge would conflict
- Does NOT delete the branch. Does NOT delete the worktree. Submit is about getting the work into main, nothing else.
- If --merge fails (conflict), the command fails loudly and suggests --pr as the next step.

apm epic close <id> [--force]
- Single phase. Deletes the local epic branch and removes the epic's worktree (if one exists). Optionally pushes the branch deletion to origin (git push origin --delete) — spec-writer to confirm whether this is default-on or behind a flag.
- Safety: if the epic branch has commits not present in main (regular ancestor check OR squash-merge check via the shared helper, both via origin/main preferred), refuse the close and print: epic has N commit(s) not yet in main. Use --force to confirm deletion (commits will be lost).
- --force: skip the safety check. Delete unconditionally. The supervisor's escape hatch for abandoning unsubmitted work.
- Close is irreversible (the branch is gone). The supervisor must intend it.

apm sync (additions)
- After the existing ticket-merge pass, add a second pass that scans local epic branches and prints up to two hint sections:
  1. Epics ready to submit: epic's derived state is done (all tickets terminal) but the branch is not yet merged into origin/main. Output: 'Epics ready to submit (apm epic submit <id>):' then a list.
  2. Epics ready to close: epic branch is merged into origin/main (use the shared squash-aware helper). Output: 'Epics ready to close (apm epic close <id>):' then a list.
- These are HINTS only. sync prints them and exits. It does not prompt to act on them. (Submit and close are real git mutations the supervisor must intend.)
- The detection uses the shared squash-merge helper (see below).

apm clean --epics is REMOVED. The --epics flag is dropped from apm clean. Bulk-clean was a stopgap; sync's hints + intentional apm epic close replace it. apm clean continues to handle ticket worktree cleanup as today.

NEW SHARED HELPER: apm-core/src/git_util.rs

Extract sync's existing squash-merge detection into a pub fn is_branch_content_merged(root, default_branch, branch) -> Result<bool>. Algorithm mirrors squash_merged today: compute merge_base, compare branch_tip; if equal return true; otherwise synthesize a virtual squash commit via git commit-tree branch carat braces tree -p merge_base, then git cherry default_branch virtual_squash and check for a leading dash. Prefer origin slash default_branch over local when the remote ref exists (mirrors merged_into_main's preference).

CONSUMERS:
- sync uses the helper in both the ticket-merge pass (already does, via squash_merged) and the new epic-detection pass.
- apm epic close uses the helper for its unmerged-work safety check.
- apm epic submit does not need the helper (it always pushes; gh handles the no-commits case).

BEHAVIORAL BREAK / MIGRATION:

This renames a public command (apm epic close changes meaning) and removes a flag (apm clean --epics). External scripts that invoke either will break. The fix is to update help text and the README to clearly describe the new model. Anyone who scripted today's apm epic close to mean push-then-clean must split it into apm epic submit (first run) + apm epic close (after merge).

OUT OF SCOPE:
- merge_failed-equivalent state for epics (today there is no epic state machine; the merge-conflict path is just an error message). If we add an epic state machine later, that is a separate concern.
- Adding tickets to an epic after submission. This already works naturally — adding a ticket pushes more commits to the epic branch, which gh auto-updates onto the open PR. No code change needed; just document.
- apm-server / apm-ui changes beyond surface-area renaming if any UI references the old close-name.
- Replacing the bulk-close path. If a future need for apm epic close --all-merged emerges, file then. For now: no bulk option.
- 0e55807c is superseded by this ticket. The squash-merge helper extraction is part of this scope; the worktree-cleanup behavior is part of apm epic close's new definition.
- dc2b08db (apm move worktree side-effect) is unrelated and unaffected.

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- apm epic submit on an epic with no PR pushes the branch and opens a PR. Output names the PR URL.
- apm epic submit on an epic with an open PR updates the PR (no new PR created). Output names the existing PR URL.
- apm epic submit --merge on an epic that would merge cleanly does the merge and pushes main. apm epic submit --merge on an epic that would conflict fails with a clear message suggesting --pr.
- apm epic submit --auto behaves like --merge when clean, --pr when conflicted.
- apm epic close on an epic whose branch is merged into origin/main (regular or squash) deletes the branch and removes the worktree.
- apm epic close on an epic with unmerged commits ahead of origin/main refuses, prints the ahead-count, and suggests --force.
- apm epic close --force deletes the branch and removes the worktree unconditionally.
- apm sync prints an Epics ready to submit section listing epics whose state is done and branch is not yet merged.
- apm sync prints an Epics ready to close section listing epics whose branch is merged into origin/main.
- apm sync does not prompt for any epic action; it only prints hints.
- apm clean has no --epics flag. apm clean --epics fails with an error suggesting apm epic close.
- A new public function apm_core::git_util::is_branch_content_merged exists and is used by sync and apm epic close.
- Unit tests for is_branch_content_merged: regular merge returns true; squash merge returns true; unmerged branch returns false; missing remote ref falls back to local.
- Integration test: end-to-end submit-then-close-after-merge flow for an empty epic and a populated epic.
- Integration test: sync hints appear after a PR merge and disappear after apm epic close.
- Help text for apm epic clearly documents submit vs close as two separate phases.
- README and any docs that reference today's apm epic close are updated.

REFERENCES:
- apm/src/cmd/epic.rs (run_close, run_epic_clean) — existing logic to refactor
- apm-core/src/git_util.rs::squash_merged (around line 217) — algorithm to extract
- apm-core/src/git_util.rs::merged_into_main (around line 102) — origin-preference pattern to mirror
- apm/src/cmd/clean.rs — remove the --epics branch
- apm/src/cmd/sync.rs and apm-core/src/sync.rs — add the epic-detection pass
- apm-core/src/worktree.rs — existing worktree-cleanup logic for apm epic close to reuse
- Background: syn project epic 72294403 hit squash-merge invisibility; design discussion in conversation history
- Supersedes: 0e55807c (which covered the helper extraction and worktree-cleanup parts of this work)

### Acceptance criteria

- [ ] `apm epic submit <id>` on an epic with no existing PR pushes the branch to origin and opens a PR; output includes the PR URL.
- [ ] `apm epic submit <id>` on an epic with an existing open PR updates the PR without creating a new one; output includes the existing PR URL.
- [ ] `apm epic submit --merge <id>` on an epic that merges cleanly into the default branch merges it locally and pushes; no PR is created.
- [ ] `apm epic submit --merge <id>` on an epic that would conflict exits non-zero with a message naming the conflict and suggesting `--pr`.
- [ ] `apm epic submit --auto <id>` merges cleanly when no conflict would occur; falls back to opening a PR when a conflict is detected.
- [ ] `apm epic close <id>` on an epic whose branch is fully merged into `origin/<default>` (regular or squash merge) deletes the local branch, removes the worktree if present, and exits zero.
- [ ] `apm epic close <id>` on an epic with commits not yet in `origin/<default>` exits non-zero, prints the number of unmerged commits, and suggests `apm epic close --force`.
- [ ] `apm epic close --force <id>` deletes the local branch and removes the worktree unconditionally regardless of merge status.
- [ ] `apm sync` output includes an "Epics ready to submit" section listing epics whose derived state is `done` and whose branch is not yet merged into `origin/<default>`.
- [ ] `apm sync` output includes an "Epics ready to close" section listing epics whose branch is merged into `origin/<default>` (squash-aware detection).
- [ ] `apm sync` prints the epic hint sections without prompting; no action is taken automatically.
- [ ] `apm clean --epics` exits non-zero with a message directing the user to `apm epic close <id>`.
- [ ] A public function `apm_core::git::is_branch_content_merged(root, default_branch, branch)` exists, prefers `origin/<default>` when the remote ref is present, and falls back to the local ref when it is not.
- [ ] `is_branch_content_merged` returns `true` for a branch merged into default via a regular (fast-forward or no-ff) merge.
- [ ] `is_branch_content_merged` returns `true` for a branch squash-merged into default.
- [ ] `is_branch_content_merged` returns `false` for a branch with commits not present in default.
- [ ] Integration test covers the end-to-end flow: `apm epic submit --merge` merges the branch; `apm epic close` subsequently deletes the branch and worktree.
- [ ] Integration test verifies that `sync::detect` populates `epic_close_hints` after a squash merge of an epic branch, and `epic_submit_hints` when the epic is done but not yet merged.
- [ ] `apm epic --help` shows `submit` and `close` as distinct subcommands with non-overlapping descriptions.

### Out of scope

- Epic state machine (no `merge_failed`-equivalent state; a conflict in `--merge` is an error, not a state transition)
- The quiescence check currently in `run_close` — dropped entirely; `submit` and `close` no longer gate on ticket states
- Adding tickets to an epic after submission — already works via normal git commits to the epic branch; no code change needed
- apm-server and apm-ui changes beyond any surface-area rename forced by the `submit`/`close` split
- Bulk close (`apm epic close --all-merged` or similar)
- Ticket 0e55807c — superseded; its squash-merge helper extraction and worktree-cleanup scope are absorbed here
- Ticket dc2b08db (apm move worktree side-effect) — unrelated and unaffected

### Approach

#### Files changed

| File | Change |
|------|--------|
| `apm-core/src/git_util.rs` | Add `pub fn is_branch_content_merged` |
| `apm-core/src/sync.rs` | Add epic-detection pass; extend `Candidates` with epic hint vecs |
| `apm/src/cmd/epic.rs` | Add `run_submit`; replace `run_close` body; delete `run_epic_clean` |
| `apm/src/main.rs` | Add `Submit` to `EpicCommand`; rework `Close` flags; hide `--epics` on `Clean` |
| `apm/src/cmd/clean.rs` | Error on `--epics` |
| `apm/src/cmd/sync.rs` | Print epic hint sections after ticket handling |

#### Step 1 — `is_branch_content_merged` (apm-core/src/git_util.rs)

`is_branch_merged_into` at line 755 already implements the squash-merge algorithm. The new public function is a thin wrapper that adds origin-preference, matching the pattern in `merged_into_main`:

```rust
pub fn is_branch_content_merged(root: &Path, default_branch: &str, branch: &str) -> Result<bool> {
    let remote_ref = format!("refs/remotes/origin/{default_branch}");
    let main_ref = if run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
        format!("origin/{default_branch}")
    } else {
        default_branch.to_string()
    };
    is_branch_merged_into(root, branch, &main_ref)
}
```

`git_util` is re-exported as `git` in `lib.rs`, so callers use `git::is_branch_content_merged` within `apm-core` and `apm_core::git::is_branch_content_merged` from the CLI.

Add inline unit tests (temp git repo, no fixture files):
- regular (no-ff) merge → `true`
- squash merge (commit-tree + reset) → `true`
- branch with unpushed commits → `false`
- origin ref absent → falls back to local ref; result still correct

#### Step 2 — `run_submit` (apm/src/cmd/epic.rs)

New function `run_submit(root, id_arg, merge, pr, auto_mode)` extracted from the current `run_close` push/PR/merge block (steps 5–7 in current code). Key changes from the existing logic:

- No quiescence check (no ticket-state gating)
- No "already merged" shortcut (that belongs to `close`)
- `do_merge = merge || (auto_mode && merge_tree_status(root, default_branch, &epic_branch)?.clean)`
- If `do_merge`: call `git_util::merge_ref` in the main worktree. On conflict: bail `"merge conflict — use --pr to open a PR instead"`
- If not `do_merge`: `push_branch_tracking` → `gh_pr_create_or_update`

The `--auto` path determines cleanness before choosing merge vs PR. When auto falls back to PR due to conflict, print a note explaining why.

#### Step 3 — `run_close` replacement (apm/src/cmd/epic.rs)

Replace the current body entirely. New signature: `run_close(root, id_arg, force)` — removes `close_all`, `merge`, `pr`, `auto_mode`.

Steps:
1. Resolve epic branch (same prefix-match)
2. Determine main ref (origin preferred): same two-line pattern as Step 1
3. Call `git::is_branch_content_merged(root, default_branch, &epic_branch)?`
4. If not merged and not `--force`:
   - Count ahead commits: `git rev-list --count <main_ref>..<epic_branch>`
   - Bail: `"epic has N commit(s) not yet in <default_branch>. Use --force to delete unconditionally."`
5. Remove worktree if present: `find_worktree_for_branch` → `remove_worktree(root, &path, true)`
6. Delete local branch: `git branch -D <epic_branch>` (force-delete; the old `-d` would fail here on an unmerged branch with `--force`)
7. Delete remote branch: `git push origin --delete <epic_branch>` (suppress "remote ref does not exist")
8. Print: `"deleted epic/<id>"` on success

Delete `run_epic_clean` — no longer called anywhere after Step 5.

#### Step 4 — Epic detection in `apm-core/src/sync.rs`

Extend `Candidates`:

```rust
pub struct Candidates {
    pub close: Vec<CloseCandidate>,
    pub hints: Vec<String>,
    pub epic_submit_hints: Vec<(String, String)>,  // (id, title)
    pub epic_close_hints:  Vec<(String, String)>,
}
```

At the end of `detect`, after the existing ticket passes, add an epic pass:

```
let epic_branches = crate::epic::epic_branches(root).unwrap_or_default();
if !epic_branches.is_empty() {
    let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)
        .unwrap_or_default();
    for branch in &epic_branches {
        let id = crate::epic::epic_id_from_branch(branch);
        let title = crate::epic::branch_to_title(branch);
        let epic_tickets: Vec<_> = all_tickets.iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
            .collect();
        let state_cfgs: Vec<&StateConfig> = epic_tickets.iter()
            .filter_map(|t| config.workflow.states.iter()
                .find(|s| s.id == t.frontmatter.state))
            .collect();
        let derived = crate::epic::derive_epic_state(&state_cfgs);
        let is_merged = git::is_branch_content_merged(root, default_branch, branch)
            .unwrap_or(false);
        if is_merged {
            epic_close_hints.push((id.to_string(), title));
        } else if derived == "done" {
            epic_submit_hints.push((id.to_string(), title));
        }
    }
}
```

`epic_branches` returns local `epic/*` branches only. The origin-preference is already inside `is_branch_content_merged`.

#### Step 5 — Print epic hints in `apm/src/cmd/sync.rs`

After the existing ticket hint/close block (around line 160), add:

```rust
if !candidates.epic_submit_hints.is_empty() {
    println!("\nEpics ready to submit (apm epic submit <id>):");
    for (id, title) in &candidates.epic_submit_hints {
        println!("  {id:<8}  {title}");
    }
}
if !candidates.epic_close_hints.is_empty() {
    println!("\nEpics ready to close (apm epic close <id>):");
    for (id, title) in &candidates.epic_close_hints {
        println!("  {id:<8}  {title}");
    }
}
```

No prompt. These are informational only.

#### Step 6 — CLI wiring in `apm/src/main.rs`

Add `Submit` to `EpicCommand`:

```rust
Submit {
    id: String,
    #[arg(long, conflicts_with_all = ["merge", "auto_mode"])]
    pr: bool,
    #[arg(long, conflicts_with_all = ["pr", "auto_mode"])]
    merge: bool,
    #[arg(long = "auto", conflicts_with_all = ["merge", "pr"])]
    auto_mode: bool,
},
```

Update `Close` variant: remove `close_all`, `merge`, `pr`, `auto_mode`; add `#[arg(long)] force: bool`.

For `--epics` removal on `Clean`: keep the field as `#[arg(long, hide = true)]` so clap still accepts it (avoids a confusing "unexpected argument" error), then in `cmd::clean::run`, if `epics` is true, bail with: `"apm clean --epics has been removed; use 'apm epic close <id>' instead"`.

Add dispatch arms for `Submit` → `cmd::epic::run_submit` and updated `Close` → `cmd::epic::run_close`.

#### Step 7 — Help text

- `EpicCommand::Close` long_about: "Delete the local epic branch and remove its worktree. Safe by default: refuses when the branch has commits not yet in the default branch. Use --force to delete unconditionally."
- `EpicCommand::Submit` long_about: "Push the epic branch to origin and open or update a GitHub PR (default), or merge it locally into the default branch (--merge). Use --auto to merge when clean and fall back to PR on conflict."
- Update any README section that describes `apm epic close` as the PR-opening command.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T06:05Z | — | new | philippepascal |
| 2026-06-02T06:07Z | new | groomed | philippepascal |
| 2026-06-02T06:11Z | groomed | in_design | philippepascal |