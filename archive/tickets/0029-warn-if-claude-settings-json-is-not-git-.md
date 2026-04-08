+++
id = 29
title = "warn if .claude/settings.json is not git-tracked during init"
state = "closed"
priority = 4
effort = 2
risk = 1
author = "claude-0326-2222-8071"
agent = "claude-0327-1854-10aa"
branch = "ticket/0029-warn-if-claude-settings-json-is-not-git-"
created_at = "2026-03-27T05:57:10.499993Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

When `apm init` is run, `.claude/settings.json` may exist but not be tracked
by git. Future ticket branches created from `main` inherit whatever is committed;
if the settings file is untracked, agent worktrees won't have it, and permission
allow-lists won't apply. There is currently no feedback to the user about this.

### Acceptance criteria

- [ ] If `.claude/settings.json` exists but is not tracked by git (`git ls-files --error-unmatch` fails), `apm init` prints a warning suggesting the user commit it
- [ ] If the file is tracked, no warning is printed
- [ ] If the file does not exist, no warning is printed
- [ ] Warning is informational only — `apm init` still succeeds

### Out of scope

- Auto-staging or committing `.claude/settings.json`
- Warning about other `.claude/` files
- Checking whether the file contents are valid JSON

### Approach

In `cmd/init.rs` `run()`, after the existing setup steps, check:
1. Does `.claude/settings.json` exist?
2. Is it tracked? (`git ls-files --error-unmatch .claude/settings.json` exits 0)
3. If exists but untracked: `eprintln!("Warning: .claude/settings.json exists but is not committed. Agent worktrees won't have it — run: git add .claude/settings.json && git commit")`

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T05:57Z | — | new | claude-0326-2222-8071 |
| 2026-03-27T06:22Z | new | specd | claude-0326-2222-8071 |
| 2026-03-28T01:01Z | specd | ready | apm |
| 2026-03-28T01:56Z | ready | in_progress | claude-0327-1854-10aa |
| 2026-03-28T01:58Z | in_progress | implemented | claude-0327-1854-10aa |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |