+++
id = "c8041bff"
title = "apm-server tests fail in CI due to missing apm-ui/dist"
state = "implemented"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
branch = "ticket/c8041bff-apm-server-tests-fail-in-ci-due-to-missi"
created_at = "2026-04-07T00:22:33.027201Z"
updated_at = "2026-04-07T04:54:54.783098Z"
+++

## Spec

### Problem

cargo test --workspace fails for apm-server because include_dir!("apm-ui/dist") panics at compile time when the UI has not been built. This is pre-existing and unrelated to any recent changes — confirmed by testing on main branch without worktree modifications. The UI dist directory must be built (npm/vite build) before apm-server tests can run.

### Acceptance criteria

- [x] A GitHub Actions workflow runs on every push and pull request to the default branch
- [x] That workflow builds the apm-ui assets (npm ci && npm run build) before invoking cargo test --workspace
- [x] cargo test --workspace passes in that workflow when no worktree modifications have been made
- [x] The workflow reports a clear failure (not a cryptic compile-time panic) if the UI build step itself fails
- [x] The existing release.yml workflow is unchanged and continues to pass

### Out of scope

- Making cargo test work without the UI built in local dev environments (no build.rs, no feature flags)
- Adding a build.rs script to auto-build the UI during Rust compilation
- Changes to how apm-server embeds the UI at compile time (the include_dir! macro stays as-is)
- Modifications to release.yml
- UI build caching or artifact reuse between CI jobs

### Approach

Add .github/workflows/ci.yml — a new workflow triggered on every push and pull_request to the default branch. The workflow has a single job with these steps:

1. actions/checkout@v4
2. actions/setup-node@v4 with Node 20 (matching release.yml)
3. dtolnay/rust-toolchain@stable (matching release.yml)
4. Build the UI: npm ci then npm run build in apm-ui/
5. Run tests: cargo test --workspace

Key constraints:
- Mirror the Node/Rust versions from release.yml exactly to avoid version skew
- Single Ubuntu runner, no matrix — correctness check only, not cross-compilation
- No separate build/test jobs; keep it flat like the release workflow pattern
- No other files change — the root cause (compile-time panic without dist/) is not being fixed structurally; the workflow simply pre-builds the UI, which is already the pattern in release.yml

File to create: .github/workflows/ci.yml

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T00:22Z | — | new | philippepascal |
| 2026-04-07T01:17Z | new | groomed | apm |
| 2026-04-07T03:05Z | groomed | in_design | philippepascal |
| 2026-04-07T04:51Z | in_design | specd | claude-0406-fix-stuck |
| 2026-04-07T04:53Z | specd | ready | apm |
| 2026-04-07T04:53Z | ready | in_progress | philippepascal |
| 2026-04-07T04:54Z | in_progress | implemented | claude-0407-0454-5dd0 |
