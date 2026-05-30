+++
id = "778b63c6"
title = "Surface merge-failure state and recovery hints in apm-server and apm-ui (read-only)"
state = "in_progress"
priority = 0
effort = 6
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/778b63c6-surface-merge-failure-state-and-recovery"
created_at = "2026-05-30T02:11:35.270399Z"
updated_at = "2026-05-30T04:22:36.079630Z"
depends_on = ["ae4104f2"]
+++

## Spec

### Problem

The apm-ui supervisor board renders tickets in `merge_failed` (and equivalently-configured) states identically to tickets in normal states such as `in_progress` or `implemented`. When a merge operation fails, the git error is captured in the ticket body under `### Merge notes` and the ticket is moved to the failure state automatically, but the UI shows no visual cue that the ticket is stuck. The supervisor must leave the UI, run `apm show <id>` in the terminal, read the captured error, and work out which `apm state` command to run — information that should be immediately visible in the triage view.

This ticket extends `apm-server` and `apm-ui` to surface two pieces of recovery context: (a) a visual badge on the ticket card indicating merge failure, and (b) a detail panel showing the raw git error and the exact CLI commands to recover. It depends on ae4104f2, which adds `classify_recovery_options(state_id, config)` to `apm-core`. That function inspects the workflow config and classifies each available transition from a given state as `RetryMerge`, `ReturnToWorker`, `Abandon`, or `Other`, without hardcoding any state name. The server consumes this output to compute which state IDs are merge-failure states and to generate per-ticket recovery command strings; the UI renders them read-only. No state-transition API surface is added.

### Acceptance criteria

- [x] `GET /api/tickets` response envelope includes a `merge_failure_state_ids` field — a JSON array of state ID strings for which `apm_core::recovery::is_merge_failure_state` returns true — i.e. states that appear as the `on_failure` value of at least one transition whose completion is `Pr`, `Merge`, or `PrOrEpicMerge`; the array is empty when no git root is present or config fails to load.
- [x] `GET /api/tickets/:id` response includes `merge_notes` — the trimmed string content of the `### Merge notes` section of the ticket body, or `null` when that section is absent or empty.
- [x] `GET /api/tickets/:id` response includes `recovery_options` — a JSON array of objects each with `to` (target state ID), `label` (human-readable name), `kind` (one of `"retry_merge"`, `"return_to_worker"`, `"abandon"`, `"other"`), and `command` (the literal string `"apm state <ticket-id> <to>"`); the array is empty when `is_merge_failure_state(ticket.state, workflow)` returns false, or when no git root is present, or when config fails to load.
- [x] A `TicketCard` whose `ticket.state` is present in the `merge_failure_state_ids` array received from the list endpoint renders a distinct red visual marker.
- [x] A `TicketCard` whose `ticket.state` is absent from `merge_failure_state_ids` renders no merge-failure marker, even if the state name happens to be `"merge_failed"`.
- [x] The `TicketDetail` panel for a ticket with a non-null `merge_notes` value displays a "Merge failure" section with the notes rendered verbatim inside a monospace pre block.
- [x] The `TicketDetail` panel for a ticket with a non-empty `recovery_options` array displays a "Recovery" section listing each option's label, a human-readable kind description (e.g. "Retry merge", "Return to worker", "Abandon"), and the `command` string in a monospace code block styled for easy copying; the section ends with a reference link to `docs/merge-failed-recovery.md`.
- [x] A ticket whose `recovery_options` is an empty array and `merge_notes` is `null` renders neither the "Merge failure" section nor the "Recovery" section in the detail panel.
- [x] `merge_failure_state_ids` under the default workflow contains `"merge_failed"` and does not contain `"in_progress"`, `"implemented"`, or `"ready"`.
- [x] A `TicketCard` with `ticket.state` equal to `"in_progress"` does not render the merge-failure badge, even when `in_progress` has an outgoing transition to `implemented`.
- [x] A `TicketDetail` for a ticket in `"in_progress"` state with no `### Merge notes` section and an empty `recovery_options` array renders neither the "Merge failure" section nor the "Recovery" section.
- [x] Server integration test (git-based, default workflow): `GET /api/tickets` returns `merge_failure_state_ids` containing `"merge_failed"` and not containing `"in_progress"`.
- [x] Server integration test (InMemory): `GET /api/tickets/:id` for a ticket whose body contains `### Merge notes\n\ngit error text` returns `merge_notes: "git error text"`.
- [ ] Server integration test (git-based, default workflow): `GET /api/tickets/:id` for a ticket in `merge_failed` state returns `recovery_options` with at least one entry where `kind` is `"retry_merge"` and `command` matches `"apm state <id> implemented"`.

### Out of scope

