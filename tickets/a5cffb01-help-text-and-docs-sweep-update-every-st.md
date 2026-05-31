+++
id = "a5cffb01"
title = "Help text and docs sweep: update every stale reference to old schema"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a5cffb01-help-text-and-docs-sweep-update-every-st"
created_at = "2026-05-31T02:59:02.592158Z"
updated_at = "2026-05-31T07:46:25.185357Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463", "9c66e199", "4d20ba2f"]
+++

## Spec

### Problem

STEP 9 of the incremental workflow schema cleanup. Doc audit and updates after all schema changes have landed.

KNOWN STALE LOCATIONS to audit and fix:

1. apm/src/main.rs and apm/src/cmd/*.rs — every clap long_about and short about:

   - apm prompt long_about: describes the OLD layer order ('Layer 1 — APM system knowledge ... Layer 3 — Role instructions'). After ticket 9ea43165 the layers were reversed; help text was not updated. Also shows the OLD --explain example with 'skipped:' and 'level 0 / level 2 — not reached' lines (cleaned up by 48d3932b and 9ea43165). Also references 'shell discipline' as part of Layer 1 content (moved to role file by a3c34ddc). Rewrite the long_about to reflect the current shape.

   - apm instructions short about: currently 'Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference'. The 'shell discipline,' clause is stale (a3c34ddc).

   - apm validate, apm new, apm state, apm next, apm list, and any other command whose help touches workflow concepts that have changed (actionable, worker_profile, role).

2. apm help <topic> output. Topics include commands, config, workflow, ticket. The workflow topic in particular needs a rewrite covering: state-level worker_profile, trigger uniqueness rule, no actionable field, no transition.worker_profile, mandatory workers.default.

3. apm/src/cmd/show.rs print_ticket — verify nothing references dropped fields (actionable). Today it does not seem to but verify.

4. apm-core/src/init.rs — if the init scaffold writes any prose about the workflow, audit and update. Especially the [workers] section in the scaffolded config.toml (4d20ba2f makes default mandatory).

5. README files: top-level README.md, apm-core/README.md, apm-server/README.md, apm-ui/README.md. Search for: 'transition.worker_profile', 'transition.role', 'actionable', 'role: worker', 'agent/supervisor', 'derive_transition_role'. Update.

6. docs/ directory if present.

7. apm-core/src/default/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md (and their .apm/ project copies). If any reference 'actionable' or describe transitions removed in 071886fc, update.

8. apm-core/src/default/workflow.toml comments — if the file has comments explaining the schema, they may reference the old field names.

9. Code comments in apm-core/src that explain workflow concepts. Grep for old terminology.

APPROACH:
- ripgrep for 'transition.worker_profile', 'transition.role', 'actionable', 'derive_transition_role', 'role_command_allowlist', 'Actionable by'
- Read each hit, decide if it is stale, update with new vocabulary
- Build the project; run apm <subcommand> --help for each command to spot-check rendered text
- Run cargo test --workspace

TESTS:
- 'apm instructions' short summary does not contain 'shell discipline'
- 'apm prompt --help' long_about does not contain old layer labels (where Layer 1 = APM system knowledge) and does not contain the 'skipped:' / 'level N — not reached' format
- A grep for 'transition.worker_profile' across apm-core/src/, apm/src/, apm-server/src/ returns nothing (excluding tests / archive / tickets if scanned)
- A grep for 'actionable' returns nothing in source code (excluding test fixtures that explicitly test the absence and historical tickets)
- A grep for 'derive_transition_role' returns nothing in source code

OUT OF SCOPE:
- Functional code changes (handled by earlier tickets in the epic).
- apm-server / apm-ui surfaces (next ticket).
- Updating historical ticket markdown files in tickets/ or archive/ (those are records, not docs).

REFERENCES:
- apm/src/main.rs
- apm/src/cmd/*
- apm-core/src/init.rs
- README files
- apm-core/src/default/agents/claude/*.md

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
| 2026-05-31T02:59Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:46Z | groomed | in_design | philippepascal |
