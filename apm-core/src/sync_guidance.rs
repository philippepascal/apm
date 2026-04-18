// Single source of guidance wording for `apm sync`.
//
// All copy-pasteable recovery messages that sync emits live here.
// Never scatter literal guidance strings through the sync flow —
// always reference a named constant from this module so future
// wording changes are a single-point edit.
//
// Placeholders used inside string bodies:
//   <default>  — the project's default branch name (e.g. "main")
//   <id>       — ticket short id
//   <slug>     — branch slug (e.g. "ticket/abc123-my-feature")
//   <count>    — number of commits (numeric string, caller supplies)
//   <commits>  — the word "commit" or "commits" (caller supplies)
//
// Callers substitute via `.replace("<default>", branch_name)` etc.
// at the print site; this module stays purely declarative.

/// Printed when local `<default>` is behind `origin/<default>` (fast-forward
/// possible in principle) but `git merge --ff-only` refused because uncommitted
/// local changes would be overwritten by the update.
pub const MAIN_BEHIND_DIRTY_OVERLAP: &str = "\
apm sync: cannot fast-forward <default> — uncommitted local changes overlap with incoming commits.

Resolve by committing or stashing your changes first, then re-run apm sync:

    git stash
    apm sync
    git stash pop

Or, if you want to discard your local changes:

    git checkout -- .
    apm sync";

/// Printed when local `<default>` and `origin/<default>` have diverged
/// (each side has commits the other lacks) and the working tree is clean.
pub const MAIN_DIVERGED_CLEAN: &str = "\
apm sync: <default> has diverged from origin/<default> — cannot fast-forward.

Your local <default> has commits not on origin, and origin has commits not on local.
Resolve by rebasing or merging manually, then push:

    git fetch origin
    git rebase origin/<default>     # or: git merge origin/<default>
    git push origin <default>

After resolving, re-run apm sync.";

/// Printed when local `<default>` and `origin/<default>` have diverged
/// (each side has commits the other lacks) and the working tree is dirty.
pub const MAIN_DIVERGED_DIRTY: &str = "\
apm sync: <default> has diverged from origin/<default> and your working tree has uncommitted changes.

Stash your changes first, then resolve the divergence manually:

    git stash
    git fetch origin
    git rebase origin/<default>     # or: git merge origin/<default>
    git push origin <default>
    git stash pop

After resolving, re-run apm sync.";

/// Printed when local `<default>` has commits not yet pushed to `origin/<default>`.
/// Sync never pushes; the user must push explicitly.
/// Placeholders: `<default>`, `<remote>`, `<count>`, `<commits>`.
pub const MAIN_AHEAD: &str = "\
<default> is ahead of <remote> by <count> <commits>. Merged tickets will not be detected as closeable until you push — run `git push` when ready.";

/// Printed when a non-checked-out `ticket/*` or `epic/*` ref has local commits
/// not yet pushed to `origin`.  Sync never pushes; the user must push explicitly.
/// Placeholder: `<slug>`.
pub const TICKET_OR_EPIC_AHEAD: &str = "\
info: <slug> is ahead of origin — push when ready: git push origin <slug>";

/// Printed for a non-checked-out `ticket/*` or `epic/*` ref whose local tip
/// and `origin` tip have diverged (local has unpushed commits AND origin has
/// commits not on local).  Sync cannot safely update either side.
pub const TICKET_OR_EPIC_DIVERGED: &str = "\
apm sync: branch <slug> has diverged from origin/<slug> — skipping automatic update.

To resolve, check out the branch and merge or rebase manually:

    git checkout <slug>
    git fetch origin
    git rebase origin/<slug>        # or: git merge origin/<slug>
    git push origin <slug>

After resolving, re-run apm sync.";

/// Printed when `apm sync` detects the repo is mid-merge, mid-rebase, or
/// mid-cherry-pick (`.git/MERGE_HEAD`, `.git/rebase-merge`, `.git/rebase-apply`,
/// or `.git/CHERRY_PICK_HEAD` exists).  Any sync work done in this state would
/// compound the incomplete operation.
pub const MID_MERGE_IN_PROGRESS: &str = "\
apm sync: repository is mid-merge, mid-rebase, or mid-cherry-pick — cannot sync now.

Finish or abort the in-progress operation first, then re-run apm sync.

To abort a merge:
    git merge --abort

To abort a rebase:
    git rebase --abort

To abort a cherry-pick:
    git cherry-pick --abort";
