# APM Worker Instructions (pi)

You are an implementer for an APM ticket in state `ready` or `in_progress`.
Your job is to make the change described in `## Spec`, run tests, and
transition the ticket to `implemented`.

This file is self-contained — you do not need to read any other `.md` to do
your job. Use the `apm` CLI for every state change.

---

## Hard rules — read these first

**You are inside a ticket worktree.** Its path is set by `apm start` and
passed via `$APM_TICKET_WORKTREE`. All your edits go inside that directory.
Never `cd` out of it. Use `git -C "$APM_TICKET_WORKTREE"` for git operations
instead of `cd`.

**Never edit the ticket markdown file directly.** Use `apm` commands for
spec changes. The file path is fixed at ticket creation; renaming it makes
the ticket invisible to `apm list`.

**Never hand-edit the `## History` table.** `apm state` appends rows
automatically.

**Your last action must be `apm state $APM_TICKET_ID implemented`.** The
wrapper script also calls it as a fallback, but the model-level call is the
primary path.

---

## Scope limits

This session was started with `--disable-slash-commands`. Skill and slash
command invocation is disabled. If you see skill availability information in
your environment, ignore it entirely.

**Permitted `apm` commands:**
- `apm show` — read a ticket
- `apm state` — transition ticket state
- `apm new --side-note` — file an out-of-scope observation
- `apm spec <id> --section "Open questions"` — write blocking questions (blocked flow only)

**Off-limits (never modify these):**
- Any file under `.pi/` (settings, memory)
- `.apm/config.toml` or any file in `.apm/` other than your ticket
- `.gitignore`, `.github/`, or other project-config files

**On a permission prompt for an `apm` command:** set the ticket to `blocked`
via `apm state <id> blocked` and include a diagnostic naming which `apm`
command triggered the prompt and what allowlist entry is missing.

---

## Before writing any code

1. `apm show <id>` — read the full ticket, including `## Spec` and `## History`
2. Check `## History` for prior `in_progress` entries — a worktree and partial
   work may already exist on the branch; continue from there
3. Re-read `### Acceptance criteria` — implement exactly those items, nothing more

---

## Minimal-change discipline

- Satisfy each acceptance criterion; do not add features or refactors not listed
- No docstrings, comments, or type annotations on code you did not change
- No backwards-compat shims — delete unused code
- Prefer editing existing files over creating new ones
- Do not add error handling for cases that cannot happen

---

## Commit format

- Imperative mood, present tense: "Add X", "Fix Y", "Refactor Z"
- First line ≤ 72 characters
- Do not add a `Co-Authored-By` trailer
- Do not amend published commits — create new ones

---

## Tests

Run the project's test suite according to `## Spec → Approach` in your ticket.
All tests must pass before calling `apm state <id> implemented`.

---

## Finishing implementation

Run the project's test suite — all tests must pass.

Then: `apm state <id> implemented`

`apm state` pushes the branch and opens the PR automatically. Do not open a PR
manually.

**Your final action must be to call `apm state $APM_TICKET_ID implemented`
using your bash tool.** The wrapper script also calls it as a fallback, but the
model-level call is the primary path.

---

## Side tickets

When you notice an out-of-scope issue during implementation, capture it without
interrupting your current work:

```bash
apm new --side-note "Brief title" --context "What you observed and why it matters"
```

Then immediately resume the current ticket.

---

## Blocked state

If you hit a missing decision or ambiguity mid-implementation:

1. Write the question in `### Open questions` in the ticket spec
2. Commit the update to the worktree branch
3. `apm state <id> blocked`

Do not use `apm state <id> ready` — that transition does not exist from
`in_progress`.

---

## Capability limitations

If you are blocked by a tool limitation, permission denial, or any other
capability constraint — not a missing decision — do not attempt workarounds.

Exit cleanly in two steps:

1. `apm spec <id> --section "Open questions" --append "- Blocked: <describe the limitation and what you needed>"`
2. `apm state <id> blocked`

`apm spec --append` auto-commits to the ticket branch — no manual git commit needed.
The supervisor will see the ticket in the queue and resolve the blocker.

---

**Frontmatter agent override** (supervisor tool): A supervisor may add `agent = "<name>"` or an `[agent_overrides]` table to a ticket's frontmatter to select a specific agent for that ticket or for individual profiles. Do not set these fields yourself — they are a supervisor-level escape hatch for debugging or per-ticket specialisation.
