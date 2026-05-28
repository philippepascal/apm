+++
id = "4d36d9bb"
title = "apm sync does not detect tickets merged into their target branch"
state = "ready"
priority = 6
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d36d9bb-apm-sync-does-not-detect-tickets-merged-"
created_at = "2026-05-28T20:46:27.893432Z"
updated_at = "2026-05-28T20:54:43.026322Z"
+++

## Spec

### Problem

Since b0ea6a04 (April 3), the Merge completion strategy routes tickets with `target_branch` set into that branch (e.g. an epic branch) rather than always into main. `sync::detect` was not updated alongside that change: all three existing passes (Cases 1, 2, 3) check merges only against the project's default branch. Tickets merged into an epic branch therefore stay in `implemented` state permanently and, since 14338748 (May 3), emit a spurious hint asking the supervisor to close them manually.

The fix adds a Case 4 after Case 3 in `sync::detect`. It iterates every `implemented` ticket branch that the earlier passes did not recognise; for any that carry a non-empty `target_branch` field, it checks whether that branch has been merged — by regular merge or squash — into the named target. Matches are added to `merged_set` (suppressing the hint) and to the close-candidates list, mirroring exactly what Case 1 already does for main-merged tickets.

### Acceptance criteria

- [ ] `apm sync` closes an `implemented` ticket whose branch is regular-merged (--no-ff) into the branch named in its `target_branch` field, with close reason `"branch merged into target"`
- [ ] `apm sync` closes an `implemented` ticket whose branch is squash-merged into its `target_branch`
- [ ] `apm sync` does not emit the "close manually" hint for a ticket auto-closed by the new target-branch pass
- [ ] Tickets without a `target_branch` field continue to be detected (or not) exactly as before — no regression in Cases 1, 2, or 3
- [ ] `apm sync` does not error or falsely close a ticket whose `target_branch` value does not exist locally
- [ ] An integration test in `apm/tests/integration.rs` verifies Case 4 for a regular merge into `target_branch`
- [ ] An integration test in `apm/tests/integration.rs` verifies Case 4 for a squash merge into `target_branch`

### Out of scope

- Case 3 analog for `target_branch`: detecting content-merged-but-trailing-state-commits into a non-default target (the `content_merged_into_main` logic extended to arbitrary refs)
- Case 2 analog for `target_branch`: detecting implemented tickets whose branch has been deleted after merging into the target
- Changing how `apm state <id> implemented` routes merges or PR bases
- Remote-only `target_branch` values (refs not fetched locally); those return false and generate no hint

### Approach

#### New function in `apm-core/src/git_util.rs`

Add `pub fn is_branch_merged_into(root: &Path, branch: &str, target_ref: &str) -> Result<bool>`.

1. Regular-merge check: `if is_ancestor(root, branch, target_ref) { return Ok(true); }` — `is_ancestor` returns `false` on any git error (unknown ref included), so no special-casing is needed.
2. Squash-merge check — mirrors the private `squash_merged` helper:
   a. `run(root, &["merge-base", target_ref, branch])` — return `Ok(false)` on error (target ref absent locally).
   b. `run(root, &["rev-parse", &format!("{branch}^{{commit}}")])` — return `Ok(false)` on error.
   c. If branch_tip == merge_base, return `Ok(true)` (belt-and-suspenders; already caught in step 1).
   d. Create virtual squash commit: `git commit-tree <branch>^{tree} -p <merge_base> -m "squash"` — return `Ok(false)` on error.
   e. `git cherry <target_ref> <squash_commit>` — return `Ok(false)` on error.
   f. Return `Ok(cherry.trim().starts_with('-'))`.

#### Case 4 in `apm-core/src/sync.rs`

Insert between Case 3 (line 88) and the hint loop (line 110):

```rust
// Case 4: implemented tickets merged into their target_branch.
for branch in &branches {
    if merged_set.contains(branch.as_str()) { continue; }
    let suffix = branch.trim_start_matches("ticket/");
    let rel_path = format!("{tickets_dir}/{suffix}.md");
    let content = match git::read_from_branch(root, branch, &rel_path) {
        Ok(c) => c,
        Err(_) => continue,
    };
    let t = match Ticket::parse(&root.join(&rel_path), &content) {
        Ok(t) => t,
        Err(_) => continue,
    };
    if t.frontmatter.state != "implemented" { continue; }
    let target = match t.frontmatter.target_branch.as_deref() {
        Some(tb) if !tb.is_empty() => tb.to_string(),
        _ => continue,
    };
    if git::is_branch_merged_into(root, branch, &target)? {
        merged_set.insert(branch.clone());
        close.push(CloseCandidate { ticket: t, reason: "branch merged into target" });
    }
}
```

#### Tests in `apm/tests/integration.rs`

Test 1 (regular merge): create a repo with `main` and an `epic/foo` branch; create a ticket branch with `target_branch = "epic/foo"` and state `implemented`; merge the ticket branch into `epic/foo` with `git merge --no-ff`; run `detect()`; assert the ticket is in `close` and `hints` is empty.

Test 2 (squash merge): same setup, but merge with `git merge --squash && git commit`; assert the same outcome.

#### Files changed

- `apm-core/src/git_util.rs`: add `is_branch_merged_into`
- `apm-core/src/sync.rs`: add Case 4 between Case 3 and the hint loop
- `apm/tests/integration.rs`: add two tests

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T20:46Z | — | new | philippepascal |
| 2026-05-28T20:46Z | new | groomed | philippepascal |
| 2026-05-28T20:46Z | groomed | in_design | philippepascal |
| 2026-05-28T20:52Z | in_design | specd | claude |
| 2026-05-28T20:54Z | specd | ready | philippepascal |
