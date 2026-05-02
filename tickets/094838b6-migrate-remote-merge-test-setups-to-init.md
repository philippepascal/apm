+++
id = "094838b6"
title = "Migrate remote-merge test setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/094838b6-migrate-remote-merge-test-setups-to-init"
created_at = "2026-05-01T20:27:20.736073Z"
updated_at = "2026-05-02T04:05:51.594813Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Three setup helpers in `apm/tests/integration.rs` back the merge-strategy integration tests:
`setup_squash_remote` (line 3914), `setup_pr_or_epic_merge_remote` (line 4710), and
`setup_merge_strategy_remote` (line 5301). All three follow the same bare-remote + local-clone
pattern, but each hand-writes a minimal `apm.toml` at repo root and never calls `apm init`.

As a result each fixture diverges from a real user repo in two ways:

1. Config is written to the legacy `apm.toml` location instead of `.apm/config.toml` /
   `.apm/workflow.toml`.
2. The hand-crafted workflow states are a small, frozen subset of the production default
   (3–4 states vs. the 12-state production default).

Any change to the init template — new required states, renamed fields, config file layout —
is invisible to the 4 tests backed by these helpers. The helpers should instead call the real
`apm init` binary (same pattern established by dependency ticket 795dce11's `init_repo()`
helper) and override only what the test needs via real commands or marked bypass.

### Acceptance criteria

- [ ] `setup_squash_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] `setup_pr_or_epic_merge_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] `setup_merge_strategy_remote()` body contains no `std::fs::write` call that writes `apm.toml`
- [ ] A new private helper `init_remote_repo() -> (TempDir, TempDir)` exists that creates a bare remote + local clone via `apm init --no-claude --quiet`
- [ ] `squash_merge_config()`, `pr_or_epic_merge_config_toml()`, and `merge_strategy_config_toml()` are removed (they become dead code after migration)
- [ ] The squash-merge detection tests `sync_detect_squash_merged_branch_remote_ref_present` and `sync_detect_squash_merged_branch_remote_ref_deleted` pass
- [ ] The pr_or_epic_merge tests `pr_or_epic_merge_with_target_branch_merges_into_target`, `pr_or_epic_merge_without_target_branch_attempts_pr`, and `pr_or_epic_merge_with_target_branch_pushes_target_to_origin` pass
- [ ] The merge-strategy test `merge_strategy_merges_locally_without_push` passes
- [ ] `setup_merge_strategy_remote()` includes a `// BYPASS:` comment explaining why `.apm/workflow.toml` is edited directly
- [ ] The bypass in `setup_merge_strategy_remote()` edits `.apm/workflow.toml` (not `apm.toml`)

### Out of scope

- Migrating `write_implemented_ticket` or `write_in_progress_ticket` to use `apm new` / `apm state` — those are ticket-content helpers covered by sibling ticket 059e2e74
- Adding an `apm` CLI command to configure completion strategy on a specific transition — product feature decision, not in scope here
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Migrating any other setup helper (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, etc.) — each has its own sibling ticket in this epic
- Changing any test function body (only the three setup helper bodies are in scope)
- Migrating `push_to_origin()`, `remote_ref_sha()`, or `local_ref_sha()` — those are support utilities, not config-carrying setup helpers
- Adding the `init_repo()` helper (covered by dependency ticket 795dce11); `init_remote_repo()` follows the same pattern but is a separate bare+clone variant

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
| 2026-05-02T04:05Z | groomed | in_design | philippepascal |