+++
id = "7a76dd16"
title = "Surface epic freshness (behind-main status) in apm commands and UI"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7a76dd16-surface-epic-freshness-behind-main-statu"
created_at = "2026-05-29T01:17:24.701677Z"
updated_at = "2026-05-29T01:17:24.701677Z"
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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T01:17Z | — | new | philippepascal |
