+++
id = "4b663160"
title = "Implement apm epic new command"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "68666"
branch = "ticket/4b663160-implement-apm-epic-new-command"
created_at = "2026-04-01T21:55:06.350633Z"
updated_at = "2026-04-02T00:43:25.666912Z"
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

- [ ] `apm epic new "My Feature"` prints a branch name of the form `epic/<8-hex-id>-my-feature`
- [ ] The printed branch exists on `origin` after the command completes
- [ ] The epic branch is created from `origin/main` HEAD (not from the local `HEAD` or current branch)
- [ ] An `EPIC.md` file containing `# My Feature\n` is committed to the epic branch
- [ ] The epic branch tracks `origin/<branch>` (pushed with `--set-upstream`)
- [ ] `apm epic new` with no title argument exits non-zero and prints a usage error
- [ ] Running `apm epic new` when `origin` has no `main` branch exits non-zero with a clear error message
- [ ] `apm epic --help` prints the `new` subcommand in the usage output

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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |