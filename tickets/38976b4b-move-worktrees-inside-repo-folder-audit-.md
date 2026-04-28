+++
id = "38976b4b"
title = "Move worktrees inside repo folder; audit apm clean for safety"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/38976b4b-move-worktrees-inside-repo-folder-audit-"
created_at = "2026-04-28T01:24:55.011587Z"
updated_at = "2026-04-28T15:29:26.854975Z"
+++

## Spec

### Problem

**Motivation**

The current default (`worktrees.dir = "../apm--worktrees"` in `.apm/config.toml`) places worktrees as a sibling of the repo. This forces every consumer of the project — Claude Code, containers, CI runners, sandboxed sessions — to know about and grant permission to a second filesystem location.

Concrete pain in this project:
- `CLAUDE.md` lists `/Users/philippepascal/repos/apm--worktrees` as an Additional working directory purely because of the external layout.
- User memory `feedback_subagent_worktree_bash.md`: subagents with `isolation:"worktree"` do not pick up `.claude/settings.json`; Bash is denied and Edit/Write are blocked for any path under `apm--worktrees/`. The workaround today is to have the main conversation handle file writes on the agent's behalf — a real cost.
- Containers/CI must mount or grant access to two paths instead of one.

Moving worktrees to a directory **inside** the repo (e.g. default `worktrees.dir = "worktrees"`) collapses these to one permission scope.

**What this ticket should change**

1. Change the `apm init` default for `worktrees.dir` from `../apm--worktrees` to `worktrees` (or another in-repo subdir).
2. Have `apm init` write/update `.gitignore` to include the worktrees dir line (e.g. `/worktrees/`). Idempotent — append only if the line is not present.
3. Audit `apm clean` for safety with the new layout (see below — this is the load-bearing concern).
4. Audit any other code path that walks the filesystem from the repo root and may descend into the worktrees subdir (e.g., `apm verify`, `apm sync` cache walks if any).
5. Leave existing repos with `../apm--worktrees` untouched. Migration is opt-in: a repo already using the external layout keeps working. Optionally provide `apm init --migrate-worktrees` later, but that is out of scope here.

**The `apm clean` case (detailed)**

Today `apm clean` removes worktrees and branches for closed tickets. With external worktrees, the worktrees dir lives outside the repo and there is no risk of clean traversing into its own siblings. With internal worktrees the topology changes, and these become real concerns:

(a) **Source of truth must be `git worktree list`, not filesystem walking.** If clean enumerates candidates by walking `<repo>/worktrees/` and matching directory names, it can:
- pick up partially-deleted worktrees (race with concurrent removal)
- pick up directories that are not registered worktrees at all (manual debris)
- miss worktrees whose paths were renamed via `git worktree move`

The fix: enumerate via `git worktree list --porcelain` (which reads from `.git/worktrees/`) and remove via `git worktree remove <path>`. Git's machinery already knows the registered set and refuses to remove a worktree with uncommitted changes unless `--force` is passed. Use that.

(b) **Clean must refuse to remove the worktree the caller is inside.** If a worker invokes `apm clean` from its own ticket worktree (currently improbable, but possible) and that ticket happens to be marked closed concurrently, clean would try to remove the directory the process is running from. This is bad on every layout but particularly easy to trigger when worktrees live under a path the caller might `cd` into casually. Compute `std::env::current_dir()` and `apm_core::worktree::main_worktree_root()` at the start of clean; if `cwd` is inside any candidate worktree path, refuse with a clear message: `refusing to remove worktree containing the current working directory: <path>`.

(c) **Clean must never `rm -rf` on a candidate path.** Use `git worktree remove` exclusively. If a worktree is "prunable" (registered but on-disk path missing), use `git worktree prune` to clean the registry — never delete files outside what git decides.

(d) **Clean's existing `--branches`, `--remote`, `--older-than`, `--untracked` flags** (commit `7ab4d84c`) should be re-checked against the new layout. `--untracked` in particular: with the worktrees dir gitignored, untracked files inside a worktree are still untracked from that worktree's own perspective; the flag's existing semantics should hold, but write a test that confirms it.

**Other concerns to address in the spec**

