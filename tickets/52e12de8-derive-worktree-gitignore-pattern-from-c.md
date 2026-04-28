+++
id = "52e12de8"
title = "Derive worktree gitignore pattern from config; validate enforces it"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/52e12de8-derive-worktree-gitignore-pattern-from-c"
created_at = "2026-04-28T19:54:13.505295Z"
updated_at = "2026-04-28T19:54:30.081282Z"
+++

## Spec

### Problem

`apm init`'s gitignore writer hardcodes `/worktrees/` regardless of the configured `worktrees.dir`, and `apm validate` doesn't check that the configured in-repo worktree dir is gitignored. Together these mean a user who customizes `worktrees.dir` ends up with worktree contents visible to git, with no detection at runtime.

**Concrete incident:** user changed `.apm/config.toml` from `dir = "../apm--worktrees"` (external) to `dir = ".apm--worktrees"` (in-repo, hidden). `.gitignore` was not updated. `apm validate` ran clean. The user only noticed when they opened `git status` and saw worktree contents staged for inclusion.

**Fix 1: `ensure_gitignore` must derive from config.**

Location: `apm-core/src/init.rs:194-217`, the `entries` array currently includes the literal `"/worktrees/"`.

Change to read `config.worktrees.dir` and emit the gitignore pattern from it:
- If the path is external (starts with `/` for absolute, or `..` for parent-traversal): skip — gitignore doesn't help here.
- Otherwise: emit `/<dir>/` (root-anchored, directory-only). For example `worktrees` → `/worktrees/`; `.apm--worktrees` → `/.apm--worktrees/`; `build/wt` → `/build/wt/`.
- The comment line `# apm worktrees` stays as-is.
- Idempotency check still applies — only append if the exact line is missing.

`ensure_gitignore` currently doesn't take `Config`; it takes `path: &Path`. Either pass the config in, or have `setup()` (the caller in `init.rs`) compute the pattern and pass it as a parameter.

**Fix 2: `apm validate` must check the gitignore.**

Location: `apm-core/src/validate.rs` and `apm/src/cmd/validate.rs`.

Add a check: when `config.worktrees.dir` is in-repo (not external), `.gitignore` must contain a pattern that matches it. Use a loose substring match against any of these forms (any one is acceptable):
- `/<dir>/`
- `/<dir>`
- `<dir>/`
- `<dir>`

Rationale for loose match: gitignore has multiple equivalent ways to ignore a directory; a strict literal-match would reject configs that are functionally correct.

Edge cases:
- `.gitignore` missing entirely → fail with a clear message; suggest re-running `apm init` or adding the line manually. `--fix` should append it (and the comment line) idempotently.
- External path (starts with `/` or `..`) → skip the check entirely; gitignore is irrelevant for paths outside the repo.
- The user's manually-added `.apm--worktrees` (no anchors) — passes the loose match.

This is the "(e)" check that was discussed when 38976b4b shipped but never filed. The hash-trip on config change (b10d957a) already runs `apm validate` on the next command after a config edit, so this check fires automatically when a user changes `worktrees.dir` — they get a clear validate failure pointing at the gitignore drift.

**Test pointers:**

- `init.rs`: `setup` writes `/<configured-dir>/` to `.gitignore`. Verify with custom `worktrees.dir` values: `worktrees`, `.apm--worktrees`, `build/wt`, `/abs/path`, `../external`. The last two should NOT add a worktree line.
- `validate.rs`: missing `.gitignore` for in-repo worktree dir → error. Pattern present in any of the four forms → ok. External worktree dir → no check fires regardless of gitignore content.
- Integration: edit `config.toml` to change `worktrees.dir` without updating `.gitignore`, run an apm command → hash-trip → validate fails with a pointer to the missing gitignore entry.

**Out of scope:**

- Already-tracked files inside the worktree dir (gitignore doesn't affect those — separate one-time migration concern).
- `.git/info/exclude` as an alternative ignore source (intentionally focus on `.gitignore` because it's committed and team-shareable).
- Renaming the worktree directory pattern across all places APM uses it (e.g. clean's filesystem walks).

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:54Z | — | new | philippepascal |
| 2026-04-28T19:54Z | new | groomed | philippepascal |