- Action buttons or any new API endpoint that triggers a state transition; recovery happens exclusively via the `apm` CLI.
- Inline editing of `### Merge notes` or any other ticket body section in the UI.
- Dispatcher or `apm work` behavior changes around merge failure.
- Hardcoding `"merge_failed"` or any other state name in the server or frontend; all merge-failure classification flows through `classify_recovery_options`.
- Changes to the recovery classification logic itself (delivered by ae4104f2).
- CLI changes to `apm show`, `apm list`, or `apm next` (covered by ae4104f2).
- The `TicketResponse` (list endpoint per-ticket object) gaining a per-ticket `recovery_options` field; the badge is driven by the envelope-level `merge_failure_state_ids` to avoid computing classification for every ticket on every list call.

### Approach

#### `apm-server/src/models.rs`

Add `merge_failure_state_ids: Vec<String>` to `TicketsEnvelope`.

Add `recovery_options: Vec<RecoveryOptionDto>` and `merge_notes: Option<String>` to `TicketDetailResponse`.

Define a new serializable DTO:

```rust
#[derive(serde::Serialize)]
pub struct RecoveryOptionDto {
    pub to: String,
    pub label: String,
    pub kind: String,    // "retry_merge" | "return_to_worker" | "abandon" | "other"
    pub command: String, // "apm state {ticket_id} {to}"
}
```

`RecoveryOptionDto` is constructed by the handler from `apm_core::recovery::RecoveryOption`; no serde dependency is added to `apm-core` for this type.

#### `apm-server/src/handlers/tickets.rs` — `list_tickets`

Restructure the config loading block so the `Config` value is retained after the `(resolved_ids, terminal_ids, supervisor_states)` computation. Add a parallel computation:

```rust
let merge_failure_state_ids: Vec<String> = cfg.workflow.states.iter()
    .filter(|s| apm_core::recovery::is_merge_failure_state(&s.id, &cfg.workflow))
    .map(|s| s.id.clone())
    .collect();
```

Fall back to `vec![]` when there is no git root or the config fails to load. Include in `TicketsEnvelope { tickets, supervisor_states, merge_failure_state_ids }`.

#### `apm-server/src/handlers/tickets.rs` — `get_ticket`

Extend the first synchronous `Config::load` call (the one used to compute `blocking_deps`) to also build `recovery_options`. Gate the call on `is_merge_failure_state` so non-failure-state tickets always return an empty array:

```rust
let (blocking_deps, recovery_options) = match apm_core::config::Config::load(&root) {
    Ok(config) => {
        let deps = apm_core::compute_blocking_deps(ticket_ref, &tickets, &config);
        let opts = if apm_core::recovery::is_merge_failure_state(
            &ticket_ref.frontmatter.state, &config.workflow
        ) {
            apm_core::recovery::classify_recovery_options(
                &ticket_ref.frontmatter.state, &config.workflow,
            )
            .into_iter()
            .map(|opt| {
                let kind = match opt.kind {
                    RecoveryKind::RetryMerge      => "retry_merge",
                    RecoveryKind::ReturnToWorker  => "return_to_worker",
                    RecoveryKind::Abandon         => "abandon",
                    RecoveryKind::Other           => "other",
                }.to_string();
                RecoveryOptionDto {
                    command: format!("apm state {} {}", full_id, opt.to),
                    to: opt.to,
                    label: opt.label,
                    kind,
                }
            })
            .collect()
        } else {
            vec![]
        };
        (deps, opts)
    }
    Err(_) => (vec![], vec![]),
};
```

Extract `merge_notes` using the existing `extract_section` helper:

```rust
let merge_notes = {
    let s = extract_section(&ticket.body, "Merge notes").trim();
    if s.is_empty() { None } else { Some(s.to_string()) }
};
```

Add both fields to the `TicketDetailResponse { ..., recovery_options, merge_notes }` constructor.

#### apm-ui type definitions

`TicketDetail.tsx` — extend the inline `TicketDetail` interface:

```ts
recovery_options?: Array<{ to: string; label: string; kind: string; command: string }>
merge_notes?: string | null
```

`SupervisorView.tsx` — extend the `fetchTickets` return type to include `merge_failure_state_ids: string[]`. Extract from data: `const mergeFailureStateIds = data?.merge_failure_state_ids ?? []`. Pass to `<Swimlane mergeFailureStateIds={mergeFailureStateIds} ... />`.

#### apm-ui component changes

**`Swimlane.tsx`** — add `mergeFailureStateIds: string[]` to `SwimlaneProps`. Pass through to `<TicketCard mergeFailureStateIds={mergeFailureStateIds} ... />`.

**`TicketCard.tsx`** — add `mergeFailureStateIds: string[]` to `TicketCardProps`. Derive `const isMergeFailed = mergeFailureStateIds.includes(ticket.state)`. When true:
- Render a red badge pill (analogous to the existing amber `?` and `A` pills): `<span title="Merge failure" className="text-[10px] px-1 rounded bg-red-900/60 text-red-300">!</span>`
- Apply red border/background to the card container (analogous to the existing `isDepBlocked` amber treatment).

**`TicketDetail.tsx`** — after the `<div className="prose ...">` body section and before `<TransitionButtons>`, add two conditional blocks:

```tsx
{data.merge_notes && (
  <div className="px-6 py-4 border-t border-gray-700">
    <p className="text-[10px] font-semibold text-red-400 uppercase tracking-wide mb-2">Merge failure</p>
    <pre className="text-xs text-gray-300 bg-gray-800 rounded p-3 overflow-x-auto whitespace-pre-wrap break-words">
      {data.merge_notes}
    </pre>
  </div>
)}
{data.recovery_options && data.recovery_options.length > 0 && (
  <div className="px-6 py-4 border-t border-gray-700">
    <p className="text-[10px] font-semibold text-amber-400 uppercase tracking-wide mb-3">Recovery</p>
    {data.recovery_options.map(opt => (
      <div key={opt.to} className="mb-3">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm text-gray-200">{opt.label}</span>
          <span className="text-[10px] text-gray-500">{kindLabel(opt.kind)}</span>
        </div>
        <code className="block text-xs bg-gray-800 rounded px-3 py-2 font-mono text-green-300 select-all cursor-text">
          {opt.command}
        </code>
      </div>
    ))}
    <a href="/docs/merge-failed-recovery.md"
       className="text-[10px] text-blue-400 hover:underline mt-2 inline-block"
       target="_blank" rel="noreferrer">
      See: docs/merge-failed-recovery.md
    </a>
  </div>
)}
```

Define `kindLabel` as a module-level helper:
```ts
function kindLabel(kind: string): string {
  switch (kind) {
    case 'retry_merge':      return 'Retry merge'
    case 'return_to_worker': return 'Return to worker'
    case 'abandon':          return 'Abandon'
    default:                 return ''
  }
}
```

#### Server tests (`apm-server/src/main.rs`)

Three new tests in the `tests` module:

**`list_tickets_merge_failure_state_ids`** (git-based): Initialise a temp repo using the default workflow TOML from `apm-core::init` (copy the inline TOML string that includes `merge_failed` with transitions back to `implemented`). Call `GET /api/tickets`. Assert `json["merge_failure_state_ids"]` is an array that contains `"merge_failed"` and does not contain `"in_progress"`. Reuse the git setup pattern from `list_tickets_blocking_deps`.

**`get_ticket_merge_notes_extracted`** (InMemory): Create a ticket with `body = "### Merge notes\n\ngit error: conflict in foo.rs\n### Other\n\n"`. Call `GET /api/tickets/:id`. Assert `json["merge_notes"] == "git error: conflict in foo.rs"`. Also test the absent-section case: ticket with no `### Merge notes` section returns `merge_notes` as `null` or absent.

**`get_ticket_recovery_options_populated`** (git-based): Use the same git setup as the first test. Commit a ticket with `state = "merge_failed"`. Call `GET /api/tickets/:id`. Assert `json["recovery_options"]` is a non-empty array. Assert at least one entry satisfies: `kind == "retry_merge"` and `command` contains both `"apm state"` and `"implemented"`.

The default workflow TOML for these tests only needs the `implemented` state (with `completion = "Pr"` and `on_failure = "merge_failed"`) and the `merge_failed` state (with a transition `to = "implemented"`). Minimal subset — no need for the full workflow.

#### Frontend tests (`apm-ui`)

Add to `package.json` devDependencies: `@testing-library/react` and `@testing-library/jest-dom`. Add to `vite.config.ts`:
```ts
test: {
  environment: 'happy-dom',
}
```

**`apm-ui/src/components/supervisor/TicketCard.test.tsx`**:
- Before each test, mock `useLayoutStore` to return stable no-op defaults (`selectedTicketId: null`, `selectedTicketIds: []`, `lastClickedTicketId: null`, stubs for all setters).
- `shows_merge_failure_badge_when_state_in_list`: render `<TicketCard ticket={{...baseTicket, state: 'merge_failed'}} columnTicketIds={[]} mergeFailureStateIds={['merge_failed']} />`. Assert the `!` badge element is in the document.
- `no_badge_when_state_not_in_list`: render with `mergeFailureStateIds={[]}`. Assert the `!` badge is absent.
- `no_badge_for_in_progress`: render `<TicketCard ticket={{...baseTicket, state: 'in_progress'}} columnTicketIds={[]} mergeFailureStateIds={['merge_failed']} />`. Assert the `!` badge is absent.

