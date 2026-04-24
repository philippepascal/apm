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

Re-running apm init generates .apm/config.toml.init that differs from the live .apm/config.toml even when the user has not edited the live config. Example: "collaborators = [\"philippepascal\"]" in live vs "collaborators = []" in .init. Root cause: default_config() at apm-core/src/init.rs:244 is re-invoked to produce .init content, but effective_username is empty on non-TTY re-runs (line ~91-97 passes empty collaborators when no username). The test at apm-core/src/init.rs:687-704 shows when .init is created. Fix: fields that are filled during interactive setup (project.name, project.description, project.collaborators) should be normalized out of the diff, either by writing .init with the live values for those fields, diffing only structural keys, or treating them as user-owned in .init.

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
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |
