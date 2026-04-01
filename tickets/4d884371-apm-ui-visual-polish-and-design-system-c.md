+++
id = "4d884371"
title = "apm-ui: visual polish and design system consistency"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/4d884371-apm-ui-visual-polish-and-design-system-c"
created_at = "2026-04-01T05:51:09.997793Z"
updated_at = "2026-04-01T05:51:17.058963Z"
+++

## Spec

### Problem

The current apm-ui has a functional but visually raw interface. All panels use the same text scale (mostly text-xs/text-sm), state badges are uniformly gray, panel differentiation relies entirely on borders, and the worker/queue panels use dense tables at widths too narrow to read comfortably. The toggle buttons in the toolbar feel like debug controls. There is no consistent visual language or accent color application. The result is a UI that works but does not communicate hierarchy or state clearly at a glance.

### Acceptance criteria

- [ ] State badges use distinct colors by category: blocked/question = red/amber, in_design/in_progress = blue, specd/ready = purple, implemented/accepted = green, closed = gray
- [ ] Swimlane column headers have a colored left or top border accent matching their state category color
- [ ] Ticket cards have a subtle shadow and clearer title/metadata hierarchy (title prominent, ID and agent de-emphasized)
- [ ] The left column has a darker background than the center and right columns, creating visual depth
- [ ] Worker activity panel uses a card-per-worker layout instead of a table, with a colored status dot and elapsed time
- [ ] Queue rows have sufficient padding and visual weight to be scannable at the panel's narrow width
- [ ] The toolbar toggle buttons are replaced with icon-only toggle buttons or collapse arrows on panel edges
- [ ] A single accent color (blue) is applied consistently: primary action buttons, selected card ring, focus rings
- [ ] The ticket detail panel has a large title header, prominent state badge, and a metadata row before the markdown body
- [ ] The markdown prose content in the detail panel renders at a comfortable max-width with adequate line height
- [ ] npm run build exits 0 with no TypeScript errors

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T05:51Z | — | new | apm |
| 2026-04-01T05:51Z | new | in_design | apm |