**`apm-ui/src/components/TicketDetail.test.tsx`**:
- Wrap renders in a `QueryClientProvider` with a fresh `QueryClient`. Mock `global.fetch` to return the fixture ticket data.
- `shows_merge_failure_section`: fixture has `merge_notes: "fatal: merge conflict"`. Assert text "Merge failure" and "fatal: merge conflict" appear.
- `shows_recovery_section`: fixture has `recovery_options: [{to: 'implemented', label: 'Retry', kind: 'retry_merge', command: 'apm state abc12345 implemented'}]`. Assert "Recovery" heading and command text appear.
- `hides_sections_when_empty`: fixture has `merge_notes: null` and `recovery_options: []`. Assert neither "Merge failure" nor "Recovery" appears.
- `hides_sections_for_normal_state`: fixture has `state: 'in_progress'`, `merge_notes: null`, and `recovery_options: []`. Assert neither "Merge failure" nor "Recovery" appears.

### Open questions


### Amendment requests

- [x] PARALLEL AMENDMENT to ae4104f2: the spec computes merge_failure_state_ids using 'state's available transitions include at least one RetryMerge recovery option', but that proxy over-fires. RetryMerge labels a transition whose to-state is in merge_target_ids (the set of states reached by Pr/Merge/PrOrEpicMerge completions anywhere in the workflow) — and the normal in_progress -> implemented transition matches that. So under the spec as written, in_progress would land in merge_failure_state_ids, every in_progress ticket would render with the red badge in the SupervisorView, and the detail panel would show Recovery on tickets that are not stuck.

The fix lives in ae4104f2 (adds a new helper pub fn is_merge_failure_state(state_id: &str, workflow: &WorkflowConfig) -> bool that iterates every transition and returns true iff state_id matches transition.on_failure for any transition whose completion is Pr/Merge/PrOrEpicMerge). This ticket consumes that helper.

REQUIRED CHANGES:
1. SWITCH the server-side computation of merge_failure_state_ids in list_tickets to use apm_core::recovery::is_merge_failure_state instead of the current 'classify_recovery_options(...).any RetryMerge' check. New body:

   let merge_failure_state_ids: Vec<String> = cfg.workflow.states.iter()
       .filter(|s| apm_core::recovery::is_merge_failure_state(&s.id, &cfg.workflow))
       .map(|s| s.id.clone())
       .collect();

2. UPDATE the AC that defines merge_failure_state_ids to say: 'a JSON array of state ID strings for which apm_core::recovery::is_merge_failure_state returns true — i.e. states that appear as the on_failure value of at least one transition whose completion is Pr, Merge, or PrOrEpicMerge'.

3. ADD negative ACs:
   - merge_failure_state_ids under the default workflow contains exactly {merge_failed} — not in_progress, not implemented, not ready.
   - A TicketCard with state 'in_progress' does NOT render the merge-failure badge, even though in_progress has an outgoing transition to implemented.
   - A TicketDetail for an in_progress ticket with no Merge notes section and no failure-state recovery options renders neither the Merge failure section nor the Recovery section.

4. ADD negative tests:
   - Server: list_tickets_merge_failure_state_ids must assert merge_failure_state_ids contains 'merge_failed' AND does NOT contain 'in_progress'. Currently the AC only checks contains. Without the negative check the over-fire passes silently.
   - Frontend: add no_badge_for_in_progress to TicketCard.test.tsx — render a ticket with state 'in_progress' and a mergeFailureStateIds list containing only 'merge_failed'. Assert no badge.
   - Frontend: hides_sections_for_normal_state in TicketDetail.test.tsx — fixture with state 'in_progress', merge_notes null, recovery_options empty (since the server should not return options for non-failure states once is_merge_failure_state gates it). Assert neither section renders.

5. The recovery_options field on GET /api/tickets/:id should ALSO be gated on is_merge_failure_state(current state) — return an empty array when the ticket is not in a failure state. This prevents the detail panel from ever rendering the Recovery section on a normal-state ticket. Update the relevant AC accordingly: 'recovery_options is empty when is_merge_failure_state(ticket.state, workflow) returns false, or when no git root is present, or when config fails to load'.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:21Z | groomed | in_design | philippepascal |
| 2026-05-30T02:32Z | in_design | specd | claude |
| 2026-05-30T02:44Z | specd | ammend | philippepascal |
| 2026-05-30T03:33Z | ammend | in_design | philippepascal |
| 2026-05-30T03:40Z | in_design | specd | claude |
| 2026-05-30T03:59Z | specd | ready | philippepascal |
| 2026-05-30T04:22Z | ready | in_progress | philippepascal |