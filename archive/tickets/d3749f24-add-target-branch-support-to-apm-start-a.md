+++
id = "d3749f24"
title = "Add target_branch support to apm start and worktree provisioning"
state = "closed"
priority = 8
effort = 2
risk = 2
author = "claude-0401-2145-a8f3"
agent = "64101"
branch = "ticket/d3749f24-add-target-branch-support-to-apm-start-a"
created_at = "2026-04-01T21:55:32.694733Z"
updated_at = "2026-04-02T02:51:01.384885Z"
+++

## Spec

### Problem

When a ticket belongs to an epic, its worktree must be provisioned from the epic branch (not `main`), and its PR must target the epic branch (not `main`). Currently `apm start` always uses `config.project.default_branch` for both: as the merge source when setting up the worktree, and as the `--base` argument when creating the PR via `gh_pr_create_or_update`.

The `docs/epics.md` design (§ Workflow integration) specifies that when `target_branch` is set in ticket frontmatter, `apm start` should merge from that branch into the worktree instead of the default branch. The PR creation call site should also pass `target_branch` as the base. Tickets without `target_branch` are completely unaffected.

The `target_branch` field does not yet exist on the `Frontmatter` struct in `apm-core/src/ticket.rs`, so it must be added before the two call-site changes can be made.

### Acceptance criteria

- [x] When a ticket has `target_branch = "epic/abc"` in its frontmatter, `apm start <id>` merges `epic/abc` (or `origin/epic/abc` if the remote ref exists) into the worktree instead of the default branch
- [x] When a ticket has no `target_branch` field, `apm start <id>` behaves identically to before this change
- [x] When a ticket has `target_branch` set and `apm state <id> implemented` is called, the PR is created with `--base epic/abc` (the target branch), not with `--base main`
- [x] When a ticket has no `target_branch` field and `apm state <id> implemented` is called, the PR is created with `--base main` (the default branch) as before
- [x] The `Frontmatter` struct round-trips a ticket file that contains `target_branch` without data loss
- [x] The `Frontmatter` struct round-trips a ticket file that does not contain `target_branch` without adding the field to the serialised output

### Out of scope

- Setting `target_branch` automatically when a ticket is created under an epic (covered by a separate epic-creation ticket)
- The `epic` and `depends_on` frontmatter fields — not added here
- Validating that `target_branch` actually exists in the repo at the time `apm start` runs
- Any UI or `apm` CLI command changes to display or filter by `target_branch`
- Epic branch lifecycle (creation, merging the epic branch back to main)

### Approach

Three files change; all changes are small.

**1. `apm-core/src/ticket.rs` — add `target_branch` to `Frontmatter`**

Add one field to the struct after the existing optional fields:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub target_branch: Option<String>,
```

The `skip_serializing_if` attribute ensures existing ticket files are not affected.

**2. `apm-core/src/start.rs` — use `target_branch` as the merge source**

Around line 163, `default_branch` is read from `config.project.default_branch`. The merge logic (lines 179-216) currently uses `default_branch` to build the merge ref (`origin/{default_branch}` or local `default_branch`). Replace every use of `default_branch` in that merge block with a local variable:

```rust
let merge_base = ticket.frontmatter.target_branch.as_deref().unwrap_or(default_branch);
```

Then use `merge_base` in place of `default_branch` when constructing the remote ref and the fallback ref. The `default_branch` binding is still needed for nothing else in that function after this substitution.

**3. `apm-core/src/state.rs` — pass `target_branch` to PR creation**

At the call site around line 138:

```rust
// before
gh_pr_create_or_update(root, &branch, &config.project.default_branch, &id, &t.frontmatter.title)?;

// after
let pr_base = t.frontmatter.target_branch.as_deref().unwrap_or(&config.project.default_branch);
gh_pr_create_or_update(root, &branch, pr_base, &id, &t.frontmatter.title)?;
```

No change to the `gh_pr_create_or_update` function signature is needed.

**Tests**

Add an integration test in `apm/tests/integration.rs` that:
1. Creates a temp repo with a `main` branch and an `epic/e1-foo` branch that has a unique commit
2. Creates a ticket with `target_branch = "epic/e1-foo"` in its frontmatter
3. Calls `apm start` on that ticket
4. Asserts the worktree was created and that the unique commit from the epic branch is present in the worktree history (confirming the merge source was `epic/e1-foo`)

A unit test in `apm-core/src/ticket.rs` verifying round-trip serialization of the new field (present and absent) covers acceptance criteria 5 and 6.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:43Z | groomed | in_design | philippepascal |
| 2026-04-02T00:46Z | in_design | specd | claude-0402-spec-d3749f24 |
| 2026-04-02T02:28Z | specd | ready | apm |
| 2026-04-02T02:38Z | ready | in_progress | philippepascal |
| 2026-04-02T02:46Z | in_progress | implemented | claude-0401-2300-w4rk |
| 2026-04-02T02:51Z | implemented | closed | apm-sync |