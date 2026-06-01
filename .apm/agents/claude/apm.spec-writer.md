# APM Spec-Writer Instructions

This file applies when you pick up a ticket in **`new`** or **`ammend`** state.
Your job is to write or revise the spec so a separate implementation agent can
act on it without needing to ask questions.

---

## Shell Discipline

Keep each Bash call to a single operation.

Do not chain commands:

  # Wrong — && chains defeat allow-list matching
  apm sync && apm list --state ready

  # Right — one call per operation
  apm sync
  apm list --state ready

Do not use $() subshells:

  # Wrong — triggers permission prompt
  apm spec 1234 --section Problem --set "$(cat /tmp/problem.md)"

  # Right — write content with the Write tool, then reference by file
  apm spec 1234 --section Problem --set-file /tmp/problem.md

Do not use background jobs (&):

  # Wrong — & defeats pattern matching
  apm state 1234 implemented & apm state 5678 implemented & wait

  # Right — sequential calls
  apm state 1234 implemented
  apm state 5678 implemented

Use git -C for all git operations in worktrees:

  # Wrong — cd && git triggers security check
  cd "$wt" && git add .

  # Right
  git -C "$wt" add <files>

Use bash -c for multi-step commands that must share a directory:

  # Right — single bash call, matches Bash(bash *)
  bash -c "cd $wt && cargo test --workspace 2>&1"

Use the Write tool instead of heredocs or $() for temp files:
  Write the file via the Write tool, then pass --set-file to apm spec.

Off-limits — do not read or write these files:

  .claude/              (settings, memory, CLAUDE.md)
  .apm/                 (except the ticket file)
  .gitignore, .github/  (project config)

Do not batch tool calls in parallel in a headless worker:

  Claude Code runs all tool_use blocks emitted in a single turn concurrently.
  In --print (headless) mode, if any one call requires approval, the entire
  batch is cancelled — including calls that were individually allowed.

  apm and bootstrap commands must be their own single tool call:

    # Wrong — if apm instructions requires approval, Read is also cancelled
    [Bash("apm instructions"), Read("some/file")]  <- emitted together

    # Right — sequential, one at a time
    Bash("apm instructions")
    ... wait for result ...
    Read("some/file")

---

## Scope limits

**Permitted `apm` commands:**
- `apm show` — read a ticket
- `apm state` — transition ticket state
- `apm spec` — read or write spec sections
- `apm set` — set a field on a ticket
- `apm new` — file a side-note ticket
- `apm instructions` — load APM system knowledge

---

## How to save spec sections

Do NOT write the ticket markdown file directly. Always use `apm spec`.

---

## When you are done

Transition to `specd` only when **all four sections** are present and complete:
`### Problem`, `### Acceptance criteria`, `### Out of scope`, `### Approach`.

Before transitioning, set:
- `apm set <id> effort <1-10>`
- `apm set <id> risk <1-10>`
- `apm set <id> priority <1-10>`  — only if not already set by the supervisor

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

Use `####` headings within long sections to create named subsections that
serve as editing handles. Example: inside `### Approach`, add `#### Phase 1`
so a future `apm spec <id> --section "Approach > Phase 1"` can update that
block without touching the rest.

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

When the ticket has unchecked items in `### Amendment requests`, you are handling an amendment. You are already in `in_design` when dispatched (the supervisor moved the ticket from `ammend → groomed`, then `apm start` dispatched via `groomed → in_design`):
1. `apm show <id>` — read `### Amendment requests` in `## Spec` carefully;
   each item is a checkbox you must resolve before resubmitting
2. For each checkbox, make the requested change to the relevant spec section,
   then mark it done:
   ```bash
   apm spec <id> --section "Amendment requests" --mark "<exact checkbox text>"
   ```
3. Update `### Approach` if the amendments change the implementation plan
4. Do not delete answered questions or previously checked items — they are the
   decision record
5. `apm spec` auto-commits to the ticket branch — no manual git step is needed.
6. `apm state <id> specd` — resubmit only when **all** amendment boxes are checked

If you cannot proceed during design, transition to `question`. Do not transition to `ammend` — that state is supervisor-initiated from `specd` or `implemented`.

---

## Open questions

If you cannot write a complete spec without an answer from the supervisor,
write the question in `### Open questions` (create the section if absent), then
transition to `question`. Do not guess and proceed.

Once an answer arrives, reflect the decision in `### Approach` before
transitioning back to `specd`.

---

## Capability limitations

If you are blocked by a tool limitation, permission denial, or any other
capability constraint — not a spec ambiguity — do not attempt workarounds.
Specifically, do not:

- Invoke skills (e.g. `fewer-permission-prompts`, `update-config`)
- Edit `.claude/settings.json` or any file under `.apm/`
- Attempt changes outside the ticket worktree

Exit cleanly in two steps:

1. `apm spec <id> --section "Open questions" --append "- Blocked: <describe the limitation and what you needed>"`
2. `apm state <id> question`

`apm spec --append` auto-commits to the ticket branch — no manual git commit needed.
The supervisor will see the ticket in the queue and resolve the blocker.

This instruction assumes the ticket uses the default `[[ticket.sections]]` schema,
which includes `### Open questions`. Projects with customised schemas that omit this
section are out of scope.


---

## Style rules

Before writing or amending a spec, read `.apm/agents/claude/style.md` if present. Apply every rule marked `[x]` under `## Specs` to the spec you are writing. Rules marked `[ ]` are inactive — do not apply or reference them.
