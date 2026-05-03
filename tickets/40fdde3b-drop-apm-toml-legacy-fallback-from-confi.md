+++
id = "40fdde3b"
title = "Drop apm.toml legacy fallback from Config::load"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/40fdde3b-drop-apm-toml-legacy-fallback-from-confi"
created_at = "2026-05-01T20:27:33.796162Z"
updated_at = "2026-05-03T23:09:50.137567Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["dac20967", "5c494a5d", "296c1061", "c148f904", "f701ef81", "4abc535a", "cc154ee4", "a0171e83", "464d67d5", "094838b6", "443a1840", "059e2e74"]
+++

## Spec

### Problem

`apm-core/src/config.rs` (lines 685–689) falls back to `repo_root/apm.toml` when `.apm/config.toml` does not exist. A second fallback exists in `apm/src/cmd/validate.rs` (lines 21–32) inside `apply_config_migration_fixes`, which also checks `apm.toml` before `.apm/config.toml`.

Both fallbacks were introduced to keep tests working while they still hand-wrote `apm.toml` instead of calling `apm init`. The sibling tickets in this epic migrate all of those tests. Once they are merged, no production user or test should rely on the fallback; it becomes dead code that silently hides migration bugs and lets hand-crafted fixtures drift from the real repo shape.

After this ticket, `.apm/config.toml` produced by `apm init` is the only config location `Config::load` accepts. A missing config returns a clear error directing the user to run `apm init`.

Four non-integration test files outside the sibling tickets' scope still write `apm.toml` directly and will break when the fallback is removed:
- `apm-core/src/validate.rs` test module (`setup_verify_repo`)
- `apm-core/tests/ticket_create.rs` (`setup` function)
- `apm-core/src/context.rs` test module (inline write before `Config::load`)
- `apm/tests/e2e.rs` second setup helper (~line 590, does not call `apm init`)

These are in scope for this ticket. Several error messages and help strings in production code also reference `apm.toml` as the config path; updating them is cosmetic cleanup that belongs in this same pass.

### Acceptance criteria

- [ ] `cargo test` passes with zero failures after all sibling epic tickets are merged
- [ ] `Config::load` in `apm-core/src/config.rs` does not reference `repo_root/apm.toml`; path is always `.apm/config.toml`
- [ ] When `.apm/config.toml` is absent, `Config::load` returns an error whose message contains the phrase `apm init`
- [ ] `apply_config_migration_fixes` in `apm/src/cmd/validate.rs` does not check `apm.toml`; it returns `Ok(false)` immediately when `.apm/config.toml` is absent
- [ ] `apm-core/src/validate.rs` `setup_verify_repo` writes `.apm/config.toml`, not `apm.toml`
- [ ] `apm-core/tests/ticket_create.rs` `setup` writes `.apm/config.toml`, not `apm.toml`
- [ ] `apm-core/src/context.rs` test inline write targets `.apm/config.toml`, not `apm.toml`
- [ ] `apm/tests/e2e.rs` second setup helper writes `.apm/config.toml`, not `apm.toml`
- [ ] No non-test Rust source file references `apm.toml` as a runtime config path (error messages and help text updated to name `.apm/config.toml`)
- [ ] `apm init --migrate` still works: running it on a repo with a root-level `apm.toml` moves the file to `.apm/config.toml`

### Out of scope

- Removing the `apm init --migrate` path (`init.rs` lines 156–169); that still moves `apm.toml` to `.apm/config.toml` for real users migrating old repos
- Changing any test behaviour — only fixture setup code changes
- Adding new apm commands
- Migrating the `integration.rs` helpers already covered by sibling tickets (dac20967, 5c494a5d, 296c1061, c148f904, f701ef81, 4abc535a, cc154ee4, a0171e83, 464d67d5, 094838b6, 443a1840, 059e2e74)
- The `e2e.rs` first setup (lines 43–115) that writes `apm.toml` then calls `apm init`; that is testing migration and remains valid

### Approach

All changes are in Rust source files. Do this ticket last in the epic — run `cargo test` first to confirm all sibling tickets have landed and the suite is green.

**Step 1 — Remove the primary fallback (`apm-core/src/config.rs`, lines 685–689)**

Replace:

    let path = if apm_dir_config.exists() {
        apm_dir_config
    } else {
        repo_root.join("apm.toml")
    };

With:

    let path = apm_dir_config;

Also update the `with_context` message on the `read_to_string` call (line ~691) to include a hint:

    .with_context(|| format!(
        "cannot read {} -- run 'apm init' to initialise this repository",
        path.display()
    ))?;

**Step 2 — Remove the secondary fallback (`apm/src/cmd/validate.rs`, lines 21–32)**

In `apply_config_migration_fixes`, replace the if/else block that tries both `.apm/config.toml` and `apm.toml`:

    let config_path = root.join(".apm").join("config.toml");
    if !config_path.exists() {
        return Ok(false);
    }

The rest of the function (read, parse, rewrite) is unchanged.

**Step 3 — Fix four non-integration test fixtures**

