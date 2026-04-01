# APM — PR Review Agent Role

> **Status:** Draft · **Date:** 2026-03-31
> A new agent role that performs critical code review between `implemented` and
> supervisor acceptance, catching the class of bugs that compile and build
> cleanly but fail silently at runtime.

---

## Motivation

The current worker loop has a blind spot. A worker agent implements a ticket,
runs `cargo test` / `npm run build`, checks the acceptance criteria boxes, and
marks the ticket `implemented`. Everything passes. But the code can still be
wrong in ways that only show up when a human (or a reviewer in a different
cognitive mode) reads it critically:

- A prop name that doesn't match the installed library version — compiles,
  renders, buttons do nothing
- A callback signature that's one level off — compiles, no TypeScript error,
  logic never fires
- A configuration key that's silently ignored — no crash, wrong behavior
- A test that hits live data and passes today but breaks when that data changes

These bugs are not caught by build tools. They require someone to read the
code and ask: "would this actually work if I ran it?"

The worker agent is in **execution mode**: produce code that satisfies the
spec, make it compile, check the boxes. A reviewer agent is in **review mode**:
read the code skeptically, question every API call, look for things that would
silently fail at runtime. These are different cognitive tasks, and mixing them
degrades both.

---

## Proposed workflow change

Add a new `in_review` state between `implemented` and `accepted`:

```
implemented → in_review   (trigger: command:review, actor: agent)
in_review   → accepted    (trigger: manual, actor: agent, preconditions: [review_passed])
in_review   → ammend      (trigger: manual, actor: agent)
```

The reviewer agent:
1. Is dispatched automatically when a ticket reaches `implemented`
2. Reads the PR diff and the ticket spec
3. Produces a structured review
4. Either approves (→ `accepted`) or writes amendment requests and sends back
   (→ `ammend`)

The supervisor is notified of the outcome but does not need to read every PR
before accepting. They can still override at any point.

---

## What the reviewer checks

### 1. API correctness against installed versions

Read `package.json` / `Cargo.toml` for the exact installed version of every
library used in the diff. Cross-reference the code against that version's API
— not latest docs, not memory, the installed version.

Flag: wrong method names, renamed props, changed callback signatures,
removed features. These are the most common source of "compiles but doesn't
work" bugs in frontend code.

**Critical discipline: verify before flagging, not after.**

Memory of a library's API is version-specific and frequently wrong. An API
that changed between v2 and v4 will feel certain but be incorrect. Before
writing any amendment request about an API call, prop name, or callback
signature, verify it against the actual installed version's type definitions:

```
node_modules/<library>/dist/<library>.d.ts
```

or the library's changelog for the installed version. If you cannot verify,
say so explicitly rather than flagging with false confidence.

The cost of a wrong amendment request is high: the worker spends a cycle
"fixing" correct code, the fix breaks the build, and the supervisor loses
trust in the review process. A missed bug is recoverable; a false positive
that corrupts working code is worse.

**When TypeScript is available, use it as the oracle.** If you are uncertain
about an API, mentally apply the type signature: would the TypeScript compiler
accept this call with the installed version's types? If you cannot answer
confidently from the type definitions, do not flag it as a required fix —
flag it as a question instead: "verify that `panelRef` is the correct prop
name for react-resizable-panels v4.8.0 before accepting".

### 2. Silent failures

Look for code paths where a bug produces no error — null/undefined
propagation, wrong type that satisfies TypeScript, ignored return values,
callbacks that are never registered. Ask: if this is wrong, what would happen?
If the answer is "nothing visible", that's a silent failure worth flagging.

### 3. Acceptance criteria vs actual behavior

Re-read each acceptance criterion. For each one, trace through the code and
ask: would this actually pass if I ran it right now? Not "does the code
attempt to implement this" but "would a user experience it working".

Flag criteria that are structurally satisfied (the button exists) but
behaviorally broken (clicking it does nothing).

### 4. Test quality

Are the tests actually testing the right thing? Do they cover the acceptance
criteria or do they cover an easier proxy? Would they catch a regression if
the implementation changed?

Flag: tests that pass trivially, tests that depend on external mutable state,
tests that check structure but not behavior.

### 5. Spec drift

Did the implementation match the approach? Flag significant deviations —
not as blockers, but so the supervisor is aware and can decide if the approach
change is acceptable.

---

## What the reviewer does NOT do

- Rewrite or fix the code — that is the worker's job after `ammend`
- Add features or suggest improvements beyond the spec
- Block on style or cosmetic issues
- Run the code (the reviewer has no browser, no running server)
- Re-implement anything — review only

---

## Review output format

The reviewer writes its findings directly into `### Code review` in the ticket,
using the `tasks` format (checkboxes) for required fixes and plain text for
observations that don't require action.

```markdown
### Code review

Required fixes:
- [ ] `panelRef` is not a valid prop in react-resizable-panels v4 — use `ref`
- [ ] `onResize` receives `number` not `{ asPercentage: number }` in v4
- [ ] `direction` not `orientation` on ResizablePanelGroup

Observations (no action required):
- `columnSizes` in the store is never updated after user resizes; will matter
  once something reads it
```

Required fixes → ticket goes to `ammend`.
Observations only → ticket goes to `accepted`.

---

## `apm.reviewer.md` — first draft

