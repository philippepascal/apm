+++
id = "2e832569"
title = "apm init re-run reports false diff for user-filled fields"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2e832569-apm-init-re-run-reports-false-diff-for-u"
created_at = "2026-04-24T06:28:08.866116Z"
updated_at = "2026-04-24T07:14:12.680669Z"
+++

## Spec

### Problem

When `apm init` is run on a project that already has `.apm/config.toml`, the re-run is supposed to detect whether the live config has drifted from the current default template. If it has drifted, `apm init` writes `.apm/config.toml.init` so the user can compare the two files and decide whether to adopt any new defaults.

The bug: `setup()` at `apm-core/src/init.rs:116` reconstructs the default config by extracting `project.name`, `project.description`, and `project.default_branch` from the live file — but hardcodes `collaborators = &[]`. Because `default_config()` serializes that as `collaborators = []`, the reconstructed default never matches the live file when the user has a non-empty collaborators list (e.g. `collaborators = ["philippepascal"]`). This causes a spurious `.apm/config.toml.init` to be produced on every re-run, even when the live config has never been touched by the user.

The affected users are anyone whose collaborators list was populated during initial interactive setup (i.e. when `apm init` ran with a detected Git username). Every subsequent re-run reports a false diff, which erodes trust in the `.init` signal.

### Acceptance criteria

- [ ] Re-running `apm init` on a project whose `.apm/config.toml` was created with a non-empty collaborators list (e.g. `collaborators = ["alice"]`) and has not been modified since does NOT produce `.apm/config.toml.init`
- [ ] Re-running `apm init` on a project whose `.apm/config.toml` has been manually edited (e.g. a `[custom]` section added) DOES produce `.apm/config.toml.init`
- [ ] The `.apm/config.toml.init` produced in the case above contains the same `collaborators` value as the live config (not an empty array)
- [ ] Re-running `apm init` on a project whose `.apm/config.toml` has `collaborators = []` does not produce `.apm/config.toml.init` when no other changes exist

### Out of scope

- Other user-editable fields (`logging.enabled`, `agents.max_concurrent`, etc.) are not normalized out of the diff; if the user changes them the `.init` signal fires correctly
- The interactive TTY path (`apm init` with a live terminal prompting for username) is not changed
- Normalizing diffs for `workflow.toml`, `ticket.toml`, `agents.md`, or other managed files
- Surfacing or formatting the diff to the user (that is a separate UX concern)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |