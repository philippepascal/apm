+++
id = "c8041bff"
title = "apm-server tests fail in CI due to missing apm-ui/dist"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8041bff-apm-server-tests-fail-in-ci-due-to-missi"
created_at = "2026-04-07T00:22:33.027201Z"
updated_at = "2026-04-07T03:05:49.328283Z"
+++

## Spec

### Problem

cargo test --workspace fails for apm-server because include_dir!("apm-ui/dist") panics at compile time when the UI has not been built. This is pre-existing and unrelated to any recent changes — confirmed by testing on main branch without worktree modifications. The UI dist directory must be built (npm/vite build) before apm-server tests can run.

### Acceptance criteria

- [ ] A GitHub Actions workflow runs on every push and pull request to the default branch
- [ ] That workflow builds the apm-ui assets (npm ci && npm run build) before invoking cargo test --workspace
- [ ] cargo test --workspace passes in that workflow when no worktree modifications have been made
- [ ] The workflow reports a clear failure (not a cryptic compile-time panic) if the UI build step itself fails
- [ ] The existing release.yml workflow is unchanged and continues to pass

### Out of scope

- Making cargo test work without the UI built in local dev environments (no build.rs, no feature flags)
- Adding a build.rs script to auto-build the UI during Rust compilation
- Changes to how apm-server embeds the UI at compile time (the include_dir! macro stays as-is)
- Modifications to release.yml
- UI build caching or artifact reuse between CI jobs

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T00:22Z | — | new | philippepascal |
| 2026-04-07T01:17Z | new | groomed | apm |
| 2026-04-07T03:05Z | groomed | in_design | philippepascal |