- `.gitignore` line: `/worktrees/` (leading slash so it does not match nested files of that name elsewhere) plus a one-line comment naming why. Idempotent.
- IDE indexers and editors will see N checkouts of the source under the repo root. Modern indexers respect `.gitignore`; older ones do not. Document this in the migration note. Out of scope to fix non-compliant tooling.
- Cargo `target/` dirs end up per-worktree (`worktrees/<id>/target/`). Disk usage grows. Acceptable; reuse via `CARGO_TARGET_DIR` is out of scope.
- Tools doing filesystem walks from the repo root that **don't** respect gitignore (any `std::fs::read_dir` walks in APM itself) must explicitly skip the configured worktrees dir. Search for any such walk in `apm-core/` and `apm/` and gate on `config.worktrees.dir`.

**Implementation pointers**

- `apm-core/src/init.rs` — change the `worktrees.dir` default in the generated config; update `ensure_worktrees_dir` if needed; ensure `.gitignore` is updated.
- `apm-core/src/worktree.rs` — `provision_worktree` already uses `main_worktree_root()` (per fix `5a36f7db` Apr 12) so path computation is correct; just verify with a unit test that the new in-repo layout works.
- `apm/src/cmd/clean.rs` — implement (a)/(b)/(c) above; add tests that drive `apm clean` from inside a ticket worktree and assert refusal.
- `apm/src/cmd/verify.rs` — verify it does not descend into the worktrees dir while doing its checks.

**Out of scope**

- Migrating existing repos automatically (would require coordinating with running workers and rewriting `.git/worktrees/*/gitdir` files; defer).
- Sharing a single `target/` across worktrees via `CARGO_TARGET_DIR`.
- Changing the worktrees dir name to something other than `worktrees` (project preference; default is fine).

### Acceptance criteria

- [x] `apm init` on a fresh repo writes `dir = "worktrees"` (not `"../{name}--worktrees"`) to `.apm/config.toml`\n- [x] After `apm init`, the directory `<repo>/worktrees/` exists\n- [x] After `apm init`, `.gitignore` contains the line `/worktrees/`\n- [x] Running `apm init` a second time does not duplicate the `/worktrees/` line in `.gitignore`\n- [x] An existing repo whose `.apm/config.toml` already has `dir = "../apm--worktrees"` continues to provision and clean worktrees at that external path without error\n- [x] `apm start` for a new ticket provisions the worktree at `<repo>/worktrees/<ticket-branch>/`, inside the repo root\n- [x] `apm clean` invoked from inside a ticket's worktree (cwd is a path under `<repo>/worktrees/<branch>/`) refuses with a message containing "refusing to remove worktree containing the current working directory" and exits non-zero\n- [x] `apm clean` enumerates candidate worktrees from `git worktree list --porcelain` only; no filesystem walk of `<repo>/worktrees/`\n- [x] `apm clean` removes worktrees via `git worktree remove`, never via `rm -rf`\n- [x] `apm clean` cleans dangling registry entries (registered path no longer on disk) via `git worktree prune`, not by deleting files\n- [x] `apm clean --untracked` does not walk into or affect `<repo>/worktrees/` from the main worktree's perspective\n- [x] `apm verify` produces no output related to files under `<repo>/worktrees/`

### Out of scope

- Automatic migration of existing repos from `../apm--worktrees` to `worktrees/` (coordinating with running workers and rewriting `.git/worktrees/*/gitdir` pointers is deferred)\n- `apm init --migrate-worktrees` command\n- Renaming the in-repo worktrees dir to something other than `worktrees`\n- Sharing a single `CARGO_TARGET_DIR` across worktrees to reduce disk usage\n- Fixing IDE indexers that do not respect `.gitignore` (document the caveat; out of scope to fix non-compliant tooling)\n- Any changes to the `apm worktree move` / `git worktree move` flows\n- Changing the path computation in `provision_worktree` beyond what the new default config value already causes

### Approach

