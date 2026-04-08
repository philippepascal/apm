+++
id = "4b663160"
title = "Implement apm epic new command"
state = "closed"
priority = 8
effort = 3
risk = 2
author = "claude-0401-2145-a8f3"
agent = "33478"
branch = "ticket/4b663160-implement-apm-epic-new-command"
created_at = "2026-04-01T21:55:06.350633Z"
updated_at = "2026-04-02T03:12:28.705793Z"
+++

## Spec

### Problem

There is currently no way to create an epic. An epic is a git branch (`epic/<id>-<slug>`) — no separate file format needed. Without a command to create one, the entire epics workflow cannot be started.

The full design is in `docs/epics.md` (§ Commands — `apm epic new`). The command must:
1. Generate an 8-hex-char short ID
2. Slugify the title
3. Fetch `origin/main`, create `epic/<id>-<slug>` from its HEAD
4. Optionally commit an `EPIC.md` file (title as H1) to establish the branch as diverged from main
5. Push with `-u origin`
6. Print the branch name

The `apm epic` subcommand group does not yet exist and must be wired into the CLI.

### Acceptance criteria

- [x] `apm epic new "My Feature"` prints a branch name of the form `epic/<8-hex-id>-my-feature`
- [x] The printed branch exists on `origin` after the command completes
- [x] The epic branch is created from `origin/main` HEAD (not from the local `HEAD` or current branch)
- [x] An `EPIC.md` file containing `# My Feature\n` is committed to the epic branch
- [x] The epic branch tracks `origin/<branch>` (pushed with `--set-upstream`)
- [x] `apm epic new` with no title argument exits non-zero and prints a usage error
- [x] Running `apm epic new` when `origin` has no `main` branch exits non-zero with a clear error message
- [x] `apm epic --help` prints the `new` subcommand in the usage output

### Out of scope

- `apm epic list` — listing epics (separate future ticket)
- `apm epic show <id>` — showing epic details (separate future ticket)
- `apm epic close <id>` — opening a PR to merge the epic (separate future ticket)
- `apm new --epic <id>` — creating tickets inside an epic (separate future ticket)
- `epic`, `target_branch`, and `depends_on` fields in ticket frontmatter
- `depends_on` scheduling in the work engine
- `apm work --epic` exclusive-mode filtering
- apm-server API routes for epics (`GET/POST /api/epics`)
- apm-ui changes (epic column, filter dropdown, engine controls)

### Approach

**Files to change**

`apm-core/src/lib.rs` — add `pub mod epic;`

`apm-core/src/epic.rs` (new file) — `pub fn create(root: &Path, title: &str) -> Result<String>`:
1. `id = git::gen_hex_id()`
2. `slug = ticket::slugify(title)`
3. `branch = format!("epic/{id}-{slug}")`
4. Fetch origin/main: inline `git fetch origin main` Command, propagate error on failure
5. Build a unique temp path using PID + subsec_nanos + branch slug (same pattern as `try_worktree_commit` in git.rs)
6. `git worktree add -b <branch> <tmp_path> origin/main`
7. Write `"# {title}\n"` to `<tmp_path>/EPIC.md`
8. `git -C tmp add EPIC.md`
9. `git -C tmp commit -m "epic({id}): create {title}"`
10. `git worktree remove --force <tmp_path>` + `fs::remove_dir_all` (best-effort cleanup)
11. `git::push_branch_tracking(root, &branch)` — see below
12. Return `Ok(branch)`

`apm-core/src/git.rs` — add `push_branch_tracking(root: &Path, branch: &str) -> Result<()>`:
mirrors `push_branch` but passes `--set-upstream` before `origin` in the git push args.

`apm/src/cmd/epic.rs` (new file) — thin handler: calls `apm_core::epic::create(root, &title)`, prints the returned branch name.

`apm/src/lib.rs` — add `pub mod epic;` inside the `pub mod cmd` block.

`apm/src/main.rs` — add `EpicCommand` enum with a `New { title: String }` variant; add `Epic { #[command(subcommand)] command: EpicCommand }` variant to `Command`; route it to `cmd::epic::run_new(&root, title)`.

