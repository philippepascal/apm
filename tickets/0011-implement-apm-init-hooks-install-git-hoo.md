+++
id = 11
title = "apm init should install git hooks unconditionally"
state = "specd"
priority = 10
effort = 2
risk = 1
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

`apm init` sets up the ticket directory and config but does not install git hooks.
Hooks are not optional — `pre-push` fires the `event:branch_push_first` auto-transition
and `post-merge` triggers a sync. Every checkout needs them. Requiring a separate
`--hooks` flag creates setup friction and guarantees hooks are forgotten.

### Acceptance criteria

- [ ] `apm init` writes `.git/hooks/pre-push` (executable) calling `apm _hook pre-push "$@"`
- [ ] `apm init` writes `.git/hooks/post-merge` (executable) calling `apm sync --quiet --offline`
- [ ] Both hooks include a guard: no-op gracefully if `apm` is not on PATH
- [ ] Running `apm init` a second time overwrites hooks (idempotent)
- [ ] Hook files are marked executable (`chmod +x`)

### Out of scope

- `apm _hook pre-push` implementation — tracked in #8
- `apm sync --quiet --offline` implementation — tracked in #4
- Adding hooks to `.claude/settings.json` — tracked in #12

### Approach

In `apm/src/cmd/init.rs`, add a `write_hooks(git_dir: &Path) -> Result<()>` function
called unconditionally from `run()`. It writes both hook files and sets executable
permissions using `std::fs::set_permissions` with mode `0o755`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
