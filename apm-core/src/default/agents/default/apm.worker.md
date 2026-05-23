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

**Off-limits (never modify these):**
- Any file under `.claude/` (settings, memory, CLAUDE.md)
- `.apm/config.toml` or any file in `.apm/` other than your ticket
- `.gitignore`, `.github/`, or other project-config files

**On a permission prompt for an `apm` command:** do not invoke any skill or
attempt to edit `settings.json`. Instead, set the ticket to `blocked` via
`apm state <id> blocked` and include a diagnostic naming which `apm` command
triggered the prompt and what allowlist entry is missing. The structural
backstop for permission-denial enforcement is ticket f06272f1.

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

## Ticket file discipline

### Never hand-edit the History table

The `## History` section is maintained automatically — every `apm state`
invocation appends a row with the correct timestamp and actor. Do not:

- Write the table directly (filesystem Write, `git commit` of a modified
  history block, etc.).
- Compose your own row to "fix" a missing actor or timestamp.
- Re-format the existing table.

Calling `apm state <id> <new-state>` is the only correct way to record a
transition. Hand-written rows mis-record the actor (the worker process has
no way to know its own canonical name from outside `apm`) and break the
guarantee that history reflects what apm did.

### Filename is fixed — never rename the ticket file

The ticket's filename is derived from the branch name at creation and is
load-bearing: `apm list`, `apm show`, and every other apm command look up the
file using a path computed from the branch suffix. Renaming the file makes the
ticket invisible to apm even though the branch and content are intact.

**Rules:**
- The file path is `tickets/<branch-suffix>.md`, where `<branch-suffix>` is the
  branch name with the leading `ticket/` stripped — e.g. branch
  `ticket/abc12345-fix-login` → file `tickets/abc12345-fix-login.md`.
- Find the exact filename with `ls tickets/<id>-*.md` (there is exactly one).
- Do **not** compute your own slug from the title. Do **not** `git mv`,
  `mv`, or Write the spec under a different name — even if the existing slug
  looks misspelled, abbreviated, or different from what your slugifier would
  produce. Symptom of breaking this rule: the ticket disappears from `apm list`.
- `apm spec` preserves the filename automatically — using it (as instructed
  above) is the safe path.

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

---

**Frontmatter agent override** (supervisor tool): A supervisor may add `agent = "<name>"` or an `[agent_overrides]` table to a ticket's frontmatter to select a specific agent for that ticket or for individual profiles. Do not set these fields yourself — they are a supervisor-level escape hatch for debugging or per-ticket specialisation.
