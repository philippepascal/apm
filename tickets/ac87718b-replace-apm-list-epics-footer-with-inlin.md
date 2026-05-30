+++
id = "ac87718b"
title = "Replace apm list epics footer with inline ↓ marker on tickets whose epic is behind main"
state = "in_design"
priority = 5
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ac87718b-replace-apm-list-epics-footer-with-inlin"
created_at = "2026-05-30T02:17:52.780155Z"
updated_at = "2026-05-30T02:36:22.339172Z"
+++

## Spec

### Problem

The `apm list` output includes an `epics:` footer block (introduced by ticket 7a76dd16) that lists each stale epic with its commit count and conflict label. This adds vertical noise to the most-used triage command and forces the user to mentally cross-reference the footer against ticket rows above to determine which specific tickets are on stale epics.

`apm epic list` already surfaces per-epic freshness in full detail. In `apm list`, the actionable signal is *which tickets* are on stale epics — the commit count and conflict label are secondary. Replacing the footer with a bare `↓` marker inline on each affected ticket row delivers that signal at the point of relevance and removes the footer entirely.

### Acceptance criteria

- [ ] `apm list` output never contains an `epics:` section header or footer block under any circumstances.
- [ ] A ticket whose `target_branch` starts with `epic/` and whose epic branch is behind `main` shows `↓` appended to the epic ID in the base column (e.g. `ab12cd34↓`).
- [ ] A ticket whose `target_branch` starts with `epic/` and whose epic branch is up to date with `main` shows no `↓` in its row.
- [ ] A ticket with no `target_branch` (main-scoped) shows no `↓` in its row.
- [ ] A ticket whose `target_branch` does not start with `epic/` shows no `↓` in its row.
- [ ] When two tickets share the same stale epic, both rows show `↓` (epic freshness is deduped per epic ID, not computed per ticket row).
- [ ] The `↓` marker appears unchanged in piped (non-TTY) output — no ANSI codes surround it.
- [ ] `apm epic list` output is unchanged by this ticket.

### Out of scope

- Changes to `apm epic list` (it already shows the full freshness label per ticket 7a76dd16).
- Changes to `apm-server` or `apm-ui` (the SupervisorView chip bar from ticket 7a76dd16 is unaffected).
- Changes to `apm next` (its existing freshness note when the top ticket has a stale epic stays as-is).
- A new CLI flag to toggle the marker or restore the footer.
- Changes to `merge_tree_status` itself.
- Color highlighting of the `↓` marker (the marker renders in plain text; color is a future concern).

### Approach

The only file that changes is `apm/src/cmd/list.rs`. The current code has two separate passes over `filtered`: a row-printing loop followed by an epic-footer loop. The change inverts this: epic freshness is pre-computed before any rows are printed, then consumed during the row loop.

#### Step 1 — Pre-compute epic freshness

Before the row loop, iterate over `filtered` and build `epic_map: BTreeMap<String, String>` keyed by epic ID (value: full branch name), restricted to tickets whose `target_branch` starts with `"epic/"`:

```rust
let mut epic_map: BTreeMap<String, String> = BTreeMap::new();
for t in &filtered {
    if let Some(tb) = t.frontmatter.target_branch.as_deref() {
        if tb.starts_with("epic/") {
            let id = apm_core::epic::epic_id_from_branch(tb).to_owned();
            epic_map.entry(id).or_insert_with(|| tb.to_owned());
        }
    }
}
```

Then iterate over `epic_map`, call `merge_tree_status` once per entry, and collect stale IDs:

```rust
let mut stale_epic_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
for (id, branch) in &epic_map {
    let s = apm_core::epic::merge_tree_status(root, default_branch, branch)
        .unwrap_or(apm_core::epic::MergeStatus { ahead: 0, clean: true });
    if s.ahead > 0 {
        stale_epic_ids.insert(id.clone());
    }
}
```

Move `let default_branch = &ctx.config.project.default_branch;` to before this block (it currently appears inside the footer block near line 70).

#### Step 2 — Inline marker in the row loop

In the row loop, change how `base` is computed. The current code:

```rust
let base = match fm.target_branch.as_deref() {
    Some(branch) => apm_core::epic::epic_id_from_branch(branch).to_owned(),
    None => ctx.config.project.default_branch.clone(),
};
```

Becomes:

```rust
let base = match fm.target_branch.as_deref() {
    Some(branch) if branch.starts_with("epic/") => {
        let id = apm_core::epic::epic_id_from_branch(branch);
        if stale_epic_ids.contains(id) {
            format!("{}↓", id)
        } else {
            id.to_owned()
        }
    }
    Some(branch) => apm_core::epic::epic_id_from_branch(branch).to_owned(),
    None => ctx.config.project.default_branch.clone(),
};
```

The `{:<12}` width specifier counts Unicode scalar values, so `ab12cd34↓` (9 scalars) fits within the 12-char column with 3 spaces of padding. No column-width change is needed.

#### Step 3 — Remove the footer block

Delete the entire "Epic freshness footer" section (current lines ~69–99): the `epic_map` build loop, the `stale_epics` Vec, the `if !stale_epics.is_empty()` block, the `println!("  epics:")` call, and the per-epic `println!` inside it.

#### Step 4 — Tests in `apm/tests/integration.rs`

Add `apm_list_epic_stale_marker`:
- Set up a repo via `init_repo()`.
- Create a stale epic: branch `epic/aa000001-stale-epic` from `main`, then add a commit to `main` (making the epic branch behind).
- Create a fresh epic: branch `epic/bb000002-fresh-epic` after the new `main` commit.
- Commit three tickets to their respective ticket branches (using `commit_ticket_to_branch`):
  - `t_stale` with `target_branch = "epic/aa000001-stale-epic"`.
  - `t_fresh` with `target_branch = "epic/bb000002-fresh-epic"`.
  - `t_main` with no `target_branch`.
- Run `run_apm(p, &["list", "--all", "--no-aggressive"])`.
- Assert: stale ticket row contains `↓`; fresh ticket row does not; main-scoped row does not; no line contains `"epics:"`.

Add `apm_list_shared_epic_stale_marker`:
- Same stale epic setup as above, but two tickets both scoped to `epic/aa000001-stale-epic`.
- Assert both rows contain `↓`.

No existing tests assert on the `"epics:"` footer wording (confirmed by grep), so no existing tests need updating.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T02:17Z | — | new | philippepascal |
| 2026-05-30T02:18Z | new | groomed | philippepascal |
| 2026-05-30T02:32Z | groomed | in_design | philippepascal |