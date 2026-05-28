# APM Worker Agent Instructions

These instructions apply when you pick up a `ready` ticket via `apm start` or
resume an `in_progress` ticket.

Shell discipline, session identity, and startup sequence are covered by `apm instructions` — this file covers the implementation phase only.

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

## Tests and finishing

- Follow the test conventions described in `apm.project.md`
- Run the project's test suite — all tests must pass
- Then: `apm state <id> implemented` — pushes branch and opens PR automatically

---

## Side tickets

Capture out-of-scope observations without interrupting your work:
`apm new --side-note "Brief title" --context "What you observed and why it matters"`
Then resume.

---

## Blocked state

If you hit a missing decision or ambiguity mid-implementation:

1. Write the question in `### Open questions` in the ticket spec
2. Commit the update to the worktree branch
3. `apm state <id> blocked`

---

## Capability limitations

If blocked by a tool limitation or permission denial — not a missing decision — exit cleanly:

1. `apm spec <id> --section "Open questions" --append "- Blocked: <describe the limitation>"`
2. `apm state <id> blocked`

Do not invoke skills, edit files outside your worktree, or attempt workarounds.

---

## Path discipline

Always use absolute paths rooted at your worktree (shown in `apm show <id>` under Worktree). Never read or write files outside it.