**Implementation order**

1. `git.rs` — add `push_branch_tracking`
2. `apm-core/src/epic.rs` + update `lib.rs`
3. `apm/src/cmd/epic.rs` + update `lib.rs`
4. `apm/src/main.rs` — wire CLI

**Constraints and gotchas**

- The temp worktree lives outside the repo (in `std::env::temp_dir()`); git worktrees can be anywhere, this is fine.
- After `git worktree remove --force`, the local branch ref still exists; the push uses that ref.
- `git::gen_hex_id()` and `ticket::slugify()` are already `pub` — no visibility changes needed.
- `EPIC.md` commit is always created (not optional); it establishes branch divergence from main.
- No config changes required.

### Files to change

**`apm-core/src/lib.rs`** — add `pub mod epic;`

**`apm-core/src/epic.rs`** (new file) — `pub fn create(root: &Path, title: &str) -> Result<String>`:
1. `id = git::gen_hex_id()`
2. `slug = ticket::slugify(title)`
3. `branch = format!("epic/{id}-{slug}")`
4. Fetch origin/main: `git fetch origin main` (inline Command, propagate error)
5. Build a unique temp path using PID + subsec_nanos + branch slug (same pattern as `try_worktree_commit`)
6. `git worktree add -b <branch> <tmp_path> origin/main`
7. Write `format!("# {title}\n")` to `<tmp_path>/EPIC.md`
8. `git -C tmp add EPIC.md`
9. `git -C tmp commit -m "epic({id}): create {title}"`
10. `git worktree remove --force <tmp_path>` + `fs::remove_dir_all` (best-effort cleanup)
11. `git::push_branch_tracking(root, &branch)` — push with `--set-upstream`
12. Return `Ok(branch)`

**`apm-core/src/git.rs`** — add `push_branch_tracking(root, branch)`:
```
git push --set-upstream origin <branch>:<branch>
```
Mirrors `push_branch` but adds `--set-upstream` before `origin`.

**`apm/src/cmd/epic.rs`** (new file):
```
pub fn run_new(root: &Path, title: String) -> Result<()> {
    let branch = apm_core::epic::create(root, &title)?;
    println!("{branch}");
    Ok(())
}
```

**`apm/src/lib.rs`** — add `pub mod epic;` inside the `pub mod cmd` block

**`apm/src/main.rs`** — three additions:
1. New `EpicCommand` enum above or below `Command`:
   ```
   #[derive(Subcommand)]
   enum EpicCommand {
       /// Create a new epic branch
       New { title: String }
   }
   ```
2. New `Epic` variant in `Command`:
   ```
   /// Manage epics
   Epic {
       #[command(subcommand)]
       command: EpicCommand,
   }
   ```
3. Match arm in `main()`:
   ```
   Command::Epic { command: EpicCommand::New { title } } => cmd::epic::run_new(&root, title),
   ```

### Order of steps

Implement in this order to keep each step buildable:
1. `apm-core/src/git.rs` — add `push_branch_tracking`
2. `apm-core/src/epic.rs` + update `lib.rs`
3. `apm/src/cmd/epic.rs` + update `lib.rs`
4. `apm/src/main.rs` — wire CLI

### Constraints and gotchas

- `std::env::temp_dir()` returns `/tmp` on macOS/Linux; the temp worktree is outside the repo — this is fine, git worktrees can be anywhere
- After `git worktree remove --force`, the local branch ref for the epic still exists; the push in step 11 uses that ref
- `git::gen_hex_id()` and `ticket::slugify()` are already `pub` — no visibility changes needed
- `EPIC.md` commit is not optional; it is what makes the branch visible in `git log --oneline main..epic/...`
- No config changes required; the command works with any existing `.apm/` setup

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:48Z | in_design | specd | claude-0401-2209-spec1 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T02:52Z | ready | in_progress | philippepascal |
| 2026-04-02T02:55Z | in_progress | implemented | claude-0402-0300-b7e2 |
| 2026-04-02T03:12Z | implemented | closed | apm-sync |