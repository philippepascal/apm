+++
id = "dd412cd3"
title = "Implement apm epic close command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "27501"
branch = "ticket/dd412cd3-implement-apm-epic-close-command"
created_at = "2026-04-01T21:55:18.313179Z"
updated_at = "2026-04-02T00:47:54.397879Z"
+++

## Spec

### Problem

There is no command to create a PR from an epic branch to `main`. When an engineering team finishes all tickets in an epic, the epic branch must be merged to `main` as a coherent unit. Currently this requires running `gh pr create` manually, knowing the exact branch name and base branch.

`apm epic close <id>` should automate this: look up the epic branch by its short ID, verify that every ticket in the epic is in `implemented` or a later state, then run `gh pr create --base main --head epic/<id>-<slug>` and print the PR URL. The command does not merge — merging is left to human reviewers on GitHub.

Without this command the epic workflow is incomplete: tickets can be created (`apm new --epic`), listed (`apm epic list/show`), but never promoted to a PR as a group.

### Acceptance criteria

- [ ] `apm epic close <id>` exits 0 and prints a GitHub PR URL when the epic branch exists and all tickets in the epic are in `implemented` or a later (terminal) state
- [ ] `apm epic close <id>` exits non-zero with a clear error message listing the non-ready tickets when one or more epic tickets are not yet `implemented`
- [ ] `apm epic close <id>` exits non-zero with a clear error message when no epic branch matching the given ID prefix is found
- [ ] `apm epic close <id>` exits 0 and prints "PR #N already open" (without creating a duplicate) when an open PR already exists for the epic branch
- [ ] `apm epic close <id>` accepts a 4–8 character prefix of the epic ID (same prefix-matching behaviour used by other apm commands)
- [ ] The created PR targets `config.project.default_branch` (not a hardcoded "main")

### Out of scope

- Merging the PR — that requires human approval on GitHub; this command only opens it
- Creating the epic branch (`apm epic new`) — separate command, not covered here
- Listing or showing epics (`apm epic list`, `apm epic show`) — separate commands
- Adding the `epic` frontmatter field to tickets or the `--epic` flag on `apm new` — separate work
- The `depends_on` scheduling feature described in `docs/epics.md`
- apm-server / apm-ui API routes for epics
- Any UI changes

### Approach

### New subcommand plumbing

Add `Epic { ... }` subcommand to `Command` enum in `apm/src/main.rs` with its own `Subcommand` enum:

```
Epic {
    #[command(subcommand)]
    action: EpicCommand,
}
```

`EpicCommand` enum has a single variant for now: `Close { id: String }`. Wire the dispatch in `main` to `cmd::epic::run_close`.

### New file: `apm/src/cmd/epic.rs`

`pub fn run_close(root: &Path, id_arg: &str) -> Result<()>` does:

1. Load config — `Config::load(root)`
2. Find the epic branch — run `git branch -r` filtered to `epic/` branches. Collect all branches whose 8-char ID segment starts with `id_arg`. Error if 0 or >1 matches.
3. Load all tickets — `ticket::load_all_from_git(root, &config.tickets.dir)`
4. Identify epic tickets — filter to tickets where `frontmatter.epic` matches the full 8-char epic ID parsed from the branch name. If no tickets carry this epic ID, the gate check passes vacuously.
5. Gate check — for each epic ticket, check whether its state is terminal (config: `state.terminal == true`) or equals `"implemented"`. Collect all non-passing tickets. If any exist, print them and bail.
6. PR idempotency — run `gh pr list --head <epic-branch> --state open --json number --jq '.[0].number'`; if a PR number comes back, print "PR #N already open for <branch>" and return Ok.
7. Create PR — `gh pr create --base <default_branch> --head <epic-branch> --title "<human title>" --body "Epic: <branch>"`. Print the URL on success.

### Frontmatter field `epic`

Add `pub epic: Option<String>` to `apm_core::ticket::Frontmatter` with `#[serde(skip_serializing_if = "Option::is_none")]`. This is the only struct change needed. No migration required — existing tickets without the field deserialise to `None`.

### PR title derivation

Strip `epic/` prefix and the `<8-char-id>-` segment from the branch name, replace remaining hyphens with spaces, title-case each word. Example: `epic/ab12cd34-user-authentication` becomes `"User Authentication"`.

### Tests

Add unit tests in `apm/src/cmd/epic.rs` (or extracted to `apm-core`) covering:
- Branch-to-title conversion
- ID prefix resolution (0 matches → error, 1 match → ok, >1 match → error)
- Gate check logic: a slice with all-implemented tickets passes; a slice with one non-implemented ticket fails

Integration tests that require a live `gh` CLI and GitHub remote are out of scope — skip for now.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T21:59Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:47Z | groomed | in_design | philippepascal |