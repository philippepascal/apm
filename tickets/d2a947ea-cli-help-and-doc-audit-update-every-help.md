+++
id = "d2a947ea"
title = "CLI help and doc audit: update every help string referencing the old workflow schema"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d2a947ea-cli-help-and-doc-audit-update-every-help"
created_at = "2026-05-31T01:59:53.144925Z"
updated_at = "2026-05-31T03:03:58.338796Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["1e758cd5"]
+++

## Spec

### Problem

After 1e758cd5 lands the new schema, audit every CLI help string, descriptive long_about, README, and in-code documentation that references the old shape (transition.role, transition.worker_profile, state.actionable_by, the 'agent / supervisor' actionable_by enum, 'role: worker' as a placeholder, etc.). Update them.

KNOWN STALE LOCATIONS TO AUDIT:

1. apm/src/main.rs and apm/src/cmd/*.rs — every #[command(long_about = ...)] string. Specifically:
   - apm prompt long_about: currently describes the OLD layer order (Layer 1 = APM system knowledge, Layer 3 = Role instructions). After 9ea43165 already reversed this, but the help text was not updated. The --explain example also shows the old verbose format with 'skipped:' and 'level 0 / level 2 — not reached' lines. Update fully.
   - apm prompt help mentions 'shell discipline' as part of Layer 1 content. This is stale since a3c34ddc moved shell discipline into role files. Remove.
   - apm instructions long_about / one-line about: currently says 'Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference'. The 'shell discipline,' clause is stale.
   - apm validate, apm new, apm state, and any other command whose help mentions workflow concepts that have changed.

2. apm-core/src/instructions.rs — any in-output prose that describes the workflow or roles. The State Machine and Command Reference sections are dynamic; verify the surrounding text is current.

3. apm help <topic> output. Topics include commands, config, workflow, ticket. The workflow topic in particular needs a rewrite to reflect the new state-level worker_profile model.

4. apm/src/cmd/show.rs print_ticket — verify whether actionable_by or any other dropped field is surfaced in apm show output. Update accordingly.

5. apm/src/cmd/list.rs and apm/src/cmd/next.rs — verify their output for any reference to old schema concepts.

6. README files: top-level README.md, apm-core/README.md, apm-server/README.md, apm-ui/README.md. Search for 'transition.role', 'transition.worker_profile', 'actionable_by', 'role: worker', 'agent/supervisor'.

7. docs/ directory if present. Search for the same terms.

8. apm-core/src/default/agents/claude/apm.*.md role-file templates and project copies. These describe the worker's mental model. If they reference 'role: worker' or 'actionable_by' verbiage, update.

9. apm-core/src/init.rs — if init scaffolding writes any prose about the workflow, audit.

APPROACH:
- Use grep / ripgrep for old schema vocabulary:
  - 'transition.role' / 'transition.worker_profile' / 'actionable_by' / 'Actionable by'
  - 'role: worker' / 'role:worker' / '"worker"' in role context
- Read each hit, decide if it is stale, update with new vocabulary
- Build the project, run cargo test --workspace, exercise apm <subcommand> --help for each command to spot-check rendered text

TESTS:
- apm instructions one-liner does not contain 'shell discipline'
- apm prompt --help long_about does not contain the old layer order labels or the old 'skipped:'/'level N — not reached' format
- A grep for 'role: worker' across the repository (excluding tickets/ and archive/) returns nothing

OUT OF SCOPE:
- Functional code changes (schema, dispatch, validate, instructions filter all have their own tickets)
- apm-server / apm-ui (separate ticket)
- External-project migration docs (separate ticket)
- Updating ticket / archive markdown files (historical record)

REFERENCES:
- apm/src/main.rs (clap command definitions and long_about)
- apm/src/cmd/* for individual subcommand help
- README.md and project documentation files
- apm-core/src/default/agents/claude/*.md role files

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
| 2026-05-31T01:59Z | — | new | philippepascal |
| 2026-05-31T03:03Z | new | closed | philippepascal |
