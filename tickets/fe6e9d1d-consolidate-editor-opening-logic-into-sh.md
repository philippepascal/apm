+++
id = "fe6e9d1d"
title = "Consolidate editor-opening logic into shared CLI module"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
branch = "ticket/fe6e9d1d-consolidate-editor-opening-logic-into-sh"
created_at = "2026-04-07T22:30:48.429150Z"
updated_at = "2026-04-08T00:14:38.672964Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Editor-opening logic is duplicated across three command handlers with slight but meaningful variations:\n\n1. **`cmd/new.rs` lines 76–128** — resolves the editor, checks out the ticket branch, opens the editor on the ticket file, auto-commits (ignoring non-zero exit with a warning), then restores the original branch.\n2. **`cmd/show.rs` lines 83–130** — resolves the editor, writes ticket content to a temp file, opens it with inherited stdio, bails on non-zero exit, diffs the result, and commits via `git::commit_to_branch` if changed.\n3. **`cmd/review.rs` lines 158–180** — resolves the editor, opens it on an existing path with inherited stdio, bails on non-zero exit.\n\nAll three contain an identical block that reads `$VISUAL`, falls back to `$EDITOR`, then falls back to `"vi"`, and spawns the process by splitting the string on whitespace. This means any change to editor resolution or invocation (e.g., adding a new env var, changing error handling, adding logging) must be applied in three places, increasing the chance of divergence.

### Acceptance criteria

- [x] `apm new` behaves identically to before — opens the editor, commits the result, restores the original branch
- [x] `apm show --edit` behaves identically to before — opens the editor on a temp file, commits if the content changed
- [x] `apm review` behaves identically to before — opens the editor on the review file and bails on non-zero exit
- [x] Changing `$VISUAL` or `$EDITOR` at runtime is reflected in all three commands without touching cmd/ files
- [x] When neither `$VISUAL` nor `$EDITOR` is set, all three commands fall back to `vi`
- [x] `cargo test` passes with no new failures

### Out of scope

- Changing any user-visible behaviour of the editor flow (temp file strategy, commit messages, branch handling)\n- Consolidating the git operations that wrap the editor call (branch checkout in new.rs, commit_to_branch in show.rs)\n- Supporting editor commands with quoted arguments containing spaces (e.g. EDITOR='vim --cmd "set ft=markdown"')\n- Adding new editor-related features (syntax highlighting hints, line-number flags, etc.)

### Approach

1. **Create `src/editor.rs`** — new module with two public functions:

   ```rust
   /// Returns the editor to use: $VISUAL, then $EDITOR, then "vi".
   pub fn resolve() -> String

   /// Resolves the editor, spawns it on `path` with inherited stdio,
   /// and returns an error if the process could not be launched or
   /// exited with a non-zero status.
   pub fn open(path: &Path) -> Result<()>
   ```

   `open` splits the resolved editor string on whitespace to extract binary + flags,
   spawns with `Stdio::inherit()` on all three streams, and bails via `anyhow::bail!`
   on non-zero exit — matching the current behaviour in `show.rs` and `review.rs`.

2. **Register the module in `src/lib.rs`** — add `pub mod editor;`.

3. **Update `cmd/review.rs`** — delete the local `open_editor` function (lines 158–180).
   Call `crate::editor::open(path)?` in its place.

4. **Update `cmd/show.rs`** — replace the inline editor-resolution and spawn block
   (lines 93–114 approximately) with `crate::editor::open(&tmp_path)?`.
   Keep the temp-file write, read-back, diff, and `git::commit_to_branch` logic unchanged.

5. **Update `cmd/new.rs`** — replace the inline editor-resolution and spawn block with:
   ```rust
   let _ = crate::editor::open(&file_path);
   ```
   The `let _ =` intentionally discards the error, preserving the existing
   "commit whatever the user wrote, even if editor exits non-zero" behaviour.
   Keep the branch-checkout, git-add, git-commit, and branch-restore logic unchanged.

6. **Run `cargo test`** to confirm no regressions.

No changes to public CLI surface, config, ticket format, or git helpers.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:56Z | groomed | in_design | philippepascal |
| 2026-04-07T22:58Z | in_design | specd | claude-0407-2256-a8f8 |
| 2026-04-08T00:06Z | specd | ready | apm |
| 2026-04-08T00:14Z | ready | in_progress | philippepascal |
