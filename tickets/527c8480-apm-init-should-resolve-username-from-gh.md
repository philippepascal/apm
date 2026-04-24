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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:27Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:13Z | groomed | in_design | philippepascal |