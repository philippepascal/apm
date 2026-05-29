+++
id = "7a76dd16"
title = "Surface epic freshness (behind-main status) in apm commands and UI"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7a76dd16-surface-epic-freshness-behind-main-statu"
created_at = "2026-05-29T01:17:24.701677Z"
updated_at = "2026-05-29T01:41:50.320975Z"
depends_on = ["12f2c7fa"]
+++

## Spec

### Problem

BACKGROUND: Epics are git branches (epic/<id>-<slug>). A ticket on an epic has target_branch = the epic branch, and apm start merges the EPIC branch (not main) into the ticket worktree. So if the epic branch is behind main and lacks content the tickets depend on (e.g. scaffold that landed on main after the epic forked), workers either get confused looking for missing files or recreate them, causing merge conflicts when the epic later merges back to main. Today there is NO indication anywhere that an epic has fallen behind main; the staleness is silent until a worker trips over it.

GOAL: Surface epic 'freshness' relative to main in a way that is VISIBLE but NON-ANNOYING. It must be informational only — never block dispatch, never auto-act.

WHAT TO SURFACE: For each epic, a concise indicator of (a) how many commits main is ahead of the epic branch, and (b) whether main merges cleanly into the epic or would conflict. Use git merge-tree for the clean/conflict check (it is already used elsewhere in the codebase and needs no working-tree changes). 'Behind by N (clean)' vs 'behind by N (CONFLICTS)' vs 'up to date'.

WHERE TO SURFACE:
- Common apm CLI commands should show it where epics are already visible — at minimum apm epic show and any epic listing, and consider a brief line in apm list / apm next / apm status output when an epic is involved. The bar is: a supervisor doing normal triage should notice it without having to run a special command.
- The UI (apm-server + apm-ui) should surface it too — e.g. a small badge/chip on the epic in the board/supervisor view, color-coded (clean vs conflict).

NON-ANNOYING CONSTRAINTS: This is a nudge, not a nag. Raw 'behind by N' is a noisy signal (main is almost always ahead of an epic), so the display must be low-key — a short status string/badge, not a warning or prompt, and never repeated noisily across every line. Keep it cheap to compute so it does not slow common commands; the spec-writer should decide when/how often the freshness is computed (e.g. lazily, cached, or only for the epic actually being shown).

PURPOSE: The decision of whether an epic needs main's changes belongs to the supervisor (APM cannot tell which of main's commits are relevant to the epic). This surfacing exists only to give the supervisor the cue to decide and then run apm refresh-epic.

SHARED PRIMITIVE: The clean/conflict detection (main -> epic via git merge-tree) is the same primitive needed by the apm refresh-epic changes (separate ticket). Implement it once as a reusable helper in apm-core rather than duplicating.

OUT OF SCOPE: auto-merging main into the epic; blocking or gating dispatch on staleness; an 'accept divergence' mechanism. Those are deliberately deferred (the accept mechanism in particular would require new epic-level metadata that does not exist today).

### Acceptance criteria

- [ ] All freshness computation uses `apm_core::epic::merge_tree_status` (defined in ticket 12f2c7fa); no new `EpicFreshness` struct or `epic_freshness` function is added in this ticket. Display mapping: `behind = status.ahead`, `conflicts = !status.clean`.
- [ ] `apm epic list` includes a freshness indicator per epic: "up to date", "↓N clean", or "↓N CONFLICTS".
- [ ] `apm epic show <id>` prints a "Freshness:" line immediately after the "State:" line.
- [ ] `apm list` appends an epic freshness footer after the ticket rows with one entry per distinct epic that has at least one visible ticket; the footer is omitted when no visible ticket has an epic.
- [ ] `apm next` prints a freshness note on a line after the ticket line when the top ticket's `epic` field is set (non-JSON mode only; JSON output is unchanged).
- [ ] `GET /api/epics` and `GET /api/epics/:id` include `behind_count` (integer ≥ 0) and `conflicts` (boolean) on every epic object in the response.
- [ ] `SupervisorView.tsx` renders a freshness chip bar below the filter row for epics that have at least one ticket in the loaded set; chips are color-coded amber for behind/clean and red for behind/conflicts; the bar is hidden when all epics are up to date.
- [ ] No code path that computes freshness is reachable from `apm state`, `apm start`, or `apm dispatch`; freshness is display-only.

### Out of scope

- Auto-merging or refreshing the epic from main — that is `apm refresh-epic` (separate ticket)
- Blocking `apm start`, ticket dispatch, or any state transition on freshness status
- An "accept divergence" or "suppress this indicator" mechanism
- Any new TOML field in `apm.toml` or ticket frontmatter
- Persisting or caching freshness across invocations
- Freshness for ticket branches relative to their epic — only epic branches relative to `config.project.default_branch`

### Approach

#### Core primitive (`apm-core/src/epic.rs`)

This ticket relies on `apm_core::epic::merge_tree_status` (added by ticket 12f2c7fa) — no new git merge-tree wrapper is added here.

Field mapping for display: `behind = status.ahead`, `conflicts = !status.clean`.

Add a private helper `fn freshness_label(ahead: usize, clean: bool) -> String` in `apm/src/cmd/epic.rs` used by all CLI call sites:
- `ahead == 0` → `"up to date"`
- `ahead > 0 && clean` → `format!("↓{} clean", ahead)`
- `ahead > 0 && !clean` → `format!("↓{} CONFLICTS", ahead)`

Call sites call `merge_tree_status(root, &default_branch, epic_branch)` with `.unwrap_or(MergeStatus { ahead: 0, clean: true })` and pass `s.ahead, s.clean` to `freshness_label`.