**1. `apm-core/src/init.rs` — default config and `.gitignore`**\n\nIn `default_config()` (lines ~276-277), change the `[worktrees]` template line from `dir = "../{name}--worktrees"` to `dir = "worktrees"`.\n\nIn `ensure_gitignore()` (lines ~194-217), add two entries to the `entries` array: a comment line `"# apm worktrees"` and the pattern `"/worktrees/"`. The function already appends only missing entries (idempotent). Treat the comment line the same way: check for its presence before appending. Leading slash scopes the ignore to the repo root.\n\n`ensure_worktrees_dir()` requires no changes; it already resolves `main_worktree_root() + config.worktrees.dir`.\n\n**2. `apm-core/src/worktree.rs` — unit test**\n\n`provision_worktree()` already uses `main_worktree_root() + config.worktrees.dir`; the new layout is automatically correct. Add one unit test: given a temp git repo and a config with `worktrees.dir = "worktrees"`, assert the computed worktree path is `<repo>/worktrees/<branch>` (inside the repo root, not a sibling).\n\n**3. `apm-core/src/clean.rs` + `apm/src/cmd/clean.rs` — safety audit and guards**\n\n(a) Source-of-truth audit: `candidates()` already delegates to `list_ticket_worktrees()` which reads `git worktree list --porcelain`. Confirm `find_worktree_for_branch()` has no filesystem-walk fallback; remove any such fallback if found.\n\n(b) CWD guard — add to the top of `cmd::clean::run()`:\n\n    let cwd = std::env::current_dir().unwrap_or_default();\n\nThen before each call to `clean::remove()`, check `cwd.starts_with(candidate_path)`. On match, print "refusing to remove worktree containing the current working directory: <path>" to stderr and exit non-zero. Use canonicalised paths if symlinks are a concern (`Path::starts_with` does component-wise matching).\n\n(c) No rm-rf / prunable entries: `remove_worktree()` already calls `git worktree remove [--force] <path>`. Add an explicit branch: if the candidate path does not exist on disk, call `git worktree prune` instead of `git worktree remove`. This keeps the registry clean without touching files. Never delete files directly.\n\n(d) `--untracked` flag: existing semantics (running `git status --porcelain` inside each worktree) are unaffected by the layout change. Add an integration test: provision a worktree in-repo, place a debris file directly under `<repo>/worktrees/` (outside any registered worktree), run `apm clean --untracked` from the main cwd, assert the debris file is untouched.\n\n**4. `apm/src/cmd/verify.rs` — no changes needed**\n\nAudit confirmed: `verify_tickets()` validates ticket state from config; `merged_into_main()` queries git refs. Neither descends into the filesystem worktrees dir. Close the audit item with a comment in the PR.\n\n**5. Filesystem walk audit — no changes needed**\n\nThe only `std::fs::read_dir` in `apm-core/` is `copy_dir_recursive()` (worktree.rs line 130), called by `sync_agent_dirs()` with an explicit named source (e.g. `.claude`) and an explicit destination inside a newly-provisioned worktree. It cannot descend from the repo root. No code change needed.\n\n**Tests to add (all in new or existing `#[cfg(test)]` blocks):**\n- `init.rs`: fresh init → `config.toml` has `dir = "worktrees"`; `.gitignore` contains `/worktrees/`; second init → no duplicate line\n- `worktree.rs`: path computation with `worktrees.dir = "worktrees"` → result is under repo root\n- `clean.rs` integration: cwd inside a ticket worktree when `apm clean` runs → non-zero exit with refusal message\n- `clean.rs` integration: `apm clean --untracked` from main cwd → `<repo>/worktrees/` debris file untouched\n- `clean.rs` unit: prunable entry (registered path missing from disk) → `git worktree prune` called, no file deletion attempted\n\n**Order of changes:** init.rs default → init.rs gitignore → worktree.rs test → clean.rs CWD guard → clean.rs prunable handling → clean.rs tests → verify.rs audit comment

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T01:24Z | — | new | philippepascal |
| 2026-04-28T07:13Z | new | groomed | philippepascal |
| 2026-04-28T07:22Z | groomed | in_design | philippepascal |
| 2026-04-28T07:27Z | in_design | specd | claude-0428-0722-a850 |
| 2026-04-28T15:13Z | specd | ready | philippepascal |
| 2026-04-28T15:29Z | ready | in_progress | philippepascal |
