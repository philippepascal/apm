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

This ticket has three independent changes plus a verify step. Deps \`d3ebdc0f\` and \`aeacd066\` must be merged to the target branch before starting.

**1. Add \`set_epic_owner\` to \`apm_core/src/epic.rs\`**

Signature:
\`\`\`rust
pub fn set_epic_owner(
    root: &Path,
    epic_id: &str,      // 8-char ID parsed from the epic branch
    new_owner: &str,    // already-validated owner string
    config: &Config,
) -> Result<(usize, usize)>  // (changed, skipped)
\`\`\`

Logic (mirrors the existing \`run_set()\` owner block):
- \`ticket::load_all_from_git(root, &config.tickets.dir)\`
- Filter to tickets whose branch contains \`epic_id\`
- Partition: terminal-state tickets go to \`skipped\`, the rest to \`to_change\`
- Pre-flight: for each ticket in \`to_change\`, call \`ticket::check_owner(root, &t)?\`
- For each ticket in \`to_change\`:
  - \`ticket::set_field(&mut t.frontmatter, "owner", new_owner)?\`
  - Serialise and call \`git::commit_to_branch(root, &branch, &rel_path, &content, &msg)\`
- Return \`(to_change.len(), skipped.len())\`

The owner validation (\`validate::validate_owner\`) stays in the CLI layer in \`run_set()\` because it needs the local git user and \`Config\`, which the CLI already has. Pass only the pre-validated string to the core function.

Add unit tests in \`apm_core/src/epic.rs\`: happy path with a mix of terminal and non-terminal tickets, and a test that terminal tickets are skipped.

**2. Update \`run_set()\` in \`apm/src/cmd/epic.rs\`**

Replace the owner-cascade block (lines ~252–300) with:
\`\`\`rust
let (changed, skipped) = apm_core::epic::set_epic_owner(root, &epic_id, value, &config)?;
println!("updated {changed} ticket(s), skipped {skipped} terminal ticket(s)");
\`\`\`
Keep the validation steps that currently precede the cascade (epic existence check, owner validation).

**3. Add \`body: &str\` to \`gh_pr_create_or_update\` in \`apm_core/src/github.rs\`**

Change the signature to:
\`\`\`rust
pub fn gh_pr_create_or_update(
    root: &Path,
    branch: &str,
    default_branch: &str,
    id: &str,
    title: &str,
    body: &str,           // new parameter
    messages: &mut Vec<String>,
) -> Result<()>
\`\`\`
Replace the hardcoded \`format!("Closes #{id}")\` body with the \`body\` parameter.

Update the existing caller in \`apm_core/src/state.rs\` to pass \`&format!("Closes #{id}")\` explicitly.

**4. Update \`run_close()\` in \`apm/src/cmd/epic.rs\`**

Remove:
- Lines ~108–127: the \`gh pr list\` idempotency check block
- Lines ~133–152: the \`Command::new("gh").args(["pr", "create", ...])\` block

Replace with a single call after \`git::push_branch_tracking(root, &epic_branch)?\`:
\`\`\`rust
let mut messages = vec![];
apm_core::github::gh_pr_create_or_update(
    root,
    &epic_branch,
    &default_branch,
    &epic_id,
    &pr_title,
    &format!("Epic: {epic_branch}"),
    &mut messages,
)?;
for m in &messages { println!("{m}"); }
\`\`\`

**5. Verify dep-introduced call-sites (no new work if deps landed correctly)**

After deps merge, confirm \`epic.rs\` already uses:
- \`apm_core::epic::branch_to_title()\` — should have been updated by \`aeacd066\`
- \`apm_core::epic::epic_id_from_branch()\` — \`run_close()\` line ~76 inline pattern; update if dep didn't cover it

**Order of changes:** steps 1→2 are independent of steps 3→4; both pairs can be done in parallel. Step 5 is last.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:28Z | groomed | in_design | philippepascal |