No new unit tests in `apm-core/src/epic.rs` for this ticket — `merge_tree_status` is tested in ticket 12f2c7fa.

#### CLI: `apm epic list` and `apm epic show` (`apm/src/cmd/epic.rs`)

`run_list`: load `default_branch` from `ctx.config.project.default_branch`. After computing `counts_str`, call `apm_core::epic::merge_tree_status(root, &default_branch, branch)`, unwrap with `.unwrap_or(MergeStatus { ahead: 0, clean: true })`. Append the label as a trailing column:

```rust
println!("{id:<8} [{derived:<12}] {title:<40} {counts_str:<30} {}", freshness_label(s.ahead, s.clean));
```

`run_show`: after `println!("State:  {derived}");`, add:

```rust
let s = apm_core::epic::merge_tree_status(root, &ctx.config.project.default_branch, &branch)
    .unwrap_or(MergeStatus { ahead: 0, clean: true });
println!("Freshness: {}", freshness_label(s.ahead, s.clean));
```

#### CLI: `apm list` (`apm/src/cmd/list.rs`)

After the existing stale-tickets footer block, collect distinct `target_branch` values from `filtered` tickets that have one set. Deduplicate into a `BTreeMap<epic_id, branch>`. For each, call `apm_core::epic::merge_tree_status(root, &default_branch, branch)`. Filter to those with `s.ahead > 0`. If any remain, print:

```
(blank line)
  epics:
    {id:<8}  ↓N clean
    {id:<8}  ↓N CONFLICTS
```

Only computes once per distinct epic branch regardless of how many tickets share it.

#### CLI: `apm next` (`apm/src/cmd/next.rs`)

In the `Some(t)` non-JSON branch: if `fm.epic.is_some()`, call `apm_core::epic::find_epic_branch(root, epic_id)`. If found, call `apm_core::epic::merge_tree_status(root, &default_branch, epic_branch)` and print `"  (epic {id}: {label})"` on the next line.

#### Server: models and handlers (`apm-server/`)

`models.rs` — add two fields to `EpicSummary`:

```rust
pub behind_count: u32,
pub conflicts: bool,
```

`handlers/epics.rs` — `build_epic_summary` gains `root: &Path` and `default_branch: &str` parameters. Call `apm_core::epic::merge_tree_status(root, default_branch, branch)` and populate the new fields (`behind_count = s.ahead as u32`, `conflicts = !s.clean`); default to `(0, false)` on error. Update the two callers (`list_epics`, `get_epic`) to pass `root` and `config.project.default_branch`. In `create_epic`, set `behind_count: 0, conflicts: false` on the returned summary (newly-created epics branch from main and are always current).

Extend `create_epic_round_trip` test to assert `behind_count == 0` and `conflicts == false`.

#### Frontend (`apm-ui/src/components/supervisor/SupervisorView.tsx`)

Extend the inline `Epic` interface:

```ts
interface Epic { id: string; title: string; branch: string; behind_count: number; conflicts: boolean }
```

After the filter bar closing `</div>` (currently line ~253), add a stale-epic chip bar. Compute `staleEpics` by filtering `epics` to those where `behind_count > 0` AND the epic ID appears in the loaded `tickets` (to avoid showing epics with no open tickets). If `staleEpics` is empty, render nothing.

Each chip is a `<span>` with `onClick={() => setEpicFilter(ep.id)}`:
- Amber chip class for `!conflicts`: `bg-amber-800/50 text-amber-200 border border-amber-600`
- Red chip class for `conflicts`: `bg-red-900/50 text-red-200 border border-red-600`
- Text: `"{ep.id.slice(0,8)} ↓{ep.behind_count}"` plus `" conflicts"` when `ep.conflicts`

No changes to `Swimlane.tsx` or `TicketCard.tsx`.

### Open questions


### Amendment requests

- [ ] Consume the shared merge-status helper from ticket 12f2c7fa (now a depends_on) instead of defining a new one. CHANGES: (1) DELETE this ticket's own EpicFreshness struct and epic_freshness() function from the Approach — do NOT add a second git merge-tree / merge-base helper to apm-core/src/epic.rs. (2) 12f2c7fa adds the canonical helper: pub fn merge_tree_status(root, default_branch, epic_branch) -> Result<MergeStatus> where MergeStatus { ahead: usize, clean: bool }. Use it everywhere this spec previously called epic_freshness. (3) Map fields for display: behind = MergeStatus.ahead (commits default_branch is ahead of the epic), conflicts = !MergeStatus.clean. (4) KEEP all display/surfacing work unchanged in intent: the freshness_label() formatter ('up to date' / '↓N clean' / '↓N CONFLICTS'), apm epic list/show, the apm list epics footer, the apm next epic note, the server /api/epics + /api/epics/:id fields (keep field names behind_count and conflicts, computed as ahead and !clean), and the SupervisorView chip bar. (5) The behind==0 short-circuit is already handled inside merge_tree_status (ahead==0 => clean, no merge-tree run), so callers need no separate guard. (6) Because of depends_on 12f2c7fa, merge_tree_status will already exist when this ticket is implemented — rely on it; do not reimplement. Update the Approach 'Core primitive' section and all ACs that reference EpicFreshness/epic_freshness accordingly.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T01:17Z | — | new | philippepascal |
| 2026-05-29T01:18Z | new | groomed | philippepascal |
| 2026-05-29T01:18Z | groomed | in_design | philippepascal |
| 2026-05-29T01:26Z | in_design | specd | claude |
| 2026-05-29T01:41Z | specd | ammend | philippepascal |
| 2026-05-29T01:41Z | ammend | in_design | philippepascal |