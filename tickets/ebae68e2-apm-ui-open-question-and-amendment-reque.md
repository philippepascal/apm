+++
id = "ebae68e2"
title = "apm-ui: open question and amendment request badges on ticket cards"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "11159"
branch = "ticket/ebae68e2-apm-ui-open-question-and-amendment-reque"
created_at = "2026-03-31T06:13:20.438546Z"
updated_at = "2026-03-31T07:27:12.986619Z"
+++

## Spec

### Problem

Ticket summary cards in the SupervisorView swimlanes show id, title, agent, effort, and risk, but give no signal about whether a ticket is waiting on supervisor input. Specifically: a ticket in *question* state may have written questions in `### Open questions` that need reading, and a ticket in *ammend* state has unchecked checkboxes in `### Amendment requests` that the spec-writer must address. Without glanceable badges, a supervisor must open every detail panel to know whether action is required.

The desired behaviour is: when a ticket has non-empty content in its `### Open questions` section, its card shows a small "?" badge; when a ticket has one or more unchecked items (`- [ ]`) in its `### Amendment requests` section, its card shows a small "A" badge. These badges let supervisors triage at a glance without opening the detail panel.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:13Z | — | new | apm |
| 2026-03-31T07:27Z | new | in_design | philippepascal |