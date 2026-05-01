# APM Worker Agent Instructions

These instructions apply when you pick up a `ready` ticket via `apm start` or
resume an `in_progress` ticket.

Read `apm.agents.md` for startup, identity, worktree setup, and shell
discipline. This file covers the implementation phase only.

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

## Shell discipline

Claude Code's permission system matches the **start** of the command string.
Compound calls defeat this matching and generate permission prompts. Keep each
Bash call to a single operation.

**Do not chain commands:**
```bash
# Wrong
apm sync && apm list --state ready

# Right — one call per operation
apm sync
apm list --state ready
```

**Do not use `$()` subshells:**
```bash
# Wrong
apm spec 1234 --section Problem --set "$(cat /tmp/problem.md)"

# Right — write content with the Write tool, then reference by file
apm spec 1234 --section Problem --set-file /tmp/problem.md
```

**Do not use background jobs (`&`):**
```bash
# Wrong
cargo test & cargo clippy & wait

# Right — sequential calls
cargo test
cargo clippy
```

**Use `git -C` for all git operations in worktrees:**
```bash
# Wrong
cd "$wt" && git add .

# Right
git -C "$wt" add <files>
```

**Use `bash -c` for multi-step commands that must share a directory:**
```bash
# Right — single bash call, matches Bash(bash *)
bash -c "cd $wt && cargo test --workspace 2>&1"
```

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
