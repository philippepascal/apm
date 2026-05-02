+++
id = "a0171e83"
title = "Migrate epic-related test setups to init_repo() + real apm epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a0171e83-migrate-epic-related-test-setups-to-init"
created_at = "2026-05-01T20:27:07.814641Z"
updated_at = "2026-05-02T03:56:54.108340Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Four setup helpers in `apm/tests/integration.rs` build their fixture repos without calling `apm init`, so changes to the production init template — default workflow states, config file layout, `.gitignore` entries — are invisible to the 9 tests that depend on them.

- **`setup_with_epic()` (line 2535)** delegates to `setup()`, which hand-writes `apm.toml` at the repo root. The epic branch is created with raw `git checkout -b epic/<id>-…` calls.
- **`setup_with_epic_for_owner_tests()` (line 5460)** is a thin wrapper around `setup_with_epic()` that adds `.apm/local.toml`; it inherits the same problem.
- **`setup_epic_list()` (line 4311)** and **`setup_epic_show()` (line 4431)** each build a fresh tempdir, write a hard-coded 3-state `apm.toml` (ready / implemented / closed), and commit it — never calling `apm init`.

In all four cases the fixture diverges from what real users get: the config lives at the legacy `apm.toml` path instead of `.apm/config.toml`, the workflow is a frozen subset of the production default, and any addition of a required field or state to the init template will not surface in these tests.

The desired state is that all four helpers use `init_repo()` for their repo scaffolding. Because `apm epic new` requires a remote origin (it runs `git fetch` and pushes the new branch), direct epic branch creation via git must remain — but must be marked `// BYPASS:` per the epic's bypass policy, making the workaround explicit and searchable.

### Acceptance criteria

- [ ] `setup_with_epic()` calls `init_repo()` instead of `setup()`
- [ ] `setup_epic_list()` body is replaced entirely by `init_repo()`
- [ ] `setup_epic_show()` body is replaced entirely by `init_repo()`
- [ ] `setup_with_epic_for_owner_tests()` requires no structural change: it inherits `init_repo()` via its call to `setup_with_epic()`
- [ ] The inline epic branch creation inside `setup_with_epic()` carries a `// BYPASS: apm epic new requires a remote origin` comment
- [ ] `create_epic_branch()` carries a `// BYPASS: apm epic new requires a remote origin` comment at its first git line
- [ ] No `apm.toml` hand-write or `git init` call remains in any of the four helpers
- [ ] All 9 tests that call these helpers pass: `new_epic_sets_frontmatter_fields`, `new_epic_branch_created_from_epic_tip`, `epic_list_no_epics_exits_zero_no_output`, `epic_list_shows_epics_with_derived_state_and_counts`, `epic_show_displays_header_and_ticket_table`, `epic_show_prefix_resolves_correctly`, `epic_bulk_owner_change_succeeds`, `epic_bulk_owner_change_skips_closed`, `epic_bulk_owner_change_blocked_non_owner`

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:56Z | groomed | in_design | philippepascal |