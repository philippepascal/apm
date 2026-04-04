+++
id = "66c5bf6b"
title = "GitHub plugin: config schema, token in local.toml, API identity and collaborators sync"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/66c5bf6b-github-plugin-config-schema-token-in-loc"
created_at = "2026-04-02T20:54:29.742423Z"
updated_at = "2026-04-04T06:01:21.758563Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

When a repo is hosted on GitHub, APM currently requires users to manually
configure their identity in `.apm/local.toml` (`username`) and maintain the
`collaborators` list in `.apm/config.toml` by hand. This is error-prone and
creates drift whenever team membership changes on GitHub.

DESIGN-users.md (point 4) specifies an optional GitHub plugin that solves
both problems: the current user's identity is resolved via `GET /user` using a
stored token, and the collaborators list is synced from
`GET /repos/{owner}/{repo}/collaborators`. When the plugin is not configured,
the system falls back to the manual approach introduced by ticket 4cec7a17.

This ticket implements the plugin foundation: the `[git_host]` config schema,
`github_token` storage in `.apm/local.toml`, and the two API resolution paths
wired into `resolve_identity()` and a new `resolve_collaborators()` helper.

### Acceptance criteria

- [x] `.apm/config.toml` containing `[git_host]` with `provider = "github"` and `repo = "owner/name"` parses into `Config` without error
- [x] A config with no `[git_host]` section parses without error (plugin is optional)
- [x] `LocalConfig` accepts an optional `github_token` field; a `local.toml` without it parses without error
- [x] `resolve_identity()` returns the GitHub login when `[git_host]` is configured and a token is available (via `local.toml` or `GITHUB_TOKEN` env var)
- [x] `resolve_identity()` falls back to the `local.toml` `username` field when the GitHub plugin is not configured
- [x] `resolve_identity()` returns `"unassigned"` when neither GitHub plugin nor `local.toml` username is set
- [x] `resolve_identity()` falls back gracefully (continues to `local.toml` / `"unassigned"`) when the GitHub API returns an error or is unreachable
- [x] `resolve_collaborators()` returns the list of GitHub logins from the collaborators API when `[git_host]` is configured and a token is available
- [x] `resolve_collaborators()` falls back to the static `collaborators` list from `config.toml` when the GitHub plugin is not configured
- [x] `resolve_collaborators()` falls back gracefully to the static list when the GitHub API returns an error or is unreachable

### Out of scope

- GitLab and Gitea provider implementations (schema can accommodate them, but only GitHub is wired up)
- `apm init` interactively prompting for a GitHub token (users write `github_token` to `local.toml` manually or set `GITHUB_TOKEN` env var)
- Caching or TTL for GitHub API responses
- Any new `apm` CLI subcommands (`whoami`, `collaborators list`, etc.)
- Webhook integration, PR sync, or any use of the existing `[provider]` section
- Validating `author` against the collaborators list at `apm new` time
- UI or server changes
- Rewriting existing ticket files to update `author` fields

### Approach

**Prerequisite:** Ticket 4cec7a17 must be merged first. It introduces
`LocalConfig` (with `username`), `resolve_identity()`, and the `collaborators`
field on `ProjectConfig`. This ticket extends those.

---

**1. Add `[git_host]` to `apm-core/src/config.rs`**

New struct (alongside existing config structs):

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct GitHostConfig {
    pub provider: Option<String>,  // "github" | future: "gitlab" | "gitea"
    pub repo: Option<String>,      // "owner/name"
}
```

Add field to root `Config`:

```rust
pub git_host: GitHostConfig,
```

`serde(default)` ensures configs without `[git_host]` parse cleanly.

---

**2. Extend `LocalConfig` in `apm-core/src/config.rs`**

Add to the struct introduced by ticket 4cec7a17:

```rust
pub github_token: Option<String>,
```

Token resolution order for GitHub API calls (private helper `effective_github_token`):
1. `local.toml` `github_token` field
2. `GITHUB_TOKEN` environment variable

---

**3. Add `apm-core/src/github.rs`**

Two public blocking functions (no async — `apm-core` is used by the CLI which is sync):

```rust
pub fn fetch_authenticated_user(token: &str) -> anyhow::Result<String>
// GET https://api.github.com/user → returns the `login` field

pub fn fetch_repo_collaborators(token: &str, repo: &str) -> anyhow::Result<Vec<String>>
// GET https://api.github.com/repos/{repo}/collaborators → returns all `login` fields
// First page only (default 30 results) — sufficient for typical team sizes
```

Headers for both calls:
- `Authorization: Bearer {token}`
- `Accept: application/vnd.github+json`
- `User-Agent: apm`

**Cargo dependency:** Add to `apm-core/Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

And to `[workspace.dependencies]` in root `Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

---

**4. Update `resolve_identity()` in `apm-core/src/config.rs`**

After ticket 4cec7a17 the signature is:
`pub fn resolve_identity(repo_root: &Path) -> String`

Extend to check GitHub plugin first:

```
load LocalConfig from local.toml
load Config from .apm/config.toml

if config.git_host.provider == Some("github") {
    if let Some(token) = effective_github_token(&local) {
        match github::fetch_authenticated_user(&token) {
            Ok(login) => return login,
            Err(e) => eprintln!("apm: GitHub identity fetch failed: {e}"),
        }
    }
}

// existing fallback: local.username → "unassigned"
```

Use `eprintln!` for warnings (no new logging dependency needed).

---

**5. Add `resolve_collaborators()` in `apm-core/src/config.rs`**

New public function:

```rust
pub fn resolve_collaborators(config: &Config, local: &LocalConfig) -> Vec<String>
```

Logic mirrors `resolve_identity()`:

```
if config.git_host.provider == Some("github") {
    if let Some(repo) = &config.git_host.repo {
        if let Some(token) = effective_github_token(local) {
            match github::fetch_repo_collaborators(&token, repo) {
                Ok(logins) => return logins,
                Err(e) => eprintln!("apm: GitHub collaborators fetch failed: {e}"),
            }
        }
    }
}
return config.project.collaborators.clone()
```

---

**6. Tests**

Unit tests in `apm-core/src/config.rs`:
- Parse a `Config` TOML with `[git_host]`; assert `provider` and `repo` fields
- Parse a `Config` TOML without `[git_host]`; assert fields are `None`
- Parse a `LocalConfig` TOML with `github_token`; assert `Some(...)`
- Parse a `LocalConfig` TOML without `github_token`; assert `None`

Unit tests in `apm-core/src/github.rs` (require live token, mark `#[ignore]`):
- `fetch_authenticated_user` with a valid token returns a non-empty login string
- `fetch_repo_collaborators` with a valid token and known repo returns a non-empty list

---

**Order of changes:**
1. Update root `Cargo.toml` and `apm-core/Cargo.toml` to add `reqwest`
2. Add `GitHostConfig` struct and `git_host` field to `Config` in `config.rs`
3. Add `github_token` to `LocalConfig` in `config.rs`
4. Create `apm-core/src/github.rs` with two fetch functions + `#[ignore]` tests
5. Add `mod github;` to `apm-core/src/lib.rs`
6. Update `resolve_identity()` to try GitHub API first
7. Add `resolve_collaborators()` function
8. Run `cargo test --workspace` — all existing tests must pass

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:05Z | groomed | in_design | philippepascal |
| 2026-04-03T00:10Z | in_design | specd | claude-0402-2310-b7f2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:45Z | ready | in_progress | philippepascal |
| 2026-04-04T02:48Z | in_progress | implemented | claude-0403-0245-f7a2 |
| 2026-04-04T06:01Z | implemented | closed | apm-sync |
