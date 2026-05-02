+++
id = "40fdde3b"
title = "Drop apm.toml legacy fallback from Config::load"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/40fdde3b-drop-apm-toml-legacy-fallback-from-confi"
created_at = "2026-05-01T20:27:33.796162Z"
updated_at = "2026-05-02T04:38:13.572145Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["dac20967", "5c494a5d", "296c1061", "c148f904", "f701ef81", "4abc535a", "cc154ee4", "a0171e83", "464d67d5", "094838b6", "443a1840", "059e2e74"]
+++

## Spec

### Problem

apm-core/src/config.rs:644-648 falls back to apm.toml at repo root when .apm/config.toml is missing. The fallback exists only for tests that hand-write apm.toml; once the integration-test migration is complete (sibling tickets in this epic), no production code path or test relies on it. Remove the fallback path. Failure mode after removal should be a clear error (config not found, run `apm init`). Verify by running the full test suite — any test that breaks indicates a sibling migration was incomplete. This ticket should be done last in the epic.

### Acceptance criteria

- [ ] cargo test passes with zero failures after all sibling epic tickets are merged\n- [ ] Config::load in apm-core/src/config.rs does not reference repo_root/apm.toml; path is always .apm/config.toml\n- [ ] When .apm/config.toml is absent, Config::load returns an error whose message contains the phrase apm init\n- [ ] apply_config_migration_fixes in apm/src/cmd/validate.rs does not check apm.toml; it returns Ok(false) immediately when .apm/config.toml is absent\n- [ ] apm-core/src/validate.rs setup_verify_repo writes .apm/config.toml, not apm.toml\n- [ ] apm-core/tests/ticket_create.rs setup writes .apm/config.toml, not apm.toml\n- [ ] apm-core/src/context.rs test inline write targets .apm/config.toml, not apm.toml\n- [ ] apm/tests/e2e.rs second setup helper writes .apm/config.toml, not apm.toml\n- [ ] No non-test Rust source file references apm.toml as a runtime config path (error messages and help text updated to name .apm/config.toml)\n- [ ] apm init --migrate still works: running it on a repo with a root-level apm.toml moves the file to .apm/config.toml

### Out of scope

- Removing the apm init --migrate path (init.rs lines 156-169); that still moves apm.toml to .apm/config.toml for real users migrating old repos\n- Changing any test behaviour — only fixture setup code changes\n- Adding new apm commands\n- Migrating the integration.rs helpers already covered by sibling tickets (dac20967, 5c494a5d, 296c1061, c148f904, f701ef81, 4abc535a, cc154ee4, a0171e83, 464d67d5, 094838b6, 443a1840, 059e2e74)\n- The e2e.rs first setup (lines 43-115) that writes apm.toml then calls apm init; that is testing migration and remains valid

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:38Z | groomed | in_design | philippepascal |