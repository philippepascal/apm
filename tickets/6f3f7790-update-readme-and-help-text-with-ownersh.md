+++
id = "6f3f7790"
title = "Update README and help text with ownership model"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "philippepascal"
branch = "ticket/6f3f7790-update-readme-and-help-text-with-ownersh"
created_at = "2026-04-08T15:32:38.451292Z"
updated_at = "2026-04-08T16:21:00.630482Z"
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

- [ ] README has a section explaining ticket ownership: author vs owner distinction, who can reassign (owner or supervisor), and dispatcher behavior (dispatchers pick only tickets they own)
- [ ] README documents `apm assign <id> <username>` and `apm assign <id> -` with a short example
- [ ] README documents `apm epic set <id> owner <user>` for bulk assignment
- [ ] README documents identity setup: config mode (`username` in `.apm/local.toml`) and GitHub mode (`[git_host] provider = "github"`)
- [ ] README happy path step 3 notes that the spec agent picks up only tickets assigned (owned) to it
- [ ] `apm assign --help` long description mentions that ownership gates dispatcher pickup
- [ ] `apm epic set --help` field description includes `owner` alongside `max_workers`
- [ ] `docs/commands.md` `apm epic set` section documents `owner` as a supported field with synopsis, description, and options table
- [ ] `docs/commands.md` `apm list` description paragraph explains the relationship between `--mine`, `--owner`, and dispatcher filtering

### Out of scope

- Documenting ownership-spec.md (the file does not exist; ownership semantics are captured in the specs of the individual implementation tickets)\n- API documentation for apm-server endpoints\n- Changing the ownership enforcement logic (this ticket is documentation only)

### Approach

This ticket is purely documentation — no logic changes. Three files change: `README.md`, `docs/commands.md`, and `apm/src/main.rs`. Apply changes after all other ownership tickets in epic `18dab82d` are merged into the target epic branch.

---

### README.md

**1. Add a new "## Ticket ownership" section** after "## Working with tickets" and before "## Agent workflow". Content:

```markdown
## Ticket ownership

Every ticket has two identity fields:

- **`author`** — set when the ticket is created; immutable. Records who created it.
- **`owner`** — who is responsible for the ticket. Dispatchers (`apm work`, `apm start --next`, the UI loop) only pick up tickets whose `owner` matches the current user's identity. Tickets with no owner are never auto-dispatched.

Assign a ticket before dispatching:

    apm assign <id> alice        # assign to alice
    apm assign <id> -            # clear the owner field

Bulk-assign all non-closed tickets in an epic at once:

    apm epic set <epic-id> owner alice

To filter the list by owner:

    apm list --owner alice       # tickets owned by alice
    apm list --mine              # tickets authored by the current user

### Identity setup

APM resolves the current user's identity in two modes:

**Config mode** (no `[git_host]` in `config.toml`): set `username` in `.apm/local.toml`:

    # .apm/local.toml
    username = "alice"

**GitHub mode** (`[git_host]` with `provider = "github"` in `config.toml`): identity is resolved from the `gh` CLI (if installed and authenticated) or from a GitHub token. No `local.toml` entry is needed — the GitHub login is used automatically.
```

**2. Update happy path step 3** from:

> **Spec agent picks it up** — the dispatch loop (`apm work`) assigns a worker.

to:

> **Spec agent picks it up** — but only if the ticket is assigned to it. The supervisor runs `apm assign a1b2 <agent-identity>` first. Then the dispatch loop (`apm work`) picks it up.

**3. Update "### Dispatching agents"** paragraph to note that dispatchers only pick owned tickets. Add a note under the dispatch examples:

> Dispatchers only pick up tickets whose `owner` matches the current user's identity. Assign tickets with `apm assign` before running `apm work`.

**4. Update the configuration table** — the `local.toml` row description (currently "Per-user settings (username, worker overrides) — untracked") is fine as-is; no change needed there.

---

### docs/commands.md

**1. `apm epic set` section** — extend the synopsis, description, and options table to cover `owner`:

- Add to synopsis:
  ```
  apm epic set <id> owner <username>
  apm epic set <id> owner -
  ```
- Add to description: "Set `owner` to bulk-assign ownership of all non-closed tickets in the epic to `<username>`. Pass `-` to clear the owner field on all non-closed tickets. The current user must be the owner of every ticket to be changed; if any check fails, no tickets are modified. Closed tickets are skipped."
- Update the options table: change `max_workers` row label to make clear it is one of multiple supported fields; add `owner` row.
- Add a "git internals" row for the ticket branch commits that `owner` triggers (same pattern as `apm assign` — `git add` + `git commit` per ticket branch, `git push` in aggressive mode).

**2. `apm list` section** — add a paragraph after the existing description paragraph explaining the relationship between `--mine`, `--owner`, and dispatch:

> `--mine` filters by `author` (the user who created the ticket). `--owner` filters by the `owner` field. Since dispatchers pick only tickets the current user owns, `apm list --owner <your-username>` shows the queue that `apm work` will draw from.

---

### apm/src/main.rs

**1. `Assign` command `long_about`** — extend the existing text to explain the dispatcher connection. Change:

```
Use this to assign a ticket to a user or agent, or to clear the owner field.
```

to:

```
Use this to assign a ticket to a user or agent, or to clear the owner field.

Ownership gates dispatcher pickup: `apm work`, `apm start --next`, and the UI
dispatch loop only pick up tickets whose owner matches the current user's identity.
Tickets with no owner are never auto-dispatched. Assign a ticket before running
the dispatch loop.
```

**2. `EpicCommand::Set` doc comments** — update the two doc comments that mention only `max_workers`:

- `/// Set a field on an epic (e.g. max_workers)` → `/// Set a field on an epic (max_workers or owner)`
- `/// Field to update (e.g. max_workers)` → `/// Field to update: max_workers or owner`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:32Z | — | new | philippepascal |
| 2026-04-08T15:34Z | new | groomed | apm |
| 2026-04-08T16:17Z | groomed | in_design | philippepascal |