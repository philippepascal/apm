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

apm-core/src/config.rs (lines 685-689) falls back to repo_root/apm.toml when .apm/config.toml does not exist. A second fallback exists in apm/src/cmd/validate.rs (lines 21-32) inside apply_config_migration_fixes, which also checks apm.toml before .apm/config.toml.\n\nBoth fallbacks were introduced to keep tests working while they still hand-wrote apm.toml instead of calling apm init. The sibling tickets in this epic migrate all of those tests. Once they are merged, no production user or test should rely on the fallback; it becomes dead code that silently hides migration bugs and lets hand-crafted fixtures drift from the real repo shape.\n\nAfter this ticket, .apm/config.toml produced by apm init is the only config location Config::load accepts. A missing config returns a clear error directing the user to run apm init.\n\nFour non-integration test files outside the sibling tickets scope still write apm.toml directly and will break when the fallback is removed:\n- apm-core/src/validate.rs test module (setup_verify_repo)\n- apm-core/tests/ticket_create.rs (setup function)\n- apm-core/src/context.rs test module (inline write before Config::load)\n- apm/tests/e2e.rs second setup helper (~line 590, does not call apm init)\n\nThese are in scope for this ticket. Several error messages and help strings in production code also reference apm.toml as the config path; updating them is cosmetic cleanup that belongs in this same pass.

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