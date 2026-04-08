+++
id = "bbd5d271"
title = "Validate owner against collaborators in config-based mode"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/bbd5d271-validate-owner-against-collaborators-in-"
created_at = "2026-04-08T15:09:59.601187Z"
updated_at = "2026-04-08T16:02:44.680486Z"
epic = "18dab82d"
target_branch = "epic/18dab82d-ticket-ownership-model"
depends_on = ["b0708201"]
+++

## Spec

### Problem

In config-based mode (no git_host provider), there is no validation when changing a ticket's owner. A typo in a username goes undetected. The `project.collaborators` list exists in config.toml but is never checked at runtime. Owner changes should validate the new owner against this list.

### Acceptance criteria

- [ ] When `git_host.provider` is not set, `apm assign` validates the new owner against `project.collaborators`
- [ ] If the new owner is not in the collaborators list, command fails with "unknown user '<name>'; valid collaborators: <list>"
- [ ] If collaborators list is empty, validation is skipped (no restriction)
- [ ] `apm set <id> owner <user>` has the same validation
- [ ] Tests cover: valid collaborator accepted, unknown user rejected, empty collaborators list skips validation

### Out of scope

GitHub-based validation (separate ticket c738d9cc). Adding users to the collaborators list (manual config edit).

### Approach

Add a `validate_owner(config: &Config, username: &str) -> Result<()>` function in `apm-core`. In config-based mode, check `config.project.collaborators`. If the list is non-empty and the username is not in it, return an error. Wire this into the ownership check helper from ticket b0708201. See `docs/ownership-spec.md`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T15:09Z | — | new | philippepascal |
| 2026-04-08T15:33Z | new | groomed | apm |
| 2026-04-08T16:02Z | groomed | in_design | philippepascal |
