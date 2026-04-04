+++
id = "aaa37e48"
title = "apm archive"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "apm"
branch = "ticket/aaa37e48-apm-archive"
created_at = "2026-04-03T00:33:18.924269Z"
updated_at = "2026-04-04T06:31:03.950795Z"
+++

## Spec

### Problem

As tickets are closed over time, the `tickets/` directory on `main` accumulates stale files indefinitely. While `apm list` hides terminal-state tickets by default, the files remain on disk and clutter the working directory for anyone browsing the repository. There is no automated way to sweep closed ticket files into a separate archive location.

This ticket adds `apm archive`, a command that moves closed ticket files from the active `tickets/` directory to a configurable archive directory on `main`. It also adds the `archive_dir` config key to `[tickets]` in `config.toml`, and extends the `apm show` fallback path so that archived tickets (whose per-ticket branch was later deleted by `apm clean --branches`) remain discoverable.

### Acceptance criteria

- [ ] `apm archive` errors with a clear message when `archive_dir` is not set in `[tickets]` config
- [ ] `apm archive` moves all terminal-state ticket files from `tickets/<id>-<slug>.md` to `<archive_dir>/<id>-<slug>.md` on the default branch in a single commit
- [ ] `apm archive --dry-run` prints the list of files that would be moved without modifying any branches
- [ ] `apm archive --older-than 30d` limits the batch to tickets whose `updated_at` is older than the threshold (same syntax as `apm clean --older-than`)
- [ ] `apm archive` skips ticket files that are not present in `tickets/` on the default branch and emits a per-ticket warning
- [ ] `apm archive` skips ticket files that are in a non-terminal state and emits a per-ticket warning
- [ ] `apm archive` prints a summary line: `archived N ticket(s)` (or `nothing to archive` when N = 0)
- [ ] `apm show <id>` succeeds for a ticket whose per-ticket branch has been deleted, when the ticket file exists in `archive_dir` on the default branch
- [ ] `[tickets] archive_dir = "archive/tickets"` in `config.toml` is accepted and loaded without error

### Out of scope

- Auto-archiving when a ticket transitions to a terminal state (i.e. no side effect on `apm state` or `apm close`) — that can be a follow-on ticket
- Restoring an archived ticket back to `tickets/` (no `apm unarchive`)
- Archiving epic branches or epic-related files
- Deleting the per-ticket git branch or worktree — that is `apm clean`'s job
- Remote branch pruning — handled by `apm clean --remote`
- Support for multiple archive directories or per-epic archive paths

### Approach

**1. Config — `apm-core/src/config.rs`**

Add `archive_dir: Option<PathBuf>` to `TicketsConfig`:

    pub struct TicketsConfig {
        pub dir: PathBuf,
        #[serde(default)]
        pub sections: Vec<String>,
        #[serde(default)]
        pub archive_dir: Option<PathBuf>,
    }

---

**2. Git primitive — `apm-core/src/git.rs`**

Add `pub fn move_files_on_branch(root, branch, moves: &[(&str, &str, &str)], message)` where each element is `(old_rel_path, new_rel_path, content)`:

- Use the same temp-worktree pattern as `commit_files_to_branch`
- For each move: write content to `new_rel_path`, stage it; run `git rm` for `old_rel_path`
- Commit all changes in a single call
- Reuse the permanent-worktree fast path when available (main normally does not have one, so the temp path will be taken)

---

**3. Archive module — `apm-core/src/archive.rs` (new file)**

    pub fn archive(
        root: &Path,
        config: &Config,
        dry_run: bool,
        older_than: Option<DateTime<Utc>>,
    ) -> Result<()>

Steps:
1. Bail if `config.tickets.archive_dir` is `None`
2. Read all ticket filenames from `config.tickets.dir` on the default branch via `git::list_files_on_branch`
3. Parse each file from the default branch; skip non-terminal-state tickets (warn per ticket)
4. If `older_than` set, skip tickets where `frontmatter.updated_at` is newer than the threshold
5. If `dry_run`: print each `tickets/<name>.md -> <archive_dir>/<name>.md` line and return
6. Build the moves list; call `git::move_files_on_branch` on the default branch
7. Print `archived N ticket(s)` (or `nothing to archive`)

---

**4. CLI — `apm/src/cmd/archive.rs` (new file) and `apm/src/main.rs`**

- Add `Archive` variant to the `Command` enum with `--dry-run` and `--older-than <THRESHOLD>`
- Delegate to `apm_core::archive::archive()`
- Parse `--older-than` using the existing `clean::parse_older_than` helper (re-export or inline)

---

**5. `apm show` fallback — `apm/src/cmd/show.rs`**

When a ticket is not found via `load_all_from_git` (no matching ticket branch), the show command falls back to searching `config.tickets.dir` on the default branch. Extend that fallback: if still not found and `config.tickets.archive_dir` is set, also check `archive_dir` on the default branch. No changes to `load_all_from_git`.

---

**6. Tests**

- Unit tests in `apm-core/src/archive.rs`: dry-run lists files, older-than filter excludes recent tickets, non-terminal ticket is skipped, missing `archive_dir` returns error
- Integration test in `apm/tests/integration.rs`: create temp repo, close a ticket, configure `archive_dir`, run `apm archive`, verify file moved on default branch; delete ticket branch, verify `apm show <id>` still succeeds via archive fallback

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-03T00:33Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:25Z | groomed | in_design | philippepascal |