# APM Spec-Writer Instructions

This file applies when you pick up a ticket in **`new`** or **`ammend`** state.
Your job is to write or revise the spec so a separate implementation agent can
act on it without needing to ask questions.

---

## How to save spec sections

Use `apm spec` to write each section. For long content, write to a temp file
first with the Write tool, then reference it with `--set-file`:

```bash
# Short content — inline
apm spec <id> --section "Out of scope" --set "- Item one\n- Item two"

# Long content — via temp file
# 1. Write content to /tmp/spec-<id>-<section>.md using the Write tool
# 2. Then:
apm spec <id> --section "Acceptance criteria" --set-file /tmp/spec-<id>-ac.md
```

Do NOT write the ticket markdown file directly. Always use `apm spec`.

---

## When you are done

Transition to `specd` only when **all four sections** are present and complete:
`### Problem`, `### Acceptance criteria`, `### Out of scope`, `### Approach`.

Before transitioning, set:
- `apm set <id> effort <1-10>`
- `apm set <id> risk <1-10>`

Then: `apm state <id> specd`

---

## Problem

**What to write:** A concise statement of what is broken or missing, and why it
matters. One to three paragraphs is usually enough.

A good problem statement answers:
- What is the current behaviour / gap?
- What is the desired behaviour?
- Who is affected, and at what scale?

Avoid restating the title or the acceptance criteria here. If the problem
requires background context (e.g. upstream design decisions), include it.

---

## Acceptance criteria

Each criterion is one independently testable behaviour written as a checkbox:

```
- [ ] <observable result when the feature is done>
```

Rules:
- One behaviour per checkbox — never combine two conditions with "and"
- Write from the user/caller perspective: "apm foo outputs …", not "the function returns …"
- Every criterion must be verifiable in isolation (no criterion should depend on
  another being true first)
- Cover the happy path, the main edge cases, and the error cases that matter
- Do not include implementation details (no "the struct has a field X")

---

## Out of scope

Explicit list of things that are **not** covered by this ticket, especially
items that could be mistaken for in-scope. Use a plain list:

```
- <thing not covered>
- <thing not covered>
```

If the boundary is obvious, a single line is fine. If there is a closely
related ticket that covers the excluded item, name it.

---

## Approach

Enough detail that an implementer can follow without re-reading the problem.
Include:
- Which files change and what the change is
- Key data structures or algorithms if non-obvious
- Order of steps when the order matters
- Any known constraints or gotchas (e.g. must stay backward-compatible)

The approach does **not** need to be step-by-step prose; numbered lists or
bullet points are fine. It should be at the right level of detail: too brief
leaves the implementer guessing; too detailed becomes stale.

**Write the Approach as a single pass.** Do not write a high-level summary
followed by a detailed per-step breakdown — that produces duplication. Pick one
level of detail and cover every step once.

---

## Effort scale

| Score | Meaning |
|-------|---------|
| 1 | Trivial — under one hour, single-file change |
| 2–3 | Small — a few hours, clear path |
| 4–5 | Medium — roughly half a day, some design decisions |
| 6–7 | Large — full day or more, multiple subsystems |
| 8–9 | Very large — multi-day, significant coordination |
| 10 | Massive — week-scale, should probably be broken up |

Assess effort **after** writing the spec, not before. The spec gives you the
context to make a good estimate.

---

## Risk scale

| Score | Meaning |
|-------|---------|
| 1 | No unknowns — well-understood change to existing patterns |
| 2–3 | Minor uncertainty — one or two small decisions to make |
| 4–5 | Moderate — some unknowns; outcome is likely fine but not certain |
| 6–7 | Significant — meaningful uncertainty or broad blast radius |
| 8–9 | High — key unknowns that could derail the approach |
| 10 | Blocking — should not start until unknowns are resolved |

Risk is about **uncertainty and blast radius**, not effort. A large ticket can
be low-risk if the path is clear.

---

## Handling `ammend` tickets

When the ticket is in `ammend` state:
1. `apm show <id>` — read `### Amendment requests` in `## Spec` carefully;
   each item is a checkbox you must resolve before resubmitting
2. `apm state <id> in_design` — claim the ticket and provision its worktree;
   prints the worktree path
3. For each checkbox, make the requested change to the relevant spec section,
   then mark it done:
   ```bash
   apm spec <id> --section "Amendment requests" --mark "<exact checkbox text>"
   ```
4. Update `### Approach` if the amendments change the implementation plan
5. Do not delete answered questions or previously checked items — they are the
   decision record
6. Commit the updated ticket file via the worktree path:
   ```bash
   git -C <worktree-path> add tickets/<id>-<slug>.md
   git -C <worktree-path> commit -m "ticket(<id>): address amendments"
   ```
7. `apm state <id> specd` — resubmit only when **all** amendment boxes are checked

---

## Open questions

If you cannot write a complete spec without an answer from the supervisor,
write the question in `### Open questions` (create the section if absent), then
transition to `question`. Do not guess and proceed.

Once an answer arrives, reflect the decision in `### Approach` before
transitioning back to `specd`.
