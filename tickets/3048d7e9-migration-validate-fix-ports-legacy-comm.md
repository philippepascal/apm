+++
id = "3048d7e9"
title = "Migration: validate --fix ports legacy command/args/model to agent + options"
state = "specd"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3048d7e9-migration-validate-fix-ports-legacy-comm"
created_at = "2026-04-30T20:03:17.277300Z"
updated_at = "2026-04-30T21:42:07.281922Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["6cac8518"]
+++

## Spec

### Problem

Existing APM projects have a `.apm/config.toml` using the legacy `[workers]` shape: `command = "claude"`, `args = ["--print", ...]`, and `model = "sonnet"`. After upgrading to the agent-wrapper architecture (ticket 6cac8518), those projects receive a deprecation warning on every `apm start` invocation but have no automated way to migrate.

The desired state is `agent = "claude"` in `[workers]` with model moved to `[workers.options]` and `args` dropped entirely (the wrapper now owns CLI flag construction). A matching migration must apply to every `[worker_profiles.<X>]` section as well.

This ticket adds that migration to `apm validate --fix`. A developer who upgrades APM runs `apm validate --fix`, sees a one-line confirmation message, and their config is correct without any manual editing. If the project was using a non-Claude command, automated migration is not safe — the tool warns and stops so the user can hand-pick a wrapper.

### Acceptance criteria

