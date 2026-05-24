+++
id = "1df8c1c6"
title = "Suppress leaked stderr from .status() subprocess calls in apm-core"
state = "implemented"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1df8c1c6-suppress-leaked-stderr-from-status-subpr"
created_at = "2026-05-24T21:19:59.285174Z"
updated_at = "2026-05-24T21:32:56.593813Z"
+++

## Spec

### Problem

apm-core spawns git subprocesses in six production call sites using .status() instead of .output(). .status() inherits the parent process stderr, so git error messages (e.g. 'fatal: couldn't find remote ref ticket/...') flow directly to whatever terminal or process is running apm-core — including the apm-server terminal when it calls state::transition.

This was discovered via the apm-server showing 'fatal: couldn't find remote ref ticket/<id>-...' in its stderr whenever the web UI triggered a ticket transition for a ticket whose branch did not yet exist on origin. The server calls apm_core::state::transition(..., no_aggressive=false, ...) which, because config.sync.aggressive defaults to true, runs fetch_branch on every transition. fetch_branch uses .status() so git's output leaks uncontrolled.

The six leaking call sites in apm-core/src/git_util.rs:

1. is_ancestor (line ~747): git merge-base --is-ancestor — boolean ancestry check, uses .status()
2. fetch_branch (line ~910): git fetch origin <branch> — the primary culprit; 'fatal: couldn't find remote ref' leaks here
3. push_branch (line ~921): git push origin <branch>:<branch> — auth/network errors leak
4. delete_remote_branch (line ~1005): git push origin --delete <branch> — errors leak
5. merge_into_default silent fetch (line ~1130): let _ = ...fetch origin <default_branch>...status() — result discarded but stderr still leaks
6. merge_into_default abort cleanup (line ~1155): let _ = ...merge --abort...status() — result discarded but stderr still leaks

The word 'fatal' in the output is git's own error vocabulary (git uses it for any non-recoverable git operation). APM already treats fetch failure as recoverable — the transition continues, the error goes into warnings, and the server discards those warnings. The problem is not the handling logic, it is that git's raw message leaks to the terminal before APM gets a chance to reframe it.

Proposed fix: add a private run_status helper to git_util.rs alongside the existing run() function, using .output() to capture stderr rather than inheriting it:

  fn run_status(dir: &Path, args: &[&str]) -> anyhow::Result<()> {
      let out = Command::new("git")
          .current_dir(dir)
          .args(args)
          .output()
          .context("git not found")?;
      if !out.status.success() {
          anyhow::bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
      }
      Ok(())
  }

Replace each .status() call site with run_status (or inline .output() for the is_ancestor boolean check). The two let _ = ... sites still discard the result but stderr is now captured rather than inherited. Error messages for fetch_branch and push_branch also improve: instead of the generic 'git fetch failed', the actual git message is included in the anyhow error.

### Acceptance criteria

- [x] `fetch_branch` does not write to the terminal's stderr when the remote ref does not exist
- [x] `push_branch` does not write to the terminal's stderr when the push fails
- [x] `delete_remote_branch` does not write to the terminal's stderr when the deletion fails
- [x] `is_ancestor` does not write to the terminal's stderr when `git merge-base --is-ancestor` exits non-zero
- [x] The silent fetch in `merge_into_default` does not write to the terminal's stderr
- [x] The merge-abort cleanup in `merge_into_default` does not write to the terminal's stderr
- [x] `fetch_branch` error value includes the raw git error text (not the generic "git fetch failed")
- [x] `push_branch` error value includes the raw git error text (not the generic "git push failed")
- [x] `cargo test --workspace` passes after the change

### Out of scope

- Suppressing stdout from any git subprocess
- Changes to how APM handles or surfaces fetch/push errors (warnings and error propagation behaviour is unchanged)
- New automated tests for stderr suppression (the fix is a structural change at the process-spawn level, not logic that can be asserted with unit tests)
- Any call sites outside `apm-core/src/git_util.rs`

### Approach

The existing `run()` helper at the top of `apm-core/src/git_util.rs` (line 7) already captures stderr via `.output()` and includes it in the returned error. The six leaking call sites each bypass this helper with an inline `Command::new("git").status()` call. The fix is to replace each site with a `run()` call — no new helper required.

Changes are all in `apm-core/src/git_util.rs`:

1. **`is_ancestor` (~line 747)** — Replace the inline `Command::new("git")…status()…map(|s| s.success()).unwrap_or(false)` block with `run(root, &["merge-base", "--is-ancestor", commit, of_ref]).is_ok()`.

2. **`fetch_branch` (~line 909)** — Replace the inline `Command::new("git")…status()` block with `run(root, &["fetch", "origin", branch]).map(|_| ())`. Remove the `std::process::Command` use here (the top-level `Command` import already covers it). The error string changes from `"git fetch failed"` to the actual git stderr text.

3. **`push_branch` (~line 920)** — Same replacement: `run(root, &["push", "origin", &format!("{branch}:{branch}")]).map(|_| ())`. Error string changes from `"git push failed"` to the actual git stderr text.

4. **`delete_remote_branch` (~line 1004)** — Replace with `run(root, &["push", "origin", "--delete", branch]).map(|_| ()).context("git push origin --delete failed")`. Remove the inline `Command` block entirely.

5. **`merge_into_default` silent fetch (~line 1130)** — Replace `let _ = std::process::Command::new("git")…status();` with `let _ = run(root, &["fetch", "origin", default_branch]);`.

6. **`merge_into_default` merge-abort cleanup (~line 1155)** — Replace `let _ = std::process::Command::new("git")…status();` with `let _ = run(&merge_dir, &["merge", "--abort"]);`.

After these replacements, verify that the remaining `std::process::Command` uses in the file are intentional (they are: `merge_into_default` uses `.output()` for the merge command itself, `pull_default` uses `.output()`, and `push_branch_tracking` already uses `.output()`). No call site is left using `.status()`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T21:19Z | — | new | philippepascal |
| 2026-05-24T21:22Z | new | groomed | philippepascal |
| 2026-05-24T21:22Z | groomed | in_design | philippepascal |
| 2026-05-24T21:23Z | in_design | specd | claude |
| 2026-05-24T21:27Z | specd | ready | philippepascal |
| 2026-05-24T21:27Z | ready | in_progress | philippepascal |
| 2026-05-24T21:32Z | in_progress | implemented | claude |
