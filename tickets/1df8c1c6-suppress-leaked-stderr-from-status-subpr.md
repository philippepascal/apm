+++
id = "1df8c1c6"
title = "Suppress leaked stderr from .status() subprocess calls in apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1df8c1c6-suppress-leaked-stderr-from-status-subpr"
created_at = "2026-05-24T21:19:59.285174Z"
updated_at = "2026-05-24T21:22:23.024962Z"
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

- [ ] `fetch_branch` does not write to the terminal's stderr when the remote ref does not exist
- [ ] `push_branch` does not write to the terminal's stderr when the push fails
- [ ] `delete_remote_branch` does not write to the terminal's stderr when the deletion fails
- [ ] `is_ancestor` does not write to the terminal's stderr when `git merge-base --is-ancestor` exits non-zero
- [ ] The silent fetch in `merge_into_default` does not write to the terminal's stderr
- [ ] The merge-abort cleanup in `merge_into_default` does not write to the terminal's stderr
- [ ] `fetch_branch` error value includes the raw git error text (not the generic "git fetch failed")
- [ ] `push_branch` error value includes the raw git error text (not the generic "git push failed")
- [ ] `cargo test --workspace` passes after the change

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-24T21:19Z | — | new | philippepascal |
| 2026-05-24T21:22Z | new | groomed | philippepascal |
| 2026-05-24T21:22Z | groomed | in_design | philippepascal |