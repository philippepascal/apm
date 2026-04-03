+++
id = "79326024"
title = "apm init: username prompt, local.toml, gitignore, and collaborators bootstrap"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "apm"
agent = "22793"
branch = "ticket/79326024-apm-init-username-prompt-local-toml-giti"
created_at = "2026-04-02T20:53:51.576153Z"
updated_at = "2026-04-03T23:42:31.712804Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["4cec7a17"]
+++

## Spec

### Problem

The `apm init` command does not prompt for a username, does not write `.apm/local.toml`, does not add `.apm/local.toml` to `.gitignore`, and does not seed a `collaborators` list in `.apm/config.toml`. As a result, a freshly initialised project has no local identity configured and no collaborator roster to build from.

Per DESIGN-users.md points 1 and 7, each machine should carry a gitignored `.apm/local.toml` with the developer's `username`, the tracked `.apm/config.toml` should include a `collaborators` list seeded with that username, and `.apm/local.toml` must be excluded from version control so it is never accidentally committed.

Ticket #4cec7a17 (dependency) adds the `LocalConfig` struct, the `collaborators` field on `ProjectConfig`, and `resolve_identity()` to `apm-core`. This ticket wires those building blocks into the `apm init` flow: the interactive prompt, the file writes, and the gitignore update.

### Acceptance criteria

- [ ] After `apm init` in an interactive TTY, `.apm/local.toml` is created containing `username = "<entered-value>"`
- [ ] After `apm init` in a non-interactive environment (no TTY), `.apm/local.toml` is not created
- [ ] After `apm init`, `.gitignore` contains the entry `.apm/local.toml`
- [ ] After `apm init` in a TTY on a fresh project (config.toml not yet written), `.apm/config.toml` `[project]` section contains `collaborators = ["<username>"]`
- [ ] After `apm init` in a non-TTY on a fresh project, `collaborators` in the written config defaults to an empty array `[]` (field is present but empty)
- [ ] Running `apm init` a second time when `.apm/local.toml` already exists does not overwrite or re-prompt for username
- [ ] Running `apm init` a second time does not duplicate the `.apm/local.toml` entry in `.gitignore`
- [ ] `ensure_gitignore` is idempotent: calling it twice produces the same `.gitignore` content

### Out of scope

- `LocalConfig`, `resolve_identity()`, and `ProjectConfig.collaborators` data structures — those are implemented in ticket #4cec7a17
- Dropping `agent` from frontmatter writes — also ticket #4cec7a17
- Using `resolve_identity()` in `apm new` — also ticket #4cec7a17
- Git host plugin (GitHub API) identity resolution — DESIGN-users.md point 4, future ticket
- Validating username against the collaborators list at `apm new` time — deferred, warn-only per design doc
- WebAuthn / OTP / `apm register` / `apm sessions` / `apm revoke` — DESIGN-users.md point 5, separate tickets
- `apm list --mine` / `--author` / `--unassigned` filter additions — DESIGN-users.md point 7, separate ticket
- `/api/me` endpoint and UI author filter — DESIGN-users.md point 8, separate ticket
- Manually adding collaborators to an existing project (post-init) — out of scope for init bootstrapping

### Approach

All changes are confined to `apm-core/src/init.rs`. The building blocks (`LocalConfig`, `write_local_config`, `collaborators` on `ProjectConfig`) are provided by ticket #4cec7a17 and assumed available.

**1. `ensure_gitignore` — add `.apm/local.toml` entry**

Extend the `entries` array in `ensure_gitignore` from `["tickets/NEXT_ID"]` to `["tickets/NEXT_ID", ".apm/local.toml"]`. The existing idempotency logic (check-before-append) handles both entries uniformly. No other changes needed to this function.

**2. `prompt_username` helper**

Add a `fn prompt_username() -> Result<String>` that prints `Username []: ` to stdout, reads a line from stdin, and returns the trimmed string (may be empty). Mirrors the existing `prompt_project_info` pattern.

**3. `write_local_toml` helper**

Add a `fn write_local_toml(apm_dir: &Path, username: &str) -> Result<()>` that writes `.apm/local.toml` with content `username = "<username>"`. Only writes if the file does not already exist (idempotency guard: check `!local_toml_path.exists()` before writing).

**4. `default_config` — optional collaborators parameter**

Add `collaborators: &[&str]` parameter to `default_config`. When non-empty, append `collaborators = ["..."]` to the `[project]` section. When empty, still emit `collaborators = []`. This keeps the field discoverable in all generated configs.

**5. `setup` — orchestrate new steps**

In `setup()`, make two changes:

*Username and local.toml (TTY path only):* After the existing `prompt_project_info` block, add a username prompt guarded by `stdin().is_terminal() && !local_toml.exists()`. If the user enters a non-empty value, call `write_local_toml`. Store the username for use in step below.

*Pass username to config generation:* Change the call to `default_config` to pass the username as a single-element slice when present, or `&[]` when absent (non-TTY or empty input).

The non-TTY path skips username entirely and passes `&[]` to `default_config`, producing `collaborators = []` in the written config.

**6. Tests (in `apm-core/src/init.rs`)**

- Extend `ensure_gitignore_creates_file` and `ensure_gitignore_appends_missing_entry` to also assert `.apm/local.toml` appears in the output.
- Add `write_local_toml_creates_file`: call the new helper, assert file contains `username = "alice"`.
- Add `write_local_toml_idempotent`: call twice; assert the file is not overwritten on the second call.
- Add `setup_non_tty_no_local_toml`: call `setup()` in non-TTY context (matches existing non-TTY test setup); assert `.apm/local.toml` does NOT exist.
- Add `default_config_with_collaborators`: call `default_config` with `collaborators = &["alice"]`; parse output as TOML; assert `project.collaborators == ["alice"]`.
- Add `default_config_empty_collaborators`: call with `&[]`; assert `project.collaborators == []`.

### Open questions


### Amendment requests

- [ ] `.apm/local.toml` is already in `.gitignore` and `ensure_gitignore` already handles it (added by e1582fd0). Remove the gitignore-related AC items and approach steps — they are already shipped.
- [ ] `LocalConfig` already exists and is loaded by `Config::load` (added by e1582fd0). The approach should use the existing struct and load path, not create new ones. `write_local_toml` writes the file; the existing `Config::load` merge logic will pick it up.
- [ ] The approach step "Extend the `entries` array in `ensure_gitignore` from `["tickets/NEXT_ID"]` to include `.apm/local.toml`" is already done — the current array is `["tickets/NEXT_ID", ".apm/local.toml", ".apm/*.init"]`.
- [ ] Set effort and risk to non-zero values.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:53Z | — | new | apm |
| 2026-04-02T23:19Z | new | groomed | apm |
| 2026-04-02T23:26Z | groomed | in_design | philippepascal |
| 2026-04-02T23:29Z | in_design | specd | claude-0402-2330-b7f2 |
| 2026-04-03T23:42Z | specd | ammend | apm |
| 2026-04-03T23:42Z | ammend | in_design | philippepascal |