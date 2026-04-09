+++
id = "8e46686e"
title = "apm-ui: review-screen transition keyboard shortcut algorithm"
state = "closed"
priority = 30
effort = 3
risk = 1
author = "claude-0331-1200-a7b9"
agent = "65623"
branch = "ticket/8e46686e-apm-ui-review-screen-transition-keyboard"
created_at = "2026-03-31T18:19:43.047043Z"
updated_at = "2026-04-01T07:12:49.068414Z"
+++

## Spec

### Problem

In the review/editor screen (ticket a6c115e1), state transition buttons are shown to the user. The keyboard spec requires each button to have a computed keyboard shortcut derived from the target state name, with conflict avoidance. The "Keep at {state}" button is always present and uses the reserved key `K`. No ticket currently defines this algorithm, implements it, or tests it. Without a spec, different agents will invent incompatible approaches.

The algorithm must: (1) assign a single lowercase letter to each transition button based on the target state name, (2) avoid conflicts between letters assigned to different transitions on the same screen, (3) reserve `K` for the "Keep" action, (4) display the assigned letter visibly on the button (e.g. underlined or bracketed), (5) register a keydown handler in the editor component that fires the corresponding transition when the assigned letter is pressed.

Affected: anyone using the review screen keyboard shortcuts to progress tickets.

### Acceptance criteria

- [x] Each transition button in the review panel has a computed keyboard shortcut letter displayed on the button
- [x] The letter is derived from the target state name: take the first letter; if that letter is already taken by another transition on the same screen, take the second letter; continue until a free letter is found
- [x] `K` is always reserved for "Keep at {state}" and is never assigned to a transition button
- [x] If all letters in the state name are taken, fall back to the first available letter of the alphabet not yet assigned
- [x] Pressing the assigned letter while the editor is focused fires the corresponding transition
- [x] Pressing `K` while the editor is focused fires the "Keep" action (no transition, closes editor)
- [x] The shortcut letter is displayed on the button in a visually distinct way (e.g. underlined character or `[X]` prefix)
- [x] The keydown handler does not fire when an inner input/textarea/contenteditable has focus
- [x] The algorithm is a pure function tested independently: given a list of transition target state names, it returns a map of state → letter with no conflicts and K excluded

### Out of scope

- Keyboard shortcuts outside the review/editor screen — global shortcuts are defined elsewhere
- The state transition API itself (covered by 8c7d47f0)
- The review screen layout and save mechanism (covered by a6c115e1)
- Shortcuts for transitions in the ticket detail panel (detail panel shows buttons only, no keyboard shortcuts)

### Approach

Prerequisites: 8c7d47f0 (transition buttons and valid_transitions API) and a6c115e1 (review screen) must be implemented.

**1. Algorithm — pure function in `apm-ui/src/lib/transitionShortcuts.ts` (new file)**

```ts
const RESERVED = new Set(['k']);

export function assignShortcuts(transitionTargets: string[]): Map<string, string> {
  const result = new Map<string, string>();
  const used = new Set<string>(RESERVED);

  for (const target of transitionTargets) {
    let assigned: string | null = null;
    // Try letters of the state name first
    for (const ch of target.toLowerCase().replace(/[^a-z]/g, '')) {
      if (!used.has(ch)) { assigned = ch; break; }
    }
    // Fall back to alphabet scan
    if (!assigned) {
      for (let i = 0; i < 26; i++) {
        const ch = String.fromCharCode(97 + i);
        if (!used.has(ch)) { assigned = ch; break; }
      }
    }
    if (assigned) {
      result.set(target, assigned);
      used.add(assigned);
    }
  }
  return result;
}
```

**2. Button rendering — in the TransitionButtons component (from 8c7d47f0)**

Call `assignShortcuts(ticket.valid_transitions.map(t => t.to))` to get the shortcut map. For each transition button, find the assigned letter and render the button label with that character visually highlighted (underline via CSS or bracket notation).

Example: if transition is `→ ready` and assigned letter is `r`, render the button label as `→ **r**eady` (underline the `r`).

**3. Keydown handler — in the ReviewEditor component (from a6c115e1)**

Add a CodeMirror keymap extension (or a React useEffect) that listens for letter keys while the editor is mounted:
- On `k`: fire the "Keep" action (close editor without transitioning)
- On any assigned letter: find the matching transition and fire it (same call as clicking the button: POST /api/tickets/:id/transition)
- Guard: do not fire if a nested input/textarea/contenteditable has focus

**4. Tests**

Unit test `assignShortcuts` in `apm-ui/src/lib/transitionShortcuts.test.ts`:
- Basic case: `['ready', 'blocked']` → `{ready: 'r', blocked: 'b'}`
- Conflict: `['ready', 'review']` → `{ready: 'r', review: 'e'}` (second letter of 'review')
- Reserved K: `['kept', 'closed']` → `{kept: 'e', closed: 'c'}` (k skipped for 'kept')
- Overflow: alphabet exhaustion falls back correctly

**File changes:**
- `apm-ui/src/lib/transitionShortcuts.ts` — new file (algorithm)
- `apm-ui/src/lib/transitionShortcuts.test.ts` — new file (unit tests)
- `apm-ui/src/components/TicketDetail.tsx` or `ReviewEditor.tsx` — wire algorithm + keydown handler into transition buttons

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:19Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T18:19Z | new | in_design | claude-0331-1200-a7b9 |
| 2026-03-31T18:21Z | in_design | specd | claude-0331-1200-a7b9 |
| 2026-03-31T19:45Z | specd | ready | apm |
| 2026-04-01T06:39Z | ready | in_progress | philippepascal |
| 2026-04-01T06:43Z | in_progress | implemented | claude-0401-0639-2dc0 |
| 2026-04-01T07:02Z | implemented | accepted | apm-sync |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |