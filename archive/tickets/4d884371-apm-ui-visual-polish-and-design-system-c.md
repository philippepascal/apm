+++
id = "4d884371"
title = "apm-ui: visual polish and design system consistency"
state = "closed"
priority = 0
effort = 5
risk = 2
author = "apm"
agent = "8711"
branch = "ticket/4d884371-apm-ui-visual-polish-and-design-system-c"
created_at = "2026-04-01T05:51:09.997793Z"
updated_at = "2026-04-01T07:12:39.925137Z"
+++

## Spec

### Problem

The current apm-ui has a functional but visually raw interface. All panels use the same text scale (mostly text-xs/text-sm), state badges are uniformly gray, panel differentiation relies entirely on borders, and the worker/queue panels use dense tables at widths too narrow to read comfortably. The toggle buttons in the toolbar feel like debug controls. There is no consistent visual language or accent color application. The result is a UI that works but does not communicate hierarchy or state clearly at a glance.

### Acceptance criteria

- [x] State badges use distinct colors by category: blocked/question = red/amber, in_design/in_progress = blue, specd/ready = purple, implemented/accepted = green, closed = gray
- [x] Swimlane column headers have a colored left or top border accent matching their state category color
- [x] Ticket cards have a subtle shadow and clearer title/metadata hierarchy (title prominent, ID and agent de-emphasized)
- [x] The left column has a darker background than the center and right columns, creating visual depth
- [x] Worker activity panel uses a card-per-worker layout instead of a table, with a colored status dot and elapsed time
- [x] Queue rows have sufficient padding and visual weight to be scannable at the panel's narrow width
- [x] The toolbar toggle buttons are replaced with icon-only toggle buttons or collapse arrows on panel edges
- [x] A single accent color (blue) is applied consistently: primary action buttons, selected card ring, focus rings
- [x] The ticket detail panel has a large title header, prominent state badge, and a metadata row before the markdown body
- [x] The markdown prose content in the detail panel renders at a comfortable max-width with adequate line height
- [x] npm run build exits 0 with no TypeScript errors

### Out of scope

- Layout structure changes (column count, resizable panels) — those are functional changes
- New features or data displayed in panels
- Dark mode
- Animations or transitions beyond what shadcn/ui provides by default
- Mobile/responsive layout

### Approach

**Design reference:** Linear is the closest analogue — dark left sidebar, swimlane-style board, card-based tickets, colored state indicators.

**1. Color tokens for state categories**
Define a mapping in a shared file (e.g. lib/stateColors.ts):
- blocked, question → red/amber
- in_design, in_progress → blue
- specd, ready → purple/indigo
- implemented, accepted → green
- closed → gray

Use these in swimlane headers, state badges, and the detail panel header badge.

**2. Swimlane columns**
- Add a colored left border (4px) to each swimlane header using the state category color
- Increase card padding slightly; give cards a box-shadow-sm
- Title at text-sm font-medium; ID at text-[10px] mono muted; agent at text-[10px] muted below title

**3. Left column**
- Set WorkerView background to bg-gray-900 text-gray-100 (dark sidebar)
- Replace worker table with worker cards: status dot (green/red) + ticket ID + agent + elapsed in a compact card
- Queue rows: increase py to py-1.5, ensure rank column is narrow and muted

**4. Toolbar**
- Replace 'Hide X / Show X' text buttons with icon toggle buttons (ChevronLeft/ChevronRight or Columns icon from lucide) placed at panel edges or in a thin top bar
- Keep keyboard shortcuts working

**5. Detail panel header**
- Add a header section above the markdown: large title (text-base font-semibold), state badge with color, a row of metadata (agent, effort E:N, risk R:N)
- Markdown content: add px-6 py-4 prose-base or prose-sm with max-w-2xl mx-auto for comfortable line length

**6. Accent color consistency**
- Primary buttons: bg-blue-600 (already partially done)
- Selected card: ring-2 ring-blue-500 (already done)
- Focus rings: focus:ring-blue-500 (audit all inputs)

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-01T05:51Z | — | new | apm |
| 2026-04-01T05:51Z | new | in_design | apm |
| 2026-04-01T05:52Z | in_design | specd | apm |
| 2026-04-01T05:58Z | specd | ready | apm |
| 2026-04-01T06:23Z | ready | in_progress | philippepascal |
| 2026-04-01T06:33Z | in_progress | implemented | claude-0401-0623-07b8 |
| 2026-04-01T06:36Z | implemented | accepted | apm |
| 2026-04-01T07:12Z | accepted | closed | apm-sync |