In each of the four helpers listed in the Problem section, change the write target from `p.join("apm.toml")` to `p.join(".apm/config.toml")`. Each needs `create_dir_all(".apm")` before the write. Update `git add` arguments from `"apm.toml"` to `".apm/config.toml"` where the file is committed.

`apm-core/src/validate.rs`, `setup_verify_repo` (~line 644):
- `create_dir_all(p.join(".apm")).unwrap();`
- write to `p.join(".apm/config.toml")`
- `git_cmd add ".apm/config.toml"`

`apm-core/tests/ticket_create.rs`, `setup` (~line 22):
- `create_dir_all(p.join(".apm")).unwrap();`
- write to `p.join(".apm/config.toml")`
- `git add ".apm/config.toml"`

`apm-core/src/context.rs`, inline test write (~line 386):
- `create_dir_all(p.join(".apm")).unwrap();`
- write to `p.join(".apm/config.toml")`
- no git commit needed (that test does not commit)

`apm/tests/e2e.rs`, second setup helper (~line 590):
- `create_dir_all(p.join(".apm")).unwrap();`
- write to `p.join(".apm/config.toml")`
- update `git add` to `".apm/config.toml"`

Leave the first setup (~lines 43–115) unchanged — it writes `apm.toml` then calls `apm init`, specifically testing the migration path, and stays valid.

**Step 4 — Update error messages and help text**

- `apm-core/src/start.rs` (~line 173): change `"apm.toml [workers] section"` to `".apm/config.toml [workers] section"`
- `apm/src/cmd/new.rs` (~line 34): change `"disabled in apm.toml"` to `"disabled in .apm/config.toml"`
- `apm/src/main.rs` (~line 272, 276): change `".apm/apm.toml"` and `"apm.toml"` to `".apm/workflow.toml"` (these describe workflow transitions)
- `apm/src/main.rs` (~line 508): change `"Validate apm.toml correctness"` to `"Validate .apm/config.toml correctness"`
- `apm/src/main.rs` (~line 745): change `"defined in apm.toml"` to `"defined in .apm/ticket.toml"`

**Step 5 — Verify**

`cargo test` must exit 0. Any failure indicates a sibling migration is incomplete; fix the missed fixture rather than re-adding the fallback.

Do not touch `apm-core/src/init.rs` lines 156–169 (the `apm init --migrate` path that moves `apm.toml` to `.apm/config.toml` for real users). That code intentionally reads `apm.toml` and must remain.

### Open questions

**Q:** **`apm init --migrate` confirmed** (2026-05-02): Flag verified via `apm init --help`:

**Q:** --migrate
**Q:** Migrate root-level apm.toml -> .apm/config.toml and apm.agents.md -> .apm/agents.md

**Q:** AC item 10 is valid. The flag exists today; no sibling ticket needed.

### Amendment requests

- [x] AC body got rendered as a single line with literal `\n` separators (a TOML-string-vs-newline bug from how the text was submitted). Re-set the AC section so each checkbox is its own line and individually toggleable.

- [x] One AC item references `apm init --migrate` — that flag does not appear to be designed elsewhere in this epic. Either confirm it exists today, file a sibling ticket to add it, or remove the AC item.
- [x] **Round 1 amendments were marked done but not actually addressed.** The previous worker checked the boxes without doing the work. Specific gaps below — verify by `apm show 40fdde3b` after completing each.

- [x] Rendering bug persists outside the AC. The Problem, Out of scope, and Approach sections still contain literal `\n` strings instead of newlines (e.g. `"Both fallbacks were introduced...\n\nAfter this ticket..."`). The previous round only fixed the AC. Re-set Problem, Out of scope, and Approach with proper newlines so each paragraph/bullet is on its own line. **Verification: after the amendment round, `apm show 40fdde3b | grep -c '\\n'` returns 0.**

- [x] AC item 10 still asserts `"apm init --migrate still works"` and Out of scope still mentions the flag. The flag does not appear to exist (`apm init --help` does not list `--migrate`). Resolve one of three ways: (a) confirm the flag exists today and paste the matching `apm init --help` line into the ticket history before marking implemented; (b) file a sibling ticket to add `apm init --migrate` and remove this AC item, leaving a pointer to the new ticket; (c) remove the AC item entirely as out of scope. **Verification: after the amendment round, the substring `apm init --migrate` either does not appear in the spec, OR appears alongside a confirmed sibling-ticket id, OR is justified by a pasted `apm init --help` line.**

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:38Z | groomed | in_design | philippepascal |
| 2026-05-02T04:45Z | in_design | specd | claude-0502-0438-0028 |
| 2026-05-02T07:20Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T07:21Z | ammend | in_design | philippepascal |
| 2026-05-02T07:22Z | in_design | specd | claude-0502-0721-8f58 |
| 2026-05-02T16:55Z | specd | ammend | claude-0502-1300-rev1 |
| 2026-05-02T16:56Z | ammend | in_design | philippepascal |
| 2026-05-02T17:01Z | in_design | specd | claude-0502-1656-af08 |
| 2026-05-03T20:17Z | specd | ready | philippepascal |
| 2026-05-03T23:09Z | ready | in_progress | philippepascal |
