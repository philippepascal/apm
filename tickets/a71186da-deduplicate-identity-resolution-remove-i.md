+++
id = "a71186da"
title = "Deduplicate identity resolution: remove identity.rs, use config.rs"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
branch = "ticket/a71186da-deduplicate-identity-resolution-remove-i"
created_at = "2026-04-07T22:30:44.747975Z"
updated_at = "2026-04-08T00:27:39.593260Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Two modules in `apm-core` independently resolve the current user's identity with overlapping logic that has drifted apart:

- `identity.rs::resolve_current_user()` (69 lines) — reads `.apm/local.toml` username, checks `git_host` config, calls `gh_username()` for GitHub, falls back to the literal string `"apm"`.
- `config.rs::resolve_identity()` (~37 lines) — reads `.apm/local.toml` username, checks `git_host` config, calls `gh_username()` then `fetch_authenticated_user()` with token resolution, falls back to `"unassigned"`.

The split happened because identity resolution evolved: `config.rs` added fuller GitHub API token support when `git_host` landed, but `identity.rs` was never removed or updated to delegate. Neither function calls the other.

Callers are split across the CLI and server layers. `apm new` and `apm list --mine` (in the `apm` crate) use `identity::resolve_current_user()`. The `apm-server` handlers (sync, POST /api/tickets, GET /api/me, queue access control) use `config::resolve_identity()`. This means a user who has no identity configured gets `author = "apm"` on tickets created via the CLI but `"unassigned"` when created via the server — the same user can appear under two different identities in the same project.

The desired state is a single identity resolution function (`config::resolve_identity()`) used by all callers. The `identity.rs` module and its `pub mod identity` declaration should be removed entirely.

### Acceptance criteria

- [x] `apm-core/src/identity.rs` does not exist in the repository
- [x] `pub mod identity` is not declared anywhere in `apm-core/src/lib.rs`
- [x] `apm new` sets the ticket `author` field using `config::resolve_identity()`
- [x] `apm list --mine` filters tickets by the value returned by `config::resolve_identity()`
- [ ] `apm new` with no identity configured (no `local.toml` username, no `git_host`) sets `author = "unassigned"`
- [ ] `apm list --mine` with no identity configured matches tickets whose author is `"unassigned"`
- [ ] `cargo build` completes without errors or unused-import warnings across all workspace crates
- [ ] All `identity::` references are absent from the `apm` crate source
- [ ] The test coverage previously provided by `identity.rs` unit tests is present in `config.rs` or is superseded by existing `resolve_identity` tests

### Out of scope

- Changing the implementation of `config::resolve_identity()` itself (token logic, fallback order, GitHub API calls)
- Migrating existing tickets that currently have `author = "apm"` to `"unassigned"` — old data is not touched
- Adding GitHub API token support to the CLI (`apm new`, `apm list`) — the server already has this
- Changing the `apm-server` callers of `resolve_identity()` — they already use the canonical function
- Renaming `resolve_identity()` to `resolve_current_user()` or any other public API rename
- Adding new identity fallback strategies (e.g. `APM_AGENT_NAME` env var support — not present in either current function)

### Approach

**Files that change:**

- `apm-core/src/identity.rs` — **delete**
- `apm-core/src/lib.rs` — remove `pub mod identity;`
- `apm/src/cmd/new.rs` — remove `identity` from use statement; replace `identity::resolve_current_user(root)` with `config::resolve_identity(root)` (the `apm` crate already imports `apm_core::config::Config`, so adding `resolve_identity` to the use path is straightforward)
- `apm/src/cmd/list.rs` — same: remove `identity` import, update call site
- `apm-core/src/config.rs` — add any unit test cases not already covered by existing `resolve_identity` tests (see Test migration below)

**Fallback value change:** `resolve_current_user()` returned `"apm"` when no identity is configured; `resolve_identity()` returns `"unassigned"`. This is an intentional correctness fix — `"unassigned"` is the canonical sentinel for an unknown author, matching server behaviour. No migration of existing ticket data is required.

**Test migration:** `identity.rs` had four unit tests. Verify which are already covered by the existing `resolve_identity` block in `config.rs`, then add the gaps:

- returns `"unassigned"` when `.apm/local.toml` is absent
- returns the `local.toml` username when present and non-empty
- returns `"unassigned"` when `local.toml` has no `username` key
- returns `"unassigned"` when `username = ""`

**Steps in order:**

1. Delete `apm-core/src/identity.rs`.
2. Remove `pub mod identity;` from `apm-core/src/lib.rs`.
3. Update `apm/src/cmd/new.rs`: remove identity import; call `config::resolve_identity(root)`.
4. Update `apm/src/cmd/list.rs`: same.
5. Add missing test cases to `config.rs` test module.
6. Run `cargo build` and `cargo test -p apm-core` to confirm clean compile and passing tests.

**Constraints:** `config::resolve_identity()` is already `pub`. No other crate besides `apm` and `apm-server` references `identity::resolve_current_user`; `apm-server` already uses `resolve_identity` and is untouched.

### Files that change

| File | Change |
|------|--------|
| `apm-core/src/identity.rs` | **Delete** |
| `apm-core/src/lib.rs` | Remove `pub mod identity;` line |
| `apm/src/cmd/new.rs` | Remove `identity` from use statement; replace `identity::resolve_current_user(root)` with `apm_core::config::resolve_identity(root)` (or adjust use imports accordingly) |
| `apm/src/cmd/list.rs` | Remove `identity` from use statement; replace `identity::resolve_current_user(root)` with `apm_core::config::resolve_identity(root)` |
| `apm-core/src/config.rs` | Add unit tests covering the cases formerly in `identity.rs` that are not already covered by the existing `resolve_identity` tests (see below) |

### Fallback value change

`resolve_current_user()` returned `"apm"` when no identity is configured. `resolve_identity()` returns `"unassigned"`. This is an intentional correctness fix — `"unassigned"` is the canonical sentinel for an unknown author, matching server behaviour. No migration of existing ticket data is required.

### Test migration

`identity.rs` had four unit tests. Check what the existing `resolve_identity` tests in `config.rs` already cover. Add any missing cases to the `#[cfg(test)]` block in `config.rs`:

- `resolve_identity` returns `"unassigned"` when `.apm/local.toml` is absent (analogous to `returns_apm_when_file_absent`)
- `resolve_identity` returns the username from `local.toml` when present and non-empty (analogous to `returns_username_when_present`)
- `resolve_identity` returns `"unassigned"` when `local.toml` exists but has no `username` key (analogous to `returns_apm_when_username_key_absent`)
- `resolve_identity` returns `"unassigned"` when `username = ""` (analogous to `returns_apm_when_username_is_empty`)

If any of these are already covered, skip duplication.

### Steps in order

1. Delete `apm-core/src/identity.rs`.
2. Remove `pub mod identity;` from `apm-core/src/lib.rs`.
3. Update `apm/src/cmd/new.rs`: remove the `identity` import, call `config::resolve_identity(root)` instead.
4. Update `apm/src/cmd/list.rs`: same — remove import, update call site.
5. Add any missing test cases to `config.rs`.
6. Run `cargo build` and `cargo test -p apm-core` to verify no compile errors and all tests pass.

### Constraints

- `config::resolve_identity()` is already `pub` — no visibility change needed.
- The `apm` crate imports `apm_core::config::Config` already; adding `resolve_identity` to that use path is straightforward.
- No other crates outside `apm` and `apm-server` reference `identity::resolve_current_user`; `apm-server` already uses `resolve_identity`, so it is untouched.

### Open questions


### Amendment requests

- [x] Remove duplicated approach content: everything after the first "Constraints" paragraph (the "Files that change" table, "Fallback value change", "Test migration", "Steps in order", and second "Constraints" sections) is a near-verbatim repeat of the earlier approach. Keep only the first pass.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:43Z | new | groomed | apm |
| 2026-04-07T22:49Z | groomed | in_design | philippepascal |
| 2026-04-07T22:53Z | in_design | specd | claude-0407-2249-4438 |
| 2026-04-08T00:03Z | specd | ammend | philippepascal |
| 2026-04-08T00:05Z | ammend | in_design | philippepascal |
| 2026-04-08T00:06Z | in_design | specd | claude-0408-0005-0a88 |
| 2026-04-08T00:20Z | specd | ready | apm |
| 2026-04-08T00:27Z | ready | in_progress | philippepascal |