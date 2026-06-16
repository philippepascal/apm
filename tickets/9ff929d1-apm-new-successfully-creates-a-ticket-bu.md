+++
id = "9ff929d1"
title = "apm new successfully creates a ticket but outputs Error:"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9ff929d1-apm-new-successfully-creates-a-ticket-bu"
created_at = "2026-06-16T18:19:39.121805Z"
updated_at = "2026-06-16T20:27:27.906100Z"
+++

## Spec

### Problem

When `apm new` is run without `--no-edit`, it opens `$EDITOR` after creating the ticket. After the editor closes, `open_editor` in `apm/src/cmd/new.rs` calls `commit_to_branch` with the content read back from the temp file. If the user makes no changes (or saves without editing), the content is identical to what `ticket::create` already committed to the branch. `git commit` then exits with code 1 and writes "nothing to commit, working tree clean" to **stdout** — not stderr. The `git_util::run` helper captures only stderr for error messages, so `String::from_utf8_lossy(&out.stderr).trim()` is empty. `anyhow::bail!("{}", "")` then propagates an error with an empty message string, which anyhow formats as `Error: ` (a trailing space). In most terminals this renders as `Error:` on its own line, with no explanation.

The user sees the ticket created successfully (`Created ticket ...`) and then immediately sees `Error:` with no explanation — which is alarming and confusing because the ticket does exist and is valid. The actual operation that failed was a no-op commit attempt. There are two defects: the empty bail message in `git_util::run` (which can affect any git command that writes failure output to stdout), and the unnecessary commit attempt in `open_editor` when content is unchanged.

### Acceptance criteria

- [ ] `apm new "title"` without `--no-edit`, when the user closes the editor without changes, prints `Created ticket ...` and exits 0 with no further output
- [ ] `apm new "title"` without `--no-edit`, when the user edits and saves the ticket, prints `Created ticket ...` and exits 0 with no further output
- [ ] When any `git` command invoked via `git_util::run` fails with empty stderr and non-empty stdout, the error message includes the stdout content rather than being blank
- [ ] `apm new --no-edit "title"` is unaffected: still creates the ticket and exits 0 without opening an editor

### Out of scope

- Changing the default `--no-edit` behaviour; agents should still pass `--no-edit` explicitly
- Fixing `apm show --edit` or `apm review`, which have their own editor flows
- Adding retry logic for genuine commit failures (permission errors, locked index, etc.)

### Approach

Two changes, each independently verifiable.

#### Fix 1 — `apm/src/cmd/new.rs`: skip commit when content is unchanged

In `open_editor`, capture the original content returned by `read_from_branch`, then compare it to `new_content` after the editor closes. Only call `commit_to_branch` when they differ:

```rust
fn open_editor(root: &Path, branch: &str, rel_path: &str) -> Result<()> {
    let content = apm_core::git_util::read_from_branch(root, branch, rel_path)?;
    // ... write tmp_path, open editor ...
    let new_content = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    if new_content != content {
        apm_core::git_util::commit_to_branch(root, branch, rel_path, &new_content, "write spec")?;
    }
    Ok(())
}
```

`content` is already bound at the top of `open_editor`, so this requires no structural change — just add the `if new_content != content` guard before the `commit_to_branch` call and move the `remove_file` call before the guard (order doesn't matter for correctness).

#### Fix 2 — `apm-core/src/git_util.rs`: include stdout in error when stderr is empty

In the `run` helper, when a git command exits non-zero and stderr is empty after trimming, fall back to stdout for the error message:

```rust
pub(crate) fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .context("git not found")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stderr = stderr.trim();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stdout = stdout.trim();
        let msg = if stderr.is_empty() { stdout } else { stderr };
        anyhow::bail!("{}", msg);
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}
```

This is a targeted defensive fix: the common case (stderr has content) is unchanged; only the empty-stderr branch changes.

#### Tests

- Unit test in `apm-core/src/git_util.rs` (or inline in the module): run a git command that fails with nothing on stderr (e.g. `git commit` on a clean tree) and assert the error message is non-empty.
- Integration test in `apm/tests/integration.rs`: call `ticket::create` then simulate the `open_editor` path (write same content, call `commit_to_branch`) and assert it returns `Ok(())` after Fix 1, or assert the error message is non-empty after Fix 2.
- Both are straightforward with the existing temp-git-repo test harness.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:19Z | — | new | philippepascal |
| 2026-06-16T18:20Z | new | groomed | philippepascal |
| 2026-06-16T18:23Z | groomed | in_design | philippepascal |
| 2026-06-16T18:27Z | in_design | specd | claude |
| 2026-06-16T20:24Z | specd | ready | philippepascal |
| 2026-06-16T20:27Z | ready | in_progress | philippepascal |
