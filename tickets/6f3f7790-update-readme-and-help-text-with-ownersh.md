+++
id = "6f3f7790"
title = "Update README and help text with ownership model"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/6f3f7790-update-readme-and-help-text-with-ownersh"
created_at = "2026-04-08T15:32:38.451292Z"
updated_at = "2026-04-08T16:17:05.508730Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["751f65f6", "b52fc7f4"]
+++

## Spec

### Problem

The README and CLI help text do not document the ownership model that is being introduced across the `18dab82d` epic. Specifically:

- There is no explanation of the **author vs owner distinction**: `author` is set at creation and is immutable; `owner` is the responsible party and determines who dispatchers will pick work for.
- The **dispatcher filtering rule** — that `apm work`, `apm start --next`, and the UI dispatch loop only pick up tickets whose `owner` matches the current user's identity — is undocumented. Users who create tickets without assigning owners will be confused why nothing gets dispatched.
- `apm assign` exists but its help text does not explain the dispatcher connection.
- `apm epic set <id> owner <user>` (added by ticket b52fc7f4) is entirely undocumented.
- The **two identity modes** are not explained: config mode (no `[git_host]`, set `username` in `.apm/local.toml`) and GitHub mode (`[git_host] provider = "github"`, identity resolved from the `gh` CLI or GitHub token). Without this, users cannot understand why `--mine` or dispatcher ownership checks use the wrong name.
- `apm list --mine` and `apm list --owner` are listed in the options table but not explained in context.
- The happy path walkthrough does not mention that the spec agent picks only tickets assigned to it.

The desired state is that a user reading the README understands the full ownership workflow end-to-end, and that `apm assign --help` and `apm epic set --help` accurately describe the effect on dispatch.

### Acceptance criteria

- [ ] README has a section explaining ticket ownership (author vs owner, who can reassign, dispatcher behavior)
- [ ] README documents `apm assign` and `apm epic set <id> owner`
- [ ] README documents identity setup (local.toml username for config mode, git_host for GitHub mode)
- [ ] `apm assign --help` text is clear and accurate
- [ ] `docs/commands.md` updated with ownership-related commands
- [ ] Happy path walkthrough reflects ownership (supervisor creates and owns, dispatches to workers)

### Out of scope

Documenting the ownership spec itself (already in docs/ownership-spec.md). API documentation for apm-server.

### Approach

Update README.md, docs/commands.md, and CLI help strings (clap descriptions in apm/src/main.rs). Use docs/ownership-spec.md as the source of truth. This ticket should be done last after all other ownership tickets are implemented and merged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:32Z | — | new | philippepascal |
| 2026-04-08T15:34Z | new | groomed | apm |
| 2026-04-08T16:17Z | groomed | in_design | philippepascal |