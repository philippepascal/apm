+++
id = "527c8480"
title = "apm init should resolve username from gh when available"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/527c8480-apm-init-should-resolve-username-from-gh"
created_at = "2026-04-24T06:27:54.558050Z"
updated_at = "2026-04-24T07:13:59.162521Z"
+++

## Spec

### Problem

apm init calls prompt_username() in apm/src/cmd/init.rs:27-31 on first run even when gh is authenticated, because has_git_host is only true after .apm/config.toml exists. Expected: when gh auth status succeeds, default the prompt to the output of: gh api user -q .login (Enter to accept or override); fall back to blank-default only when gh is unavailable or unauthenticated.

### Acceptance criteria

- [ ] When `gh` is authenticated, `apm init` prompts `Username [<gh-login>]:` where `<gh-login>` is the value returned by `gh api user -q .login`
- [ ] Pressing Enter at the prompt when a gh-supplied default is shown accepts that default and writes it to `.apm/local.toml`
- [ ] Typing a value at the prompt overrides the gh default; the typed value is written to `.apm/local.toml`
- [ ] When `gh` is not installed, the prompt falls back to `Username []:` and blank-default behaviour is unchanged
- [ ] When `gh api user -q .login` exits non-zero (unauthenticated or API error), the prompt falls back to `Username []:` and blank-default behaviour is unchanged
- [ ] When `gh api user -q .login` exits zero but returns an empty string, the prompt falls back to `Username []:`
- [ ] All existing conditions that skip the prompt entirely — `has_git_host` true, not a TTY, `.apm/local.toml` already exists — are unaffected by this change

### Out of scope

- Token-based GitHub auth fallback (fetch_authenticated_user) — not relevant when gh CLI is unavailable at init time\n- GitLab, Bitbucket, or any non-GitHub git host provider\n- Changing behaviour when has_git_host is true (identity already comes from the provider in that path)\n- Auto-accepting the gh username without prompting (user must always have the chance to override)

### Approach

**File changed:** `apm/src/cmd/init.rs` only. `apm_core::github::gh_username()` already exists in `apm-core/src/github.rs` and does exactly what is needed — no new helpers required.

1. Change `prompt_username()` signature to accept an optional default:
   ```rust
   fn prompt_username(default: Option<&str>) -> Result<String>
   ```

2. Update the prompt text inside `prompt_username`:
   - If `default.is_some()` → print `Username [{}]: ` filled with the default value
   - Otherwise → print `Username []: ` (current behaviour)

3. After reading the line, if the trimmed input is empty and a default was provided, return `default.to_string()`; otherwise return the trimmed input as before.

4. At the call site (currently line 28), resolve the gh default before calling the prompt:
   ```rust
   let gh_default = apm_core::github::gh_username();
   let username = prompt_username(gh_default.as_deref())?;
   ```
   This call is already guarded by `!has_git_host && !local_toml.exists() && is_tty`, so no additional guard is needed.

**Constraint:** `apm_core` is already a dependency of the `apm` binary crate, and `github::gh_username` is `pub`, so no new imports or feature flags are required.

**No changes** to `apm-core/src/init.rs`, `apm-core/src/github.rs`, or any config logic — the username value flows through the existing path unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:27Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:13Z | groomed | in_design | philippepascal |