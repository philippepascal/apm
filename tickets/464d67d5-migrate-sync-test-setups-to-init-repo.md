+++
id = "464d67d5"
title = "Migrate sync test setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/464d67d5-migrate-sync-test-setups-to-init-repo"
created_at = "2026-05-01T20:27:11.656953Z"
updated_at = "2026-05-02T04:03:02.661491Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_sync_repo()` (line 5711) and `setup_branch_in_origin()` (line 5889) in `apm/tests/integration.rs` are the two setup helpers backing the sync integration tests. `setup_sync_repo()` calls `setup()` for the local clone — which hand-writes a minimal `apm.toml` string literal and never invokes `apm init`. `setup_branch_in_origin()` creates a plain local repo containing only a `README` file, also bypassing `apm init` entirely.

Because neither fixture goes through `apm init`, the repos they produce diverge from the shape real users get: the config is at the legacy `apm.toml` root location instead of `.apm/config.toml`, the workflow states are a frozen subset of the production default, and the `.gitignore` and other init-generated files are absent. Changes to the production init template — config file layout, new default states, `.gitignore` entries — are invisible to the 10+ tests that depend on these helpers.

Both helpers should use `init_repo()` for the local clone. The bare-origin creation (`git init --bare`) and the disposable-clone branch-seeding approach have no real-`apm` alternative and should be retained with `// BYPASS:` annotations.

### Acceptance criteria

- [ ] `setup_sync_repo()` calls `init_repo()` instead of `setup()` for the local clone
- [ ] `setup_sync_repo()`'s bare-origin `git init --bare` block is annotated with a `// BYPASS:` comment
- [ ] `setup_branch_in_origin()` calls `init_repo()` instead of the inline `git init` + README block for the local repo
- [ ] `setup_branch_in_origin()`'s bare-origin `git init --bare` block is annotated with a `// BYPASS:` comment
- [ ] `setup_branch_in_origin()`'s disposable-clone branch-seeding block is annotated with a `// BYPASS:` comment
- [ ] `setup_sync_repo()` returns `(TempDir, TempDir)` — signature is unchanged
- [ ] `setup_branch_in_origin()` returns `(TempDir, TempDir, String)` — signature is unchanged
- [ ] All five `sync_main_*` tests pass after migration
- [ ] All `sync_ticket_ref_*` tests pass after migration

### Out of scope

- Migrating `push_to_origin()` or `rev_parse()` — these are support functions, not config-carrying setup helpers
- Migrating any other setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in this epic
- Changing any test function body (only the two helper bodies are in scope)
- Adding an `apm` command to seed branches into a remote or bare origin
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- The `init_repo()` implementation itself — covered by dependency ticket 795dce11

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
| 2026-05-02T04:03Z | groomed | in_design | philippepascal |