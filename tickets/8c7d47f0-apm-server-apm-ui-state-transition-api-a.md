+++
id = "8c7d47f0"
title = "apm-server + apm-ui: state transition API and buttons"
state = "in_design"
priority = 0
effort = 4
risk = 0
author = "apm"
agent = "82538"
branch = "ticket/8c7d47f0-apm-server-apm-ui-state-transition-api-a"
created_at = "2026-03-31T06:12:47.638355Z"
updated_at = "2026-03-31T06:48:49.597253Z"
+++

## Spec

### Problem

The ticket detail panel (added in Step 6) is read-only: a supervisor looking at a ticket in the UI cannot change its state without switching to the CLI. This blocks the supervisor from completing their core workflow — reviewing specs, approving tickets, sending amendments — entirely from the browser.

**Current state (after Step 6):** The right column renders full ticket markdown and updates reactively when a ticket is selected. State is shown as a badge but there are no controls to change it.

**Desired state:**
- A new `POST /api/tickets/:id/transition` endpoint accepts `{"to":"<state>"}` and executes the apm-core state machine, including all guards (spec validation, criteria checks, valid-transition enforcement).
- The ticket detail panel grows a row of action buttons — one per valid transition from the current state — derived from the workflow config. Each button label comes from the `label` field in the transition config (or `→ {to}` as fallback).
- A "Keep at {state}" button is always present as a no-op affordance, matching the CLI `apm review` menu.
- Transition errors (invalid transition, precondition failure) surface inline near the buttons.
- After a successful transition the panel refreshes with the new state and new available transitions automatically.

**Who is affected:** Supervisors using the web UI to review and progress tickets.

### Acceptance criteria

- [ ] POST /api/tickets/:id/transition with a valid body transitions the ticket state and returns 200 with the updated ticket JSON
- [ ] POST /api/tickets/:id/transition returns 422 with a JSON `{"error":"..."}` body when the transition is invalid (not defined in the state machine from the current state)
- [ ] POST /api/tickets/:id/transition returns 422 with a JSON error when a precondition fails (e.g. transitioning to `specd` with missing spec sections, or to `implemented` with unchecked criteria)
- [ ] POST /api/tickets/:id/transition returns 404 when the ticket id does not exist
- [ ] GET /api/tickets/:id response includes a `valid_transitions` array, each entry having `to` (state id) and `label` (transition label from config, or `-> {to}` if blank)
- [ ] The ticket detail panel renders one button per entry in `valid_transitions` using the entry's `label` as the button text
- [ ] A "Keep at {state}" button is always visible in the detail panel and performs no API call when clicked
- [ ] Clicking a transition button fires POST /api/tickets/:id/transition and disables all transition buttons while the request is in-flight
- [ ] On a successful transition, the detail panel and swimlanes update to reflect the new state without a full page reload (TanStack Query cache invalidation)
- [ ] On a failed transition, an inline error message appears near the buttons showing the text from the API error response; the buttons re-enable
- [ ] The transition buttons are not shown when no ticket is selected
- [ ] npm run build in apm-ui/ exits 0 with no TypeScript errors
- [ ] cargo test --workspace passes

### Out of scope

- The markdown editor with RO/RW sections — covered by Step 9 (ticket a6c115e1)
- Editing any ticket content (body, spec sections) — Step 9
- The "review" button opening a full editor screen — Step 9
- Priority reordering in the worker queue — Step 11
- Keyboard shortcut for state transitions (Step 8 scope is buttons only; shortcuts are a general feature)
- Validation that the acting user has permission for the transition (no auth layer exists yet)
- Optimistic UI updates — transitions wait for the server round-trip before refreshing

### Approach

**Prerequisites:** Step 6 (ticket 268f5694) must be implemented — TicketDetail.tsx renders ticket markdown, TanStack Query is wired up, AppState has `root`.

---

**1. Extend TicketResponse (apm-server/src/routes/tickets.rs)**

Add a `valid_transitions` field to the response type returned by `GET /api/tickets/:id`:

```rust
#[derive(serde::Serialize)]
struct TransitionOption {
    to: String,
    label: String,
}

#[derive(serde::Serialize)]
struct TicketResponse<'a> {
    #[serde(flatten)]
    frontmatter: &'a Frontmatter,
    body: &'a str,
    valid_transitions: Vec<TransitionOption>,
}
```

Compute `valid_transitions` from the loaded config:
```rust
let current_state = &ticket.frontmatter.state;
let valid_transitions = config.workflow.states.iter()
    .find(|s| &s.id == current_state)
    .map(|s| s.transitions.iter().map(|tr| TransitionOption {
        to: tr.to.clone(),
        label: if tr.label.is_empty() {
            format!("-> {}", tr.to)
        } else {
            tr.label.clone()
        },
    }).collect())
    .unwrap_or_default();
```

Pass `config` into `AppState` (or re-load it per request — it's cheap).

---

**2. Add POST /api/tickets/:id/transition (apm-server/src/routes/tickets.rs)**

Request body:
```rust
#[derive(serde::Deserialize)]
struct TransitionRequest { to: String }
```

Handler:
1. Extract `:id` and deserialise the JSON body (return 400 on parse error)
2. `tokio::task::spawn_blocking(move || apm_core::state::transition(&root, &id, to_state, /*no_aggressive=*/false, /*force=*/false))`
3. On `Ok(output)`: re-load the ticket via `ticket::load_all_from_git` and return the updated `TicketResponse` as 200 JSON
4. On `Err(e)`: return 422 with `Json(serde_json::json!({"error": e.to_string()}))`
5. If the ticket is not found after transition (shouldn't happen), return 404

Register the route in main.rs:
```rust
.route("/api/tickets/:id/transition", post(transition_ticket))
```

**Error mapping note:** `apm_core::state::transition` returns descriptive `anyhow::Error` messages for invalid transitions and precondition failures. Returning them verbatim as the error body is correct — the messages are user-readable.

---

**3. Update AppState to carry Config (apm-server/src/main.rs)**

```rust
struct AppState {
    root: PathBuf,
    config: apm_core::config::Config,
}
```

Load `Config` once at server startup. Pass `Arc<AppState>` to all handlers (already the pattern from Step 2).

---

**4. Transition buttons component (apm-ui/src/components/TicketDetail.tsx)**

Add below the markdown content:

```tsx
function TransitionButtons({ ticket, onTransitioned }: {
  ticket: TicketDetail,
  onTransitioned: () => void
}) {
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function doTransition(to: string) {
    setPending(true)
    setError(null)
    try {
      const res = await fetch(`/api/tickets/${ticket.id}/transition`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ to }),
      })
      if (!res.ok) {
        const body = await res.json()
        setError(body.error ?? `Error ${res.status}`)
      } else {
        onTransitioned()
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="border-t p-3 flex flex-wrap gap-2 items-center">
      {ticket.valid_transitions.map(tr => (
        <Button key={tr.to} size="sm" disabled={pending}
          onClick={() => doTransition(tr.to)}>
          {tr.label}
        </Button>
      ))}
      <Button key="keep" size="sm" variant="ghost" disabled={pending}>
        Keep at {ticket.state}
      </Button>
      {error && <p className="text-destructive text-sm w-full">{error}</p>}
    </div>
  )
}
```

In `TicketDetail.tsx`, after the markdown section, render `<TransitionButtons>` when a ticket is loaded. Pass an `onTransitioned` callback that calls:
```ts
queryClient.invalidateQueries({ queryKey: ['ticket', ticket.id] })
queryClient.invalidateQueries({ queryKey: ['tickets'] })
```

---

**5. TypeScript type update (apm-ui/src/types.ts or inline)**

Extend the `Ticket` type to include:
```ts
valid_transitions: { to: string; label: string }[]
```

---

**6. File changes summary**

Backend:
- `apm-server/src/routes/tickets.rs` — add `valid_transitions` to `TicketResponse`, add `TransitionRequest` + `transition_ticket` handler
- `apm-server/src/main.rs` — add `Config` to `AppState`, register new POST route

Frontend:
- `apm-ui/src/components/TicketDetail.tsx` — add `TransitionButtons` component, invalidate queries on success
- `apm-ui/src/types.ts` (or wherever the Ticket type lives) — add `valid_transitions` field

---

**7. Ordering constraint**

`apm_core::state::transition` is synchronous and does git I/O. Always call it inside `tokio::task::spawn_blocking` to avoid blocking the async runtime. This mirrors the pattern already established for `ticket::load_all_from_git` in Step 2.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:42Z | new | in_design | philippepascal |