- [ ] `apm validate --fix` on a config with `[workers] command = "claude"` rewrites it to `[workers] agent = "claude"` and removes the `command` key
- [ ] `apm validate --fix` on a config with `[workers] model = "sonnet"` moves the value to `[workers.options] model = "sonnet"` and removes the top-level `model` key
- [ ] `apm validate --fix` on a config with `[workers] args = [...]` removes the `args` key regardless of its contents
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] model = "opus"` moves the value to `[worker_profiles.X.options] model = "opus"` and removes the profile-level `model` key
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] command = "claude"` removes the profile-level `command` key (profile inherits `agent` from global)
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] args = [...]` removes the profile-level `args` key
- [ ] `apm validate --fix` on a config where `[workers] command` is anything other than `"claude"` prints a warning naming the offending command and does not modify the config file
- [ ] `apm validate --fix` on a config where any `[worker_profiles.X] command` is anything other than `"claude"` prints a warning naming the profile and command, and does not modify the config file
- [ ] After a successful migration `apm validate` (without `--fix`) exits zero on the rewritten config
- [ ] `apm validate --fix` on a config that has no legacy fields (`agent` already set, no `command`/`args`/`model`) makes no changes to the file
- [ ] `apm validate --fix` on a config with both legacy fields and new fields (e.g. `agent` already present alongside a leftover `model`) removes the legacy fields and leaves the new fields intact
- [ ] A successful migration prints exactly the line: `migrated [workers] config to agent-driven shape; legacy command/args/model removed`
- [ ] TOML comments present in the config file survive the migration unchanged
- [ ] Key ordering of unrelated sections (e.g. `[keychain]`, `[env]`) is preserved after migration

### Out of scope

- An `apm init --migrate` or `apm agents migrate` subcommand — `apm validate --fix` is the canonical migration path
- Migration of `.apm/agents.md`, `.apm/apm.worker.md`, or `.apm/apm.spec-writer.md` — those are prompt content, not config
- Hash-trip integration changes — the existing hash-trip already runs validate on config change; no adjustment needed here
- Removing deprecated `command`/`args`/`model` fields from the Rust structs — that happens after the deprecation window, tracked separately (ticket 6cac8518 retains the fields for backward compatibility)
- `apm validate --fix` for `workflow.toml` files — only `.apm/config.toml` (and `apm.toml` legacy root path) contain worker config
- Rollback or backup of the original config — the caller can use version control
- Migrating a config where `command` is non-Claude — this ticket explicitly stops and warns rather than guessing a wrapper name
- Windows execute-bit semantics or platform-specific config path differences

### Approach

#### Files changed

**`apm/Cargo.toml`** — add `toml_edit` to the `[dependencies]` table (it is already in the workspace Cargo.toml at `toml_edit = "0.22"`; add `toml_edit.workspace = true`).

**`apm/src/cmd/validate.rs`** — add `apply_config_migration_fixes(root: &Path) -> Result<bool>` (returns `true` if any change was written) and call it from `run()` when `fix = true`, before the existing branch/on-failure/merged fix calls. Print the migration message from `run()` after `apply_config_migration_fixes` returns `true`.

No changes to `apm-core/src/` — the migration is a TOML rewrite at the CLI layer, not a semantic config operation.

---

#### `apply_config_migration_fixes(root)` — step by step

**1. Locate config file.**
Check `root/.apm/config.toml` first, then `root/apm.toml`. If neither exists, return `Ok(false)`.

**2. Parse with `toml_edit`.**
`let mut doc = content.parse::<toml_edit::DocumentMut>()?;`

**3. Detect legacy fields.**
Check `doc["workers"]` for the presence of any of `command`, `args`, `model`. Check each table under `doc["worker_profiles"]` for the same keys. If none are present in any section, return `Ok(false)` (no-op).

**4. Guard: non-claude command.**
If `doc["workers"]["command"]` exists and its string value is not `"claude"`, print:
```
warning: [workers] command = "<value>" is not "claude" — cannot auto-migrate; choose a wrapper manually
```
and return `Ok(false)` without modifying the file.

For each `worker_profiles.<name>` table that has a `command` key whose value is not `"claude"`, print:
```
warning: [worker_profiles.<name>] command = "<value>" is not "claude" — cannot auto-migrate; choose a wrapper manually
```
and return `Ok(false)`.

Only proceed past this point if every `command` field present is exactly `"claude"`.

**5. Migrate `[workers]`.**
- If `workers.command` is present (value must be `"claude"` at this point): remove it, set `workers.agent = "claude"`.
- If `workers.model` is present: read the value, remove the key, set `workers.options.model = <value>`. Create `workers.options` as an inline table if it does not exist.
- If `workers.args` is present: remove the key. No replacement.

**6. Migrate each `[worker_profiles.<name>]`.**
For each profile table:
- If `command` is present (value must be `"claude"`): remove it. Do **not** add `agent` at the profile level — profiles inherit `agent` from `[workers]`.
- If `model` is present: read the value, remove the key, set `profile.options.model = <value>`.
- If `args` is present: remove it.

**7. Write back.**
`fs::write(config_path, doc.to_string())?;`

`toml_edit` preserves comments, key ordering, and whitespace in untouched sections automatically.

**8. Re-validate.**
Call `apm_core::validate::run(root, /*fix=*/false, /*json=*/false, /*config_only=*/true, /*no_aggressive=*/false)`. If it returns an error, surface it as a `bail!` so the user knows the migration produced an invalid config (should not happen in normal cases, but guards against bugs).

---

#### Message output

`apply_config_migration_fixes` returns `Ok(true)` on success. The caller in `run()` prints:
```
migrated [workers] config to agent-driven shape; legacy command/args/model removed
```

---

#### Tests

Add to `apm/tests/validate_fix.rs` (or the existing validate integration test file):

- **`test_fix_migrates_claude_command`** — fixture config with `command = "claude"`, `args = [...]`, `model = "sonnet"` → assert written config has `agent = "claude"`, `options.model = "sonnet"`, no `command`/`args`/`model` keys.
- **`test_fix_noop_on_non_claude_command`** — fixture with `command = "my-ai"` → assert file is unchanged, stderr contains `"cannot auto-migrate"`.
- **`test_fix_noop_on_non_claude_profile_command`** — global command absent, `worker_profiles.impl_agent.command = "my-ai"` → unchanged, warning names the profile.
- **`test_fix_mixed_legacy_and_new_fields`** — fixture has both `agent = "claude"` (already present) and leftover `model = "opus"` → `model` is removed, `agent` preserved, `options.model = "opus"` added.
- **`test_fix_already_migrated_noop`** — fixture with `agent = "claude"`, `[workers.options] model = "sonnet"`, no legacy keys → file content is byte-identical after `--fix`.
- **`test_fix_preserves_comments`** — fixture contains a TOML comment between sections → the comment survives unchanged in the output.
- **`test_fix_profile_model_migration`** — `worker_profiles.spec_agent` has `model = "opus"`, no global model → `worker_profiles.spec_agent.options.model = "opus"` in output, profile `model` key gone.
- **`test_fix_revalidate_passes`** — after migration, `apm_core::validate::run` with `config_only=true` returns `Ok(())`.

Test fixtures are small inline TOML strings written to a `tempdir`; no external fixture files needed.

### Files changed

**`apm/Cargo.toml`** — add `toml_edit` to the `[dependencies]` table (it is already in the workspace Cargo.toml at `toml_edit = "0.22"`; add `toml_edit.workspace = true`).

**`apm/src/cmd/validate.rs`** — add `apply_config_migration_fixes(root: &Path) -> Result<bool>` (returns `true` if any change was written) and call it from `run()` when `fix = true`, before the existing branch/on-failure/merged fix calls. Print the migration message from `run()` after `apply_config_migration_fixes` returns `true`.

No changes to `apm-core/src/` — the migration is a TOML rewrite at the CLI layer, not a semantic config operation.

---

### `apply_config_migration_fixes(root)` — step by step

**1. Locate config file.**
Check `root/.apm/config.toml` first, then `root/apm.toml`. If neither exists, return `Ok(false)`.

**2. Parse with `toml_edit`.**
`let mut doc = content.parse::<toml_edit::DocumentMut>()?;`

**3. Detect legacy fields.**
Check `doc["workers"]` for the presence of any of `command`, `args`, `model`. Check each table under `doc["worker_profiles"]` for the same keys. If none are present in any section, return `Ok(false)` (no-op).

**4. Guard: non-claude command.**
If `doc["workers"]["command"]` exists and its string value is not `"claude"`, print:
```
warning: [workers] command = "<value>" is not "claude" — cannot auto-migrate; choose a wrapper manually
```
and return `Ok(false)` without modifying the file.

For each `worker_profiles.<name>` table that has a `command` key whose value is not `"claude"`, print:
```
warning: [worker_profiles.<name>] command = "<value>" is not "claude" — cannot auto-migrate; choose a wrapper manually
```
and return `Ok(false)`.

Only proceed past this point if every `command` field present is exactly `"claude"`.

**5. Migrate `[workers]`.**
- If `workers.command` is present (value must be `"claude"` at this point): remove it, set `workers.agent = "claude"`.
- If `workers.model` is present: read the value, remove the key, set `workers.options.model = <value>`. Create `workers.options` as an inline table if it does not exist.
- If `workers.args` is present: remove the key. No replacement.

**6. Migrate each `[worker_profiles.<name>]`.**
For each profile table:
- If `command` is present (value must be `"claude"`): remove it. Do **not** add `agent` at the profile level — profiles inherit `agent` from `[workers]`.
- If `model` is present: read the value, remove the key, set `profile.options.model = <value>`.
- If `args` is present: remove it.

**7. Write back.**
`fs::write(config_path, doc.to_string())?;`

`toml_edit` preserves comments, key ordering, and whitespace in untouched sections automatically.

**8. Re-validate.**
Call `apm_core::validate::run(root, /*fix=*/false, /*json=*/false, /*config_only=*/true, /*no_aggressive=*/false)`. If it returns an error, surface it as a `bail!` so the user knows the migration produced an invalid config (should not happen in normal cases, but guards against bugs).

---

### Message output

`apply_config_migration_fixes` returns `Ok(true)` on success. The caller in `run()` prints:
```
migrated [workers] config to agent-driven shape; legacy command/args/model removed
```

---

### Tests

Add to `apm/tests/validate_fix.rs` (or the existing validate integration test file):

- **`test_fix_migrates_claude_command`** — fixture config with `command = "claude"`, `args = [...]`, `model = "sonnet"` → assert written config has `agent = "claude"`, `options.model = "sonnet"`, no `command`/`args`/`model` keys.
- **`test_fix_noop_on_non_claude_command`** — fixture with `command = "my-ai"` → assert file is unchanged, stderr contains `"cannot auto-migrate"`.
- **`test_fix_noop_on_non_claude_profile_command`** — global command absent, `worker_profiles.impl_agent.command = "my-ai"` → unchanged, warning names the profile.
- **`test_fix_mixed_legacy_and_new_fields`** — fixture has both `agent = "claude"` (already present) and leftover `model = "opus"` → `model` is removed, `agent` preserved, `options.model = "opus"` added.
- **`test_fix_already_migrated_noop`** — fixture with `agent = "claude"`, `[workers.options] model = "sonnet"`, no legacy keys → file content is byte-identical after `--fix`.
- **`test_fix_preserves_comments`** — fixture contains a TOML comment between sections → the comment survives unchanged in the output.
- **`test_fix_profile_model_migration`** — `worker_profiles.spec_agent` has `model = "opus"`, no global model → `worker_profiles.spec_agent.options.model = "opus"` in output, profile `model` key gone.
- **`test_fix_revalidate_passes`** — after migration, `apm_core::validate::run` with `config_only=true` returns `Ok(())`.

Test fixtures are small inline TOML strings written to a `tempdir`; no external fixture files needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:36Z | groomed | in_design | philippepascal |
| 2026-04-30T21:42Z | in_design | specd | claude-0430-2136-f2a8 |
