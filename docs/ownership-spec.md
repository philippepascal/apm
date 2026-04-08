# Ticket Ownership Spec

## Overview

Tickets have two identity fields: `author` and `owner`. Author is immutable (who created the ticket). Owner is who is responsible for managing the ticket through its lifecycle — typically a supervisor.

## Fields

| Field | Set by | Mutable | Purpose |
|-------|--------|---------|---------|
| `author` | `apm new` (from current user identity) | No | Record of who created the ticket |
| `owner` | `apm new` (defaults to author), `apm assign` | Yes (with restrictions) | Who manages/dispatches the ticket |

The `supervisor` field is removed. The `agent_name` concept (from `resolve_agent_name()`) remains for history/logging only — it is not an ownership concept.

## Rules

1. **Owner defaults to author** on ticket creation.
2. **Only the current owner can change the owner.** The system checks `current_user == ticket.owner` before allowing reassignment.
3. **Owner cannot be changed after a ticket is closed** (terminal state).
4. **Dispatchers only act on tickets owned by the current user.** `apm start --next`, `apm work`, and the UI dispatcher filter by `owner == current_user`. This means the supervisor runs dispatchers to spawn workers on their tickets.
5. **Workers never become owners.** State transitions do not change the owner field. A worker is assigned a specific ticket to implement; they don't own it.
6. **No special authorization in code.** The ownership check is the only enforcement. There is no role-based access control beyond "you must be the owner to reassign."

## Intended workflow

```
Supervisor creates ticket        → author=supervisor, owner=supervisor
Supervisor grooms it             → owner unchanged
Supervisor runs `apm work`       → dispatcher picks up owned tickets, spawns workers
Worker writes spec               → owner unchanged (still supervisor)
Supervisor reviews spec          → owner unchanged
Supervisor runs `apm work`       → dispatcher picks up owned ready tickets
Worker implements                → owner unchanged
Supervisor reviews and closes    → owner unchanged
```

To hand off a batch of tickets to another supervisor:
```
apm epic set <epic-id> owner <other-supervisor>
```

## Identity resolution (current user)

Two modes, determined by `git_host.provider` in config:

### Config-based mode (no git_host)
- Current user = `username` from `.apm/local.toml` (untracked, per-user)
- Valid usernames = `collaborators` list in `config.toml`
- Owner changes validated against the collaborators list

### GitHub mode (`git_host.provider = "github"`)
- Current user = GitHub username (via `gh api user` or token)
- Valid usernames = repo collaborators (fetched from GitHub API)
- Owner changes validated against GitHub collaborators

Future providers (GitLab, etc.) will follow the same pattern: identity from the provider, validation against provider's collaborator list.

## What to clean up

- Remove `supervisor` field from `Frontmatter` struct and all references
- Remove dead `resolve_collaborators()` function (replace with active validation)
- Ensure `resolve_agent_name()` is only used for history/logging, not confused with ownership
- Remove `agent` as a filterable/settable concept (it's a history concern, not an ownership one)
