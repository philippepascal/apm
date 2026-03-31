+++
id = "8e46686e"
title = "apm-ui: review-screen transition keyboard shortcut algorithm"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0331-1200-a7b9"
branch = "ticket/8e46686e-apm-ui-review-screen-transition-keyboard"
created_at = "2026-03-31T18:19:43.047043Z"
updated_at = "2026-03-31T18:19:52.398096Z"
+++

## Spec

### Problem

In the review/editor screen (ticket a6c115e1), state transition buttons are shown to the user. The keyboard spec requires each button to have a computed keyboard shortcut derived from the target state name, with conflict avoidance. The "Keep at {state}" button is always present and uses the reserved key `K`. No ticket currently defines this algorithm, implements it, or tests it. Without a spec, different agents will invent incompatible approaches.

The algorithm must: (1) assign a single lowercase letter to each transition button based on the target state name, (2) avoid conflicts between letters assigned to different transitions on the same screen, (3) reserve `K` for the "Keep" action, (4) display the assigned letter visibly on the button (e.g. underlined or bracketed), (5) register a keydown handler in the editor component that fires the corresponding transition when the assigned letter is pressed.

Affected: anyone using the review screen keyboard shortcuts to progress tickets.

### Acceptance criteria

- [ ] Each transition button in the review panel has a computed keyboard shortcut letter displayed on the button
- [ ] The letter is derived from the target state name: take the first letter; if that letter is already taken by another transition on the same screen, take the second letter; continue until a free letter is found
- [ ] `K` is always reserved for "Keep at {state}" and is never assigned to a transition button
- [ ] If all letters in the state name are taken, fall back to the first available letter of the alphabet not yet assigned
- [ ] Pressing the assigned letter while the editor is focused fires the corresponding transition
- [ ] Pressing `K` while the editor is focused fires the "Keep" action (no transition, closes editor)
- [ ] The shortcut letter is displayed on the button in a visually distinct way (e.g. underlined character or `[X]` prefix)
- [ ] The keydown handler does not fire when an inner input/textarea/contenteditable has focus
- [ ] The algorithm is a pure function tested independently: given a list of transition target state names, it returns a map of state → letter with no conflicts and K excluded

### Out of scope

- Keyboard shortcuts outside the review/editor screen — global shortcuts are defined elsewhere
- The state transition API itself (covered by 8c7d47f0)
- The review screen layout and save mechanism (covered by a6c115e1)
- Shortcuts for transitions in the ticket detail panel (detail panel shows buttons only, no keyboard shortcuts)

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T18:19Z | — | new | claude-0331-1200-a7b9 |
| 2026-03-31T18:19Z | new | in_design | claude-0331-1200-a7b9 |