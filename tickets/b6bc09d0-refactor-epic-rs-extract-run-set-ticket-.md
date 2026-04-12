+++
id = "b6bc09d0"
title = "Refactor epic.rs: extract run_set ticket logic and apply shared helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b6bc09d0-refactor-epic-rs-extract-run-set-ticket-"
created_at = "2026-04-12T09:02:48.936896Z"
updated_at = "2026-04-12T09:28:51.684207Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f", "aeacd066"]
+++

## Spec

### Problem

\`apm/src/cmd/epic.rs\` (439 lines) is the largest command file and contains two pieces of domain logic that belong in \`apm_core\` rather than the CLI layer:

**Owner cascade in \`run_set()\`** (lines ~252–300): When setting an epic's owner, the function loads all tickets across the epic, pre-flight checks ownership on each, bulk-updates the \`owner\` field, and commits to each ticket's branch. This is a domain operation — mutating a collection of tickets — that should live in \`apm_core::epic\` as \`set_epic_owner()\` so other callers (e.g. a future server endpoint) can reuse it without going through the CLI.

**PR creation in \`run_close()\`** (lines ~108–152): The function re-implements both the idempotency check (\`gh pr list\`) and the \`gh pr create\` invocation inline, duplicating logic already extracted to \`apm_core::github::gh_pr_create_or_update()\`. It should delegate to the shared function. The only difference is the PR body (epics use \`"Epic: {branch}"\` instead of \`"Closes #{id}"\`), which is resolved by adding a \`body: &str\` parameter to the shared function.

Once the prerequisite tickets land, two additional call-sites need updating in this file:
- dep \`aeacd066\` moves \`branch_to_title()\` and epic-ID parsing to \`apm_core::epic\`; \`run_close()\` still has one inline ID-parsing expression that should be replaced with \`epic_id_from_branch()\`.
- dep \`d3ebdc0f\` adds \`apm::util\` helpers; \`epic.rs\` currently has no matching patterns (no confirmation prompts, no aggressive-fetch blocks), so this is a verify-only step.

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:28Z | groomed | in_design | philippepascal |