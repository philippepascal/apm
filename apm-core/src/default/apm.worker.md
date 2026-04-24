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

- Write tests appropriate for your project's structure and conventions
- Run your project's test suite — all tests must pass before calling `apm state <id> implemented`

---

## Finishing implementation

Run your project's test suite — all tests must pass.

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
apm state 1234 implemented & apm state 5678 implemented & wait

# Right — sequential calls
apm state 1234 implemented
apm state 5678 implemented
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
bash -c "cd $wt && <your-test-command> 2>&1"
```
