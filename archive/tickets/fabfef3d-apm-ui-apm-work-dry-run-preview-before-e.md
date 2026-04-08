+++
id = "fabfef3d"
title = "apm-ui: apm work dry-run preview before engine start"
state = "closed"
priority = 20
effort = 3
risk = 2
author = "apm"
agent = "31108"
branch = "ticket/fabfef3d-apm-ui-apm-work-dry-run-preview-before-e"
created_at = "2026-03-31T06:13:13.767038Z"
updated_at = "2026-04-01T07:12:58.252579Z"
+++

## Spec

### Problem

When the apm work engine is stopped, users have no way to preview which tickets would be dispatched if they started it. They must either start the engine and watch what it does, or run `apm work --dry-run` on the command line. Neither gives visibility directly from the UI before committing to a start.

The CLI already implements the dry-run logic in `apm/src/cmd/work.rs:run_dry()`: load all tickets from git, filter to actionable+startable states, sort by score (priority_weight × priority + effort_weight × effort + risk_weight × risk), and take up to `max_concurrent`. This ticket exposes that same logic through a `GET /api/work/dry-run` HTTP endpoint and renders the result in a preview panel in the workerview column, visible whenever the engine is stopped.

### Acceptance criteria

- [x] `GET /api/work/dry-run` returns HTTP 200 with a JSON object `{ "candidates": [...] }`
- [x] Each candidate object includes: `id`, `title`, `state`, `priority`, `effort`, `risk`, `score` (float)
- [x] Candidates are sorted by score descending, matching the order `apm work --dry-run` produces
- [x] The response contains at most `config.agents.max_concurrent` candidates
- [x] The response returns `{ "candidates": [] }` (empty array) when there are no actionable tickets
- [x] The dry-run preview panel is visible in the workerview column when the engine is stopped
- [x] The panel calls `GET /api/work/dry-run` and renders each candidate as a row showing id, title, and state
- [x] The panel shows an empty-state message when candidates is empty
- [x] The panel is hidden when the engine is running

### Out of scope

- The start/stop button and engine status indicator (covered by Step 12a)
- Assigning specific named worker processes to candidate tickets (workers are spawned dynamically at start time; the preview shows candidates, not assignments)
- Auto-refresh / polling of the dry-run panel (a manual refresh button is sufficient)
- Filtering candidates by agent or any other criterion beyond what the existing score-based algorithm already does
- Any changes to the CLI `apm work --dry-run` behaviour

### Approach

The implementation has two parts: a new API endpoint in `apm-server` and a new UI component in `apm-ui`. Both depend on Step 12a being merged first.

**1. Backend — `GET /api/work/dry-run` in `apm-server`**

Add a handler in `apm-server/src/routes/work.rs` (or wherever Step 12a placed the work routes):

```rust
// Response type
#[derive(serde::Serialize)]
struct DryRunCandidate {
    id: String,
    title: String,
    state: String,
    priority: u8,
    effort: u8,
    risk: u8,
    score: f64,
}

#[derive(serde::Serialize)]
struct DryRunResponse {
    candidates: Vec<DryRunCandidate>,
}
```

Logic mirrors `apm/src/cmd/work.rs:run_dry()`:
1. Load config and tickets via `Config::load(root)` and `ticket::load_all_from_git(root, &config.tickets.dir)`.
2. Determine `startable` states: those with a `command:start` trigger.
3. Determine `actionable` states: `config.actionable_states_for("agent")`.
4. Filter tickets to those whose state is in both sets.
5. Sort by `score(pw, ew, rw)` descending.
6. Take at most `config.agents.max_concurrent` entries.
7. Return 200 `{ "candidates": [...] }`.

Wire the route: `GET /api/work/dry-run` → handler, registered in the router alongside the Step 12a routes.

No new apm-core functions are needed — the filtering and scoring logic is already public.

**2. Frontend — `DryRunPreview` component in `apm-ui`**

New file: `apm-ui/src/components/DryRunPreview.tsx`

- Uses TanStack Query: `useQuery({ queryKey: ['work-dry-run'], queryFn: () => fetch('/api/work/dry-run').then(r => r.json()) })`.
- Query is enabled only when the engine is **stopped** (read `engineStatus` from the Zustand store set by Step 12a).
- Renders a shadcn/ui `Card` or `Table` inside the workerview column (below the start/stop controls from Step 12a):
  - If `candidates.length === 0`: show "No tickets ready to dispatch."
  - Otherwise: one row per candidate showing `#<id>` badge, title, state badge, score.
- Include a "Refresh" icon button that calls `queryClient.invalidateQueries(['work-dry-run'])`.

**Integration point with Step 12a:**
- The panel is conditionally rendered: `{engineStatus === 'stopped' && <DryRunPreview />}`.
- `engineStatus` lives in the Zustand store introduced by Step 12a; this ticket only reads it.

**Tests:**
- Unit test the handler in `apm-server/tests/` using an in-memory config with a small set of tickets, asserting order and count of candidates.
- React component tested via Vitest + Testing Library: mock the fetch, assert rows render, assert empty-state renders.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:05Z | new | in_design | philippepascal |
| 2026-03-31T07:10Z | in_design | specd | claude-0331-0800-b7e2 |
| 2026-03-31T19:44Z | specd | ready | apm |
| 2026-04-01T06:40Z | ready | in_progress | philippepascal |
| 2026-04-01T06:48Z | in_progress | implemented | claude-0401-0640-7260 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |