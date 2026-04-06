# UI Spec Review
_Generated 2026-03-31 — covers all 20 specd UI tickets and keyboard shortcut consistency_

---

## Keyboard shortcut analysis

Your edits to the shortcut table (vs. my earlier draft):

| Change | Verdict |
|--------|---------|
| Removed global `1`–`9` for effort, `Shift+1`–`Shift+9` for risk | Correct — letter picker was already the right call |
| But also removed `e` / `Shift+R` pickers from the global table | Fine — effort/risk are now handled purely by inline click-to-edit in the detail panel (95ef3505). No global shortcut needed. |
| Removed `]` / `[` state transition shortcuts | Good — transitions are now review-screen only with config-driven letter keys |
| State transitions in review screen: algorithmically derived from state name | Good design, but no ticket currently specifies this algorithm. Flagged below (#6). |
| Column visibility: toolbar-only | Correct — `Ctrl+1/2/3` conflicts confirmed. |
| Editor shortcuts: only `Escape` survives | `Ctrl+S` and `Ctrl+Enter` removed — creates inconsistency in a6c115e1. Flagged below. |
| `Shift+W` for work engine | Ticket 56499b61 says `Ctrl+Shift+W`. Conflict. Flagged below. |
| `Shift+D` for dry-run toggle | Ticket fabfef3d never mentions this shortcut. Gap. Flagged below. |
| `Shift+K` for stop worker | Ticket 6d46e15c never mentions this shortcut. Gap. Flagged below. |
| `Shift+S` for sync | Ticket 15b7b28e says `s` (lowercase). Conflict. Flagged below. |

---

## Issues requiring amendments

### 1. **e1748434** — 3-column layout shell
**Severity: AC amendment required**

Acceptance criterion: _"Pressing Ctrl+1, Ctrl+2, Ctrl+3 moves keyboard focus to WorkerView, SupervisorView, and TicketDetail respectively"_ conflicts with the updated keyboard spec (toolbar-only, no keyboard shortcut for column visibility).

The `Ctrl+1/2/3` keyboard shortcut must be removed from the AC and the corresponding `useEffect` from the Approach. The toggle controls in each panel header (eye icon / button) remain; they just have no keyboard binding.

---

### 2. **56499b61** — work engine start/stop
**Severity: AC amendment required**

The Acceptance Criterion says _"A keyboard shortcut (`Ctrl+Shift+W`) toggles the engine start/stop"_. The keyboard spec says `Shift+W`. Pick one and make both consistent. Recommendation: use `Shift+W` (simpler, less likely to conflict).

Also: the approach uses `std::process::Command::new("apm")` to spawn the daemon. This requires the `apm` binary to be on `PATH` relative to wherever the server process runs. The approach should note: the server must be started from the repo root (same constraint as `ServeDir` in Step 3), and `apm` must be findable via `PATH`. Alternatively, use the full path from `std::env::current_exe()` or a configured binary path.

Ph: apm-serve must not call commandlines. it must be self contained.

---

### 3. **a6c115e1** — markdown editor (CodeMirror 6)
**Severity: AC amendment required**

Acceptance criterion: _"Cmd+S / Ctrl+S inside the editor triggers the save action"_. The keyboard spec no longer lists `Ctrl+S` as an editor shortcut — `Escape` is the only listed key. This line in the AC needs to be removed.

The save action is triggered by the Save button (and implicitly by state-transition shortcuts defined algorithmically per the review screen spec). No `Ctrl+S` binding should be added.

Ph: once in review, the appropriate state transition buttons are shown, with their own keybindings. Keep state included. which should probably be triggered by 'K'

---

### 4. **15b7b28e** — sync button
**Severity: AC / approach amendment required**

The approach says the keyboard shortcut is lowercase `s`. The keyboard spec says `Shift+S`. These must match. If `s` (lowercase) is preferred, the keyboard spec table should be updated; if `Shift+S` is preferred, the ticket approach must change. Recommendation: `Shift+S` — lowercase `s` would fire unexpectedly while typing text in other fields even with the focus guard, since some focus paths (e.g. closing a modal) can return focus to an unexpected element.

---

### 5. **4ce2a53e** — ticket search and filter
**Severity: approach bug**

The approach references `['closed', 'cancelled']` for the show-closed toggle. There is no `cancelled` state in the workflow config. The only terminal state is `closed`. Remove `'cancelled'` from the extra-states list; the show-closed toggle should append `['closed']` only.

---

### 6. **Missing ticket: review-screen transition algorithm**
**Severity: gap — no ticket covers this**

The keyboard spec says: _"Shortcut letter is calculated using an algorithm based on state name and avoid conflict."_ No ticket defines what this algorithm is, implements it, or tests it. The tickets that currently deliver the review screen (a6c115e1, 8c7d47f0) are silent on keyboard shortcuts for transitions.

This needs either: (a) a new ticket, or (b) an amendment to a6c115e1 or 8c7d47f0 that specifies the algorithm (e.g., take the first letter of `to` state name; if conflict, take the second; display the shortcut letter on the button).

Ph: new ticket

---

### 7. **fabfef3d** — dry-run preview
**Severity: AC gap**

The keyboard spec lists `Shift+D` as "Toggle dry-run preview" but the ticket's AC has no keyboard shortcut at all. If `Shift+D` is intended, it needs an AC entry and a note about the global keydown handler (same pattern as `n` in 51038e03). If dry-run has no keyboard shortcut, remove `Shift+D` from the keyboard spec.

Ph: I don't think dry run means much in the UI: the users sees the queue, so they know what tickets get picked up next.

---

### 8. **6d46e15c** — worker management
**Severity: AC gap + assumption risk**

- The keyboard spec has `Shift+K` for "Stop selected worker" but the ticket's AC mentions only a "Stop" button in the table row — no keyboard shortcut. Add the shortcut to the AC or remove it from the keyboard spec.
- The approach assumes `apm_core::start::resolve_agent_name()`, `apm_core::git::list_ticket_worktrees()`, and `ticket::handoff()` by those exact names. These may not exist in apm-core under those names. The implementing agent must verify or define them. Consider adding a note to flag this.

---

## Observations (no amendment required, but worth knowing)

### **7777cf5c** — priority queue & **fabfef3d** — dry-run
Both call `config.actionable_states_for("agent")` — a helper method that doesn't yet exist on the Config struct. It needs to be implemented in apm-core (scan `[[workflow.states]]` for entries with `actionable = ["agent"]`). This isn't a spec problem, but the implementing agent for Step 1 of 7777cf5c should add it to apm-core.

### **8c7d47f0** — state transitions
`valid_transitions` returns all transitions regardless of actor. Supervisors will see buttons for "agent-only" transitions (e.g. `in_progress → implemented`). The spec correctly calls this out of scope (no auth layer). Fine as-is, but the UX will be awkward until an auth layer is added.

### **51038e03** — new ticket form
References `ticket::create(... aggressive: bool ...)` — the `aggressive` parameter was added during the push-on-close refactor. The implementing agent should verify the current signature before writing the handler.

### **7f61c54a** — drag-and-drop
Typo in file changes: `apm-serve/src/routes/tickets.rs` should be `apm-server/src/routes/tickets.rs`.

### **54eb5bfc** — ticket list/detail API
`ticket::normalize_id_arg` may not exist by that name. The actual apm-core resolution logic uses `resolve_id_in_slice` or prefix-matching inline. The implementing agent should check the current apm-core API.

### **e9ba2503** — log tail viewer
Log file path is read from `config.logging.file` — currently `/tmp/apm.log`. Fine for dev. For production deployments the path will differ. No change needed now, just be aware.

### **3b0019a3** — supervisor swimlanes
Hard-codes `SUPERVISOR_STATES = ['question', 'specd', 'blocked', 'implemented', 'accepted']`. The `ammend` state (note: apm spells it with double-m) is actionable by agent/supervisor but not in this list. If a ticket enters `ammend` state, it won't appear in any swimlane. Consider adding `ammend` to the supervisor states list.

---

## Summary table

| Ticket | Title (short) | Issue | Action |
|--------|--------------|-------|--------|
| e1748434 | 3-column layout | `Ctrl+1/2/3` in AC contradicts keyboard spec | Amend — remove from AC |
| 56499b61 | Work engine start/stop | `Ctrl+Shift+W` vs `Shift+W` | Amend — align keyboard shortcut |
| a6c115e1 | Markdown editor | `Cmd+S/Ctrl+S` in AC, removed from keyboard spec | Amend — remove from AC |
| 15b7b28e | Sync button | `s` vs `Shift+S` | Amend — align keyboard shortcut |
| 4ce2a53e | Search & filter | `cancelled` state doesn't exist | Amend — remove `cancelled` |
| *(none)* | Review-screen transitions | Algorithm for shortcut letters unspecified | New ticket or amendment |
| fabfef3d | Dry-run preview | `Shift+D` in keyboard spec, not in AC | Add to AC or remove from spec |
| 6d46e15c | Worker management | `Shift+K` in keyboard spec, not in AC | Add to AC or remove from spec |
| 3b0019a3 | Supervisor swimlanes | `ammend` state not in supervisor swimlane | Consider adding |
| 7f61c54a | Drag-and-drop | `apm-serve` typo | Fix typo |

**Tickets with no issues: 36ea9bdb, 54eb5bfc, ed5c2b3b, 268f5694, 651f8a63, 7777cf5c, 8c7d47f0, 51038e03, 95ef3505, 7f61c54a (minor), 56499b61 (aside from shortcut), fabfef3d (aside from shortcut), e9ba2503, ebae68e2.**
