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

- [ ] \`apm_core::epic::set_epic_owner(root, epic_id, new_owner, config)\` exists as a public function and returns \`(usize, usize)\` (changed, skipped counts)
- [ ] \`set_epic_owner\` loads all tickets, filters to those belonging to the given epic, skips terminal-state tickets, and bulk-updates the \`owner\` field by committing to each ticket's branch
- [ ] \`run_set()\` in \`epic.rs\` delegates the owner-cascade work entirely to \`set_epic_owner()\`; all ownership-iteration code is removed from the CLI layer
- [ ] \`apm_core::github::gh_pr_create_or_update\` accepts a \`body: &str\` parameter; the existing caller in \`state.rs\` passes \`&format!("Closes #{id}")\` explicitly
- [ ] \`run_close()\` calls \`apm_core::github::gh_pr_create_or_update()\` with the epic-appropriate body (\`"Epic: {epic_branch}"\`) and removes its inline \`gh pr list\` idempotency check and inline \`gh pr create\` block
- [ ] \`run_close()\` calls \`apm_core::epic::epic_id_from_branch()\` instead of the inline trim/split expression (dep \`aeacd066\` must be merged first)
- [ ] The local \`branch_to_title()\` definition is absent from \`epic.rs\` (removed by dep \`aeacd066\`); \`run_close()\` calls \`apm_core::epic::branch_to_title()\`
- [ ] \`set_epic_owner\` has unit tests covering: happy path (owner updated on non-terminal tickets), skipping terminal tickets
- [ ] \`cargo test\` passes across all crates

### Out of scope

- Extracting \`run_list()\`, \`run_show()\`, or \`run_new()\` logic from \`epic.rs\`
- Moving the \`max_workers\` branch of \`run_set()\` (TOML editing) out of the CLI layer
- Changing the PR body format used for epic PRs
- Refactoring any other command files in \`apm/src/cmd/\`
- Moving \`branch_to_title()\` or \`epic_id_from_branch()\` to core (covered by dep \`aeacd066\`)
- Creating \`apm::util\` or its helpers (covered by dep \`d3ebdc0f\`)

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