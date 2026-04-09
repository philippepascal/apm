+++
id = "059d0a50"
title = "modify apm-demo script"
state = "in_progress"
priority = 0
effort = 5
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/059d0a50-modify-apm-demo-script"
created_at = "2026-04-08T23:59:41.422002Z"
updated_at = "2026-04-09T00:33:38.446311Z"
+++

## Spec

### Problem

The `scripts/create-demo.sh` script builds a demo APM repository around "jot," a minimal Rust CLI notes tool. It currently creates 14 tickets covering all 11 workflow states, with one epic ("Search feature") containing 3 tickets.

The demo needs to better showcase APM's epic management, dependency graphs, and the full implemented→closed lifecycle. Specifically: there is only one epic, the ticket count is modest for a realistic project, and the `implemented` state appears only once (T3 — list notes). Users exploring the demo get an incomplete picture of a healthy project backlog.

The goal is to extend the script to add a second epic (7 tickets with intra-epic dependencies), double the count of all non-new-epic tickets from 14 to 28 (by adding 14 more standalone tickets), and ensure multiple tickets reach `implemented` state. All new content must remain coherent with the jot project.

### Acceptance criteria

- [x] The script creates exactly 2 epics
- [x] The new epic contains exactly 7 tickets (assigned via `--epic`)
- [x] The new epic tickets include at least 3 intra-epic dependency edges (via `--depends-on`)
- [x] The total ticket count after the script runs is 35 (28 non-new-epic + 7 new-epic)
- [x] At least 4 tickets across the whole demo are in `implemented` state
- [x] Every new ticket title describes a plausible jot feature or fix
- [x] Every ticket in `specd`, `implemented`, or `closed` state has all four spec sections populated (Problem, Acceptance criteria, Out of scope, Approach)
- [x] The script runs end-to-end without errors on a clean GitHub account
- [x] The README is updated to reflect 35 tickets and 2 epics

### Out of scope

- Changes to the jot Rust source code (`src/main.rs`, `Cargo.toml`)
- Adding a third epic or modifying the existing "Search feature" epic
- Changing the set of workflow states or APM config
- Modifying the preflight checks or GitHub repo creation logic
- Changing how the existing 14 tickets are structured or ordered

### Approach

**File to change:** `scripts/create-demo.sh` only.

---

### Step 1 — Create the second epic: "Multi-notebook support"

Insert immediately after the existing `apm epic new 'Search feature'` block (around line 237). Use the same pattern:

```bash
EPIC2_ID=$(apm epic new --no-aggressive 'Multi-notebook support' | extract_id)
echo "    EPIC2=$EPIC2_ID"
```

---

### Step 2 — Add 7 tickets to the new epic

Add these after the existing 14-ticket block (but before the README section). Ticket variable names TE1–TE7. Dependency edges: TE3→TE1, TE3→TE2, TE4→TE1, TE4→TE2, TE5→TE1, TE5→TE2, TE6→TE3, TE7→TE6 (≥3 intra-epic edges met).

| Var | Title | State | Depends on | Notes |
|-----|-------|-------|-----------|-------|
| TE1 | Create a named notebook | `closed` | — | Full spec + code review; foundational |
| TE2 | Switch active notebook | `closed` | — | Full spec + code review |
| TE3 | List all notebooks | `implemented` | TE1, TE2 | Full spec, all AC checked |
| TE4 | Delete a notebook | `ready` | TE1, TE2 | Full spec, AC unchecked |
| TE5 | Rename a notebook | `specd` | TE1, TE2 | Full spec, AC unchecked |
| TE6 | Move note between notebooks | `in_design` | TE3 | Partial spec, in_design state |
| TE7 | Merge two notebooks | `new` | TE6 | No spec; stays new |

Spec content for each ticket — write concise Problem/AC/Out of scope/Approach consistent with jot's Rust CLI style (see existing ticket specs for tone). TE1 and TE2 get Code review sections with checked boxes.

---

### Step 3 — Add 14 additional standalone tickets

Add after the epic tickets block. Variable names TS1–TS14. These bring total non-new-epic tickets from 14 to 28.

| Var | Title | State | Depends on |
|-----|-------|-------|-----------|
| TS1 | Add --version flag | `closed` | — |
| TS2 | Colorize list output | `implemented` | T3 |
| TS3 | Record timestamp on note creation | `implemented` | T2 |
| TS4 | Edit a note in-place (jot edit N) | `specd` | T3 |
| TS5 | Clear all notes (jot clear) | `ready` | T3 |
| TS6 | Word count and stats (jot stats) | `in_design` | — |
| TS7 | Deduplicate notes | `groomed` | T3 |
| TS8 | Pin a note to the top of jot list | `new` | T3 |
| TS9 | Copy note to clipboard (jot copy N) | `new` | T3 |
| TS10 | Archive notes older than N days | `blocked` | T3 |
| TS11 | Shell completion scripts (bash/zsh) | `specd` | — |
| TS12 | Man page generation | `question` | — |
| TS13 | Encrypted notes at rest | `in_progress` | T2 |
| TS14 | Import notes from a plain-text file | `groomed` | T2 |

Spec content rules (same discipline as existing tickets):
- `closed` / `implemented` / `specd` / `ready`: full four-section spec; `implemented` AC boxes all checked
- `in_design` / `groomed` / `in_progress`: Problem + partial AC
- `question`: Problem + AC + `### Open questions` section
- `blocked`: Problem + Approach + `### Open questions`
- `new`: no spec

TS12 open question: "Should the man page be generated from a hand-written Markdown file (using pandoc) or auto-generated from clap's help text? Decision needed before design can start."

TS10 open question: "What 'age' threshold is appropriate — calendar days since note was written, or days since last viewed? Waiting on supervisor guidance."

---

### Step 4 — Update the README

In the here-doc that writes `README.md` (around line 589), update:
- "14 tickets" → "35 tickets"
- "one epic" → "two epics"
- The ticket-state summary table to mention both epics

---

### Step 5 — Update the progress echo

Change the line:
```bash
echo "==> All 14 tickets created and transitioned"
```
to:
```bash
echo "==> All 35 tickets created and transitioned"
```

---

### Ordering constraint

Insert new code **after** the existing `T14` block and **before** the `# ─── 8. Write README.md` section, so all ticket variable IDs are available if cross-references are needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T23:59Z | — | new | philippepascal |
| 2026-04-08T23:59Z | new | groomed | apm |
| 2026-04-09T00:14Z | groomed | in_design | philippepascal |
| 2026-04-09T00:18Z | in_design | specd | claude-0409-0014-4828 |
| 2026-04-09T00:24Z | specd | ready | apm |
| 2026-04-09T00:33Z | ready | in_progress | philippepascal |