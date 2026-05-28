+++
id = "d2720f0b"
title = "apm new editor flow must not checkout the ticket branch in main"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d2720f0b-apm-new-editor-flow-must-not-checkout-th"
created_at = "2026-05-28T07:37:16.051173Z"
updated_at = "2026-05-28T07:48:51.458701Z"
depends_on = ["f16e4035"]
+++

## Spec

### Problem

`apm new` (without `--no-edit`) currently checks out the ticket branch in the main working tree so the ticket file lands on disk for the editor. During the editor session, `HEAD` points to the ticket branch rather than the branch the user was on before. This side effect is what allowed `find_worktree_for_branch` (before f16e4035) to return the main repo path when the ticket branch was checked out there, triggering incorrect dispatch. Even after f16e4035, the checkout-based flow is undesirable: it moves HEAD for the duration of an interactive session (potentially minutes), blocking any concurrent read of the main repo's branch state, and it makes `--no-edit` a safety requirement for agents rather than a performance flag.

The desired behaviour is: read the ticket file from the ticket branch via git plumbing, write it to a temp file, open the editor on that temp file, read the result back, and commit it to the ticket branch using `commit_to_branch` — which already handles temp worktrees and never touches HEAD. The main working tree is never modified.

### Acceptance criteria

- [x] After `apm new <title>` (without `--no-edit`) completes, `git branch --show-current` in the main repo returns the same branch that was checked out before the command ran.
- [x] The content shown in the editor is the content committed on the ticket branch at the moment the editor launches (not an empty file or stale content).
- [x] Changes made in the editor are committed to the ticket branch at `tickets/<id>-<slug>.md` with the commit message `write spec`.
- [x] `git checkout` is never invoked against the main repo during the editor session — no `git checkout <ticket-branch>` or `git checkout <prev-branch>` calls are made.
- [x] The temp file created for editing is removed after the editor exits (best-effort; a removal failure must not fail the command).
- [x] `--no-edit` is unaffected: when passed, no editor opens, no temp file is created, and HEAD is never moved.

### Out of scope

- Conflict resolution when a concurrent process commits to the ticket branch while the editor is open.
- Changes to `--no-edit` semantics.
- Changes to how `ticket::create` writes the initial commit.
- Changes to `apm-core` — all changes are in `apm/src/cmd/new.rs`.
- Skipping the "write spec" commit when the user makes no edits (always commits if the editor exits 0).

### Approach

All changes are in `apm/src/cmd/new.rs`.

#### Replace `open_editor`

Remove the `config: &Config` parameter — it was only used to compute the `prev_branch` fallback, which the new implementation does not need. Update the call site in `run()` accordingly.

New body of `open_editor(root: &Path, branch: &str, rel_path: &str) -> Result<()>`:

1. Read the ticket file content from the branch using git plumbing (no checkout):
   ```rust
   let content = apm_core::git_util::read_from_branch(root, branch, rel_path)?;
   ```

2. Create a temp file. Use the ticket filename as the suffix so the editor title bar is identifiable and syntax highlighting fires on `.md`:
   ```rust
   let fname = std::path::Path::new(rel_path)
       .file_name().unwrap().to_string_lossy();
   let tmp_path = std::env::temp_dir()
       .join(format!("apm-{}-{}", std::process::id(), fname));
   std::fs::write(&tmp_path, &content)?;
   ```

3. Open the editor on the temp file:
   ```rust
   crate::editor::open(&tmp_path)?;
   ```

4. Read back the (possibly modified) content:
   ```rust
   let new_content = std::fs::read_to_string(&tmp_path)?;
   ```

5. Commit to the ticket branch via the existing `commit_to_branch` helper, which uses a temp git worktree and never moves HEAD:
   ```rust
   apm_core::git_util::commit_to_branch(root, branch, rel_path, &new_content, "write spec")?;
   ```

6. Clean up the temp file (best-effort — a failure must not propagate):
   ```rust
   let _ = std::fs::remove_file(&tmp_path);
   ```

#### Integration test

Add a test in `apm/tests/integration.rs` that:

1. Calls `init_repo()` to get a clean repo on `main`.
2. Records the current branch with `git branch --show-current`.
3. Runs `apm new "My ticket"` (without `--no-edit`) with `EDITOR=true` in the environment. (`true` exits 0 without touching the file — simulates a no-op edit session.)
4. Asserts the current branch is still `main`.
5. Asserts a `ticket/` branch exists with a `write spec` commit at its tip (confirming the edit was committed).

Use `std::process::Command` directly (not `run_apm`) so the `EDITOR` env var can be set on the command without affecting the test process.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T07:37Z | — | new | philippepascal |
| 2026-05-28T07:37Z | new | groomed | philippepascal |
| 2026-05-28T07:39Z | groomed | in_design | philippepascal |
| 2026-05-28T07:43Z | in_design | specd | claude |
| 2026-05-28T07:44Z | specd | ready | philippepascal |
| 2026-05-28T07:48Z | ready | in_progress | philippepascal |
