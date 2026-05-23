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

## Tests

- Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
- Integration tests in `apm/tests/integration.rs` — use temp git repos, no
  fixture files needed
- Run `cargo test --workspace` — all tests must pass before calling `apm state <id> implemented`

---

## Finishing implementation

Run `cargo test --workspace` — all tests must pass.

Then: `apm state <id> implemented`

`apm state` pushes the branch and opens the PR automatically. Do not open a PR manually.

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
Specifically, do not:

- Invoke skills (e.g. `fewer-permission-prompts`, `update-config`)
- Edit `.claude/settings.json` or any file under `.apm/`
- Attempt changes outside the ticket worktree

Exit cleanly in two steps:

1. `apm spec <id> --section "Open questions" --append "- Blocked: <describe the limitation and what you needed>"`
2. `apm state <id> blocked`

`apm spec --append` auto-commits to the ticket branch — no manual git commit needed.
The supervisor will see the ticket in the queue and resolve the blocker.

This instruction assumes the ticket uses the default `[[ticket.sections]]` schema,
which includes `### Open questions`. Projects with customised schemas that omit this
section are out of scope.

---

## Path discipline

Your working directory is the ticket worktree. Never read or write files outside
it. Always use absolute paths rooted at your worktree. The worktree path appears
in `apm show <id>` under Worktree — note it at the start of your run.

```
# Correct — absolute path inside your worktree
/Users/you/repos/myproject/.apm--worktrees/ticket-abc123-my-feature/src/main.rs

# Wrong — path in the main repo root (leaks edits outside your worktree)
/Users/you/repos/myproject/src/main.rs
```

If a tool call resolves to a path outside your worktree, stop immediately, file
a side-note ticket, and set yourself to blocked.