```markdown
# APM Reviewer Agent Instructions

You are a Reviewer agent. You have been assigned ticket #N which is in
`in_review` state. A worker agent has already implemented this ticket and
opened a PR. Your job is to review the PR diff critically and either approve
it or send it back for amendments.

## Your identity

Generate a session name at the start: `claude-MMDD-HHMM-XXXX`.
Export it: `export APM_AGENT_NAME=claude-MMDD-HHMM-XXXX`

## Step 1 — Read the ticket

    apm show <id>

Read the spec carefully. Understand what was asked: the acceptance criteria,
the approach, and the out-of-scope boundaries.

## Step 2 — Read the PR diff

    gh pr list --head ticket/<branch>
    gh pr diff <number>

Read the entire diff. Do not skim.

## Step 3 — Verify library APIs against installed versions

For every external library used in the diff:
1. Find its version in `package.json` or `Cargo.toml` in the diff or in the
   repo root
2. Cross-reference the API used in the code against that exact version
3. Do not rely on memory or latest docs — version APIs change

Pay particular attention to: prop names, callback signatures, method names,
configuration keys.

## Step 4 — Check for silent failures

For each piece of logic in the diff, ask: if this is subtly wrong, what
happens? If the answer is "nothing visible — it compiles, it renders, no
error", trace the logic more carefully. Silent failures are the hardest bugs
to catch and the most important to flag.

Common patterns:
- A callback that is registered on the wrong event or with the wrong signature
- A ref that is never populated because the prop name is wrong
- A config key that is ignored because the name doesn't match what the library
  expects
- An async path that swallows errors silently

## Step 5 — Trace each acceptance criterion

For each checkbox in `### Acceptance criteria`, trace through the code and
ask: would this criterion pass if a user ran the app right now?

Do not ask "does the code attempt to implement this". Ask "would it work".

## Step 6 — Assess test quality

For each test: would it catch a regression if the implementation broke? Does
it test behavior or just structure? Does it depend on external mutable state?

## Step 7 — Write your findings

Write your review into the `### Code review` section of the ticket:

    apm spec <id> --section "Code review" --set "..."

Use this structure:
- Required fixes: checkboxes for anything that must change before the ticket
  can be accepted
- Observations: plain text for issues that don't require action but the
  supervisor should know about

## Step 8 — Transition

If there are required fixes:

    apm state <id> ammend

If there are only observations (or nothing to flag):

    apm state <id> accepted

## What you must not do

- Do not fix the code yourself
- Do not add features or suggest improvements beyond what was asked
- Do not block on style, formatting, or naming preferences
- Do not re-implement anything
- Do not open a browser or run the server — you cannot
- Do not check boxes in `### Acceptance criteria` — that is the worker's job
```

---

## Configuration changes

### New state in `apm.toml`

```toml
[[workflow.states]]
id           = "in_review"
label        = "In Review"
color        = "#f59e0b"
layer        = 2
actionable   = ["agent"]
instructions = ".apm/apm.reviewer.md"

  [[workflow.states.transitions]]
  to      = "accepted"
  trigger = "manual"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "accepted"
  trigger = "manual"
  actor   = "supervisor"
```

### Modified `implemented` state

Add a `command:review` trigger alongside the existing transitions:

```toml
  [[workflow.states.transitions]]
  to      = "in_review"
  trigger = "command:review"
  actor   = "agent"
```

`apm work` dispatches a reviewer agent when it finds a ticket in `implemented`
state, using the same priority queue as worker dispatch.

---

## Known reviewer failure modes

These are failure modes observed in practice, not hypothetical. The reviewer
instructions must be written to prevent them.

### Confident incorrectness on library APIs

A reviewer flagged `panelRef`, `orientation`, `PanelImperativeHandle`, and
`{ asPercentage: number }` as wrong in react-resizable-panels v4.8.0. All
four were correct for the installed version. The reviewer was applying API
knowledge from a different version and did not check the actual type
definitions before writing four amendment requests. The worker had done the
right thing; the reviewer's amendments would have broken working code.

**Root cause:** API knowledge felt certain but was version-specific. No
verification step was performed.

**Mitigation:** Make verification a hard gate, not a recommendation. The
reviewer agent instructions (apm.reviewer.md) must require checking
`node_modules/<lib>/dist/*.d.ts` for every library API claim before it can
be written as a required fix. Unverified API concerns must be written as
questions, not amendment requests.

### Review confidence ≠ review correctness

The reviewer operates in a skeptical mode that is valuable for catching silent
failures, but that same skepticism can generate false positives when applied
to unfamiliar APIs. Skepticism without verification is just noise.

A useful heuristic: if the code compiled and the TypeScript build passed, any
API correctness concern should be treated as "possibly wrong" not "definitely
wrong" until verified against the type definitions. The TypeScript compiler
already caught the real API errors; surviving that check is meaningful signal.

---

## Open questions

- **Review SLA**: should there be a timeout after which the supervisor is
  notified if no reviewer has acted? Long-running reviews block the worker
  queue.

- **Reviewer identity**: should the reviewer agent be a separate named process,
  or can the same delegator that dispatched the worker also review? Separation
  is cleaner; sharing is faster.

- **Stacked reviews**: if the same reviewer pattern catches the same class of
  bug across many tickets (e.g. wrong library API version), is there a way to
  feed that back as a spec-writing improvement rather than fixing it
  ticket-by-ticket?

- **Cost vs value**: review adds a full agent turn per ticket. For trivial
  tickets (e.g. adding two stub files) this is waste. A heuristic based on
  effort score — only review tickets with `effort >= 2` — might be the right
  default.
