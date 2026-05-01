+++
id = "71d80e40"
title = "apm agents subcommand: new, list, test, eject"
state = "in_progress"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/71d80e40-apm-agents-subcommand-new-list-test-ejec"
created_at = "2026-04-30T20:04:57.796154Z"
updated_at = "2026-05-01T19:27:25.670266Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

Add the `apm agents` subcommand family for discovering, scaffolding, smoke-testing, and ejecting wrappers. Discoverability and authoring are the load-bearing UX for the wrapper feature.

**Reference spec:** `docs/agent-wrappers.md` — sections 'Skeleton command', 'Other wrapper-related commands'.

**Scope:** four subcommands under `apm agents`.

**`apm agents list`** — discover available wrappers.
- Lists built-in wrappers (from the registry in d3b93b95 + mocks from 25c92daa) and project-defined wrappers (from `.apm/agents/<name>/wrapper.*` per 2c32a282).
- For each: name, kind (built-in or project), and current configured-as marker (which profile/global uses it).
- One column for parser strategy if the wrapper declares one in manifest.toml.

**`apm agents new <name>`** — scaffold a custom wrapper.
- Creates `.apm/agents/<name>/` if it doesn't exist; refuses if it does (suggest `--force` for overwrite).
- Writes:
  - `wrapper.sh` — runnable template that prints all `APM_*` env vars to stderr, emits a minimal valid JSONL event on stdout, exits 0. Documents the contract inline as comments. Sets the execute bit (`chmod +x`).
  - `apm.worker.md` — copy of the project's current `.apm/apm.worker.md` (or the claude built-in's default if no project file).
  - `apm.spec-writer.md` — same.
  - `manifest.toml` — defaults written explicitly: `contract_version = 1`, `parser = "canonical"`.
- Prints next-step guidance: edit `wrapper.sh`, run `apm agents test <name>` to validate.

**`apm agents test <name>`** — smoke-test a wrapper.
- Spawns the wrapper against a synthetic ticket in a temp worktree (no real ticket touched).
- Captures the wrapper's output and exit code.
- Reports: exit code, count of canonical JSONL events, any non-canonical lines on stdout, count of stderr lines, wall time.
- Pass criteria: exit 0, at least one canonical JSONL event, no parse errors.
- Useful before assigning a new wrapper to a real worker queue.

**`apm agents eject <name>`** — extract a built-in to a script.
- Writes the built-in wrapper's source equivalent to `.apm/agents/<name>/wrapper.sh` (a bash script that reproduces the built-in's behaviour). The Rust built-in stays registered; the project script shadows it per the resolution rules in 2c32a282.
- Useful when a user wants to customize a built-in (e.g. add custom env vars, change the model invocation).
- Refuses if `.apm/agents/<name>/` already exists.

**Out of scope:**
- Wrapper-contract version checking inside `apm agents test` — defer to the versioning ticket.
- Distributing wrappers across projects (`apm agents install`) — out of scope.
- An `apm agents remove` command — users can `rm -r` the directory.

**Tests:**
- `list`: built-ins appear; a fixture project script appears with kind=project.
- `new`: directory and files created; `wrapper.sh` is executable; second invocation refuses.
- `test`: passing wrapper reports success; failing wrapper (non-zero exit) reports the failure with the captured stderr.
- `eject`: claude eject writes a script that, when run as the configured agent, produces the same canonical events as the built-in.

### Acceptance criteria

- [x] `apm agents list` prints a row for the `claude` built-in with kind `built-in`
- [ ] `apm agents list` prints a row for each executable `wrapper.*` found under `.apm/agents/<name>/` with kind `project`
- [ ] `apm agents list` marks the agent matching the configured `workers.command` (legacy field) with a `(configured)` indicator
- [ ] `apm agents list` shows a `parser` column value read from `manifest.toml`; defaults to `canonical` when the manifest is absent or the field is unset
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/wrapper.sh` with the execute bit set (mode `0o755` on Unix)
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/apm.worker.md` with content copied from `.apm/apm.worker.md` or the built-in default when the project file is absent
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/apm.spec-writer.md` with content copied from `.apm/apm.spec-writer.md` or the built-in default when the project file is absent
- [ ] `apm agents new <name>` creates `.apm/agents/<name>/manifest.toml` containing `contract_version = 1` and `parser = "canonical"`
- [ ] `apm agents new <name>` exits non-zero with a message that mentions `--force` when `.apm/agents/<name>/` already exists
- [ ] `apm agents new <name> --force` succeeds when the directory already exists and overwrites the scaffolded files
- [ ] `apm agents new <name>` prints next-step guidance directing the user to edit `wrapper.sh` and run `apm agents test <name>`
- [ ] `apm agents test <name>` exits 0 and prints a pass summary when the wrapper exits 0 and emits at least one canonical JSONL line (a JSON object containing a `"type"` key)
- [ ] `apm agents test <name>` exits non-zero and prints a fail summary when the wrapper exits non-zero
- [ ] `apm agents test <name>` reports exit code, canonical JSONL event count, non-canonical log line count, stderr line count, and wall-clock milliseconds in its output
- [ ] `apm agents test <name>` exits non-zero with a clear error message when `<name>` is not a known wrapper (built-in or project)
- [ ] `apm agents eject claude` creates `.apm/agents/claude/wrapper.sh` containing a bash script that invokes the `claude` CLI with `--print --output-format stream-json --verbose`
- [ ] `apm agents eject <name>` creates `.apm/agents/<name>/manifest.toml` with `contract_version = 1` and `parser = "canonical"`
- [ ] `apm agents eject <name>` sets the execute bit on the ejected `wrapper.sh`
- [ ] `apm agents eject <name>` exits non-zero when `.apm/agents/<name>/` already exists
- [ ] `apm agents eject <name>` exits non-zero with a message when `<name>` is not a known built-in

### Out of scope

- Per-agent instruction file resolution (`.apm/agents/<name>/apm.worker.md` etc.) — ticket 7f5f73d5; `apm agents new` writes these files as scaffold aids but their resolution order is not wired here
- `apm agents install` and `apm agents remove` subcommands
- Wrapper-contract version checking inside `apm agents test` — ticket 2e772eab
- Mock built-in wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) appearing in `apm agents list` — ticket 25c92daa must land first; once it does they appear automatically via the built-in registry without changes here
- The previous `apm agents` (no subcommand) behaviour of printing the instructions file — this ticket replaces it entirely
- Running `apm agents test` against the real `claude` CLI in automated tests — fixtures use small shell scripts
- Config-driven active-profile column once ticket 6cac8518 lands — pre-6cac8518 the marker uses `workers.command`; `list_wrappers` includes a TODO comment for the post-6cac8518 switch to `workers.agent` and per-profile iteration
- Windows execute-bit semantics (same limitation as ticket 2c32a282; any `wrapper.*` file is treated as executable on non-Unix platforms)

### Approach

**Prereqs:** d3b93b95 (`Wrapper` trait, `WrapperContext`, `resolve_builtin`, `ClaudeWrapper`) and 2c32a282 (`resolve_wrapper`, `WrapperKind`, `find_script`, `parse_manifest`, `CustomWrapper`) must be merged into the epic branch before this ticket is implemented. All API references below assume those tickets' final shapes.

---

**`apm-core/src/wrapper/mod.rs` — add `list_builtin_names`**

Add one new public function returning the static list of registered built-in names:

```rust
pub fn list_builtin_names() -> &'static [&'static str] {
    &["claude"]
}
```

When ticket 25c92daa lands it expands the list; no other change needed here.

---

**`apm-core/src/agents.rs` — new module**

Register with `pub mod agents;` in `apm-core/src/lib.rs`.

Public types:

```rust
pub struct WrapperEntry {
    pub name: String,
    pub kind: WrapperKind,          // re-exported from wrapper::WrapperKind (2c32a282)
    pub parser: String,             // "canonical" or value from manifest.toml
    pub configured_as: Vec<String>, // e.g. ["(default)"] or ["spec_agent"]
}

pub struct TestReport {
    pub exit_code: i32,
    pub canonical_events: usize,
    pub non_canonical_lines: usize,
    pub stderr_lines: usize,
    pub wall_millis: u64,
    pub passed: bool,               // exit 0 && canonical_events >= 1
}
```

**`list_wrappers(root: &Path, config: &Config) -> anyhow::Result<Vec<WrapperEntry>>`**

1. Built-in entries: for each name in `wrapper::list_builtin_names()`, create a `WrapperEntry` with `kind: WrapperKind::Builtin(name.to_owned())`, `parser: "canonical"`, `configured_as: vec![]`.
2. Project entries: read `root/.apm/agents/` (skip if absent or unreadable). For each subdirectory `entry_name`, call `wrapper::resolve_wrapper(root, entry_name)?`. If `Ok(Some(WrapperKind::Custom ..))`, add a project entry. Set `parser` from `wrapper::parse_manifest(root, entry_name)` — use manifest `parser` field, default to `"canonical"`.
3. Configured marker: read `config.workers.command` (legacy, defaults to `"claude"`). For the entry whose `name == config.workers.command`, push `"(default)"` to `configured_as`. (TODO post-6cac8518: switch to `config.workers.agent` and iterate `config.worker_profiles` for per-profile markers.)
4. Sort: built-ins first (in `list_builtin_names` order), then project wrappers alphabetically.

**`scaffold_wrapper(root: &Path, name: &str, force: bool) -> anyhow::Result<()>`**

1. `let dir = root.join(".apm/agents").join(name);`
2. If `dir.exists() && !force`: bail with `.apm/agents/{name}/ already exists; use --force to overwrite`
3. `fs::create_dir_all(&dir)?`
4. Write `dir/wrapper.sh` using the `WRAPPER_TEMPLATE` constant (see below). On Unix, set permissions mode `0o755` via `std::os::unix::fs::PermissionsExt::from_mode`.
5. Write `dir/manifest.toml`: `[wrapper]\ncontract_version = 1\nparser = "canonical"\n`
6. For `apm.worker.md`: try `fs::read_to_string(root.join(".apm/apm.worker.md"))`; if absent, use the same default string that `apm init` writes (locate the constant in `apm_core::init`). Write to `dir/apm.worker.md`.
7. Same for `apm.spec-writer.md`.
8. `Ok(())`

**`WRAPPER_TEMPLATE` constant** (define as `const WRAPPER_TEMPLATE: &str` in `agents.rs`):

A bash script with:
- `#!/usr/bin/env bash` shebang
- Inline comments documenting each `APM_*` env var, the stdout/stderr/exit-code contract
- `set -euo pipefail`
- `env | grep '^APM_' >&2 || true` to dump APM vars to stderr on startup
- `SYSTEM_PROMPT="$(cat "$APM_SYSTEM_PROMPT_FILE")"` and `USER_MESSAGE="$(cat "$APM_USER_MESSAGE_FILE")"`
- A `printf` emitting one minimal valid JSONL line (`{"type":"text","text":"wrapper skeleton — replace with real invocation"}`) so `apm agents test` counts at least one canonical event
- TODO comment to replace the printf with a real agent invocation
- TODO comment to call `apm state "$APM_TICKET_ID" <target-state>`
- `exit 0`

**`test_wrapper(root: &Path, name: &str) -> anyhow::Result<TestReport>`**

1. Call `wrapper::resolve_wrapper(root, name)?`; if `Ok(None)`: bail with agent-not-found message.
2. Create a temp dir (use `std::env::temp_dir()` joined with a `rand_u16()` suffix, the same helper used in `start.rs`). Write `system.txt` → `"You are a test agent."`, `message.txt` → `"Test run — apm agents test."`. Set `log_path = tmpdir/wrapper.log`.
3. Build `WrapperContext` (from d3b93b95): `worker_name = "agents-test"`, `ticket_id = "00000000"`, `ticket_branch = "test/agents-test"`, `worktree_path = tmpdir`, file paths as above, `skip_permissions = false`, `profile = "test"`, rest at defaults/empty.
4. Spawn via the resolved `WrapperKind`: `Custom { script_path, manifest }` → `CustomWrapper::spawn(&ctx)?`; `Builtin(n)` → `resolve_builtin(&n).expect("registered").spawn(&ctx)?`.
5. Record start instant, call `child.wait()`, compute `wall_millis`.
6. Read log file (stdout+stderr interleaved per Wrapper contract; treat missing file as empty). Classify each non-empty line: valid JSON object containing a `"type"` key → `canonical_events += 1`; line starting with `APM_` (env-dump from skeleton) → `stderr_lines += 1`; everything else → `non_canonical_lines += 1`.
7. `passed = status.success() && canonical_events >= 1`. Return `TestReport`.
8. Remove tmpdir on return (best-effort, ignore errors).

Line classification is heuristic because stdout and stderr share the log file. This is acceptable for a smoke test.

**`eject_wrapper(root: &Path, name: &str) -> anyhow::Result<()>`**

1. If `wrapper::resolve_builtin(name).is_none()`: bail with `'<name>' is not a known built-in; run apm agents list to see available wrappers`.
2. `let dir = root.join(".apm/agents").join(name);`
3. If `dir.exists()`: bail with `.apm/agents/{name}/ already exists; delete it first to eject again`.
4. `fs::create_dir_all(&dir)?`
5. Match `name`: `"claude"` → write `CLAUDE_EJECT_SCRIPT` constant; other built-in names from 25c92daa → add cases as those land; default arm bails with "eject not yet implemented for built-in NAME".
6. Set mode `0o755` on the script file.
7. Write `dir/manifest.toml` with the identical template used by `scaffold_wrapper` — `[wrapper]\ncontract_version = 1\nparser = "canonical"\n`. This is intentional: the ejected manifest is the same v1-canonical template, so it is recognised without extra setup by both the manifest resolution path in `2c32a282` and the version check in `2e772eab`.
8. `Ok(())`

**`CLAUDE_EJECT_SCRIPT` constant** (define as `const CLAUDE_EJECT_SCRIPT: &str` in `agents.rs`):

A bash script with:
- `#!/usr/bin/env bash` shebang, header comment `Ejected from APM built-in: claude`
- `set -euo pipefail`
- Builds `ARGS=(--print --output-format stream-json --verbose)`
- Appends `--system-prompt "$(cat "$APM_SYSTEM_PROMPT_FILE")"` to ARGS
- Conditionally appends `--model "$APM_OPT_MODEL"` when `APM_OPT_MODEL` is non-empty
- Conditionally appends `--dangerously-skip-permissions` when `APM_SKIP_PERMISSIONS = "1"`
- Ends with `exec claude "${ARGS[@]}" "$(cat "$APM_USER_MESSAGE_FILE")"`

---

**`apm/src/cmd/agents.rs` — replace with subcommand handlers**

Delete existing `run(root)`. Add four functions:

`run_list(root: &Path) -> Result<()>`: load config, call `apm_core::agents::list_wrappers(root, &config)?`, print a column-aligned table using `println!` (no external crate):
```
NAME             KIND       PARSER     STATUS
claude           built-in   canonical  (configured)
my-wrapper       project    canonical
```

`run_new(root: &Path, name: &str, force: bool) -> Result<()>`: call `apm_core::agents::scaffold_wrapper(root, name, force)?`, then print the list of created files and next-step guidance directing the user to edit `wrapper.sh` and run `apm agents test <name>`.

`run_test(root: &Path, name: &str) -> Result<()>`: call `apm_core::agents::test_wrapper(root, name)?`. Print one-line summary: `PASS  exit=0  events=N  non-canonical=0  stderr=N  wall=Nms`. On fail, print `FAIL  ...` and call `anyhow::bail!` to produce non-zero exit.

`run_eject(root: &Path, name: &str) -> Result<()>`: call `apm_core::agents::eject_wrapper(root, name)?`, then print path of ejected script and guidance to run `apm agents test <name>`.

---

**`apm/src/main.rs` — wire subcommands**

Add `AgentsCommand` enum (four variants: `List`, `New { name: String, #[arg(long)] force: bool }`, `Test { name: String }`, `Eject { name: String }`).

Change `Command::Agents` from unit variant to `Agents { #[command(subcommand)] command: AgentsCommand }`.

Update the match arm to dispatch to the four `cmd::agents::run_*` functions.

Update help text from `"agents         Print agent instructions"` to `"agents         Manage agent wrappers (list, new, test, eject)"`.

Update `hash_trip::is_read_only_command` if `Agents` is listed: `List` and `Test` are read-only; `New` and `Eject` are mutating.

---

**Tests — `apm-core/tests/agents_integration.rs`**

- `list_shows_builtin_claude` — no `.apm/agents/` dir; assert `list_wrappers` returns entry with `name == "claude"` and `WrapperKind::Builtin(_)`
- `list_shows_project_wrapper` — create `root/.apm/agents/my-wrapper/wrapper.sh` mode `0o755`; assert `list_wrappers` returns a `WrapperKind::Custom` entry for `"my-wrapper"`
- `scaffold_creates_all_files` — call `scaffold_wrapper(root, "test-wrap", false)`; assert all four files exist; assert `wrapper.sh` permissions `& 0o111 != 0`
- `scaffold_refuses_existing_dir` — call twice without force; assert second call is `Err` with message containing `"--force"`
- `scaffold_force_overwrites` — call twice with `force = false` then `force = true`; assert second call is `Ok`
- `test_passes_for_good_script` — write `.apm/agents/test-ok/wrapper.sh` (mode `0o755`) emitting one valid JSONL line and exiting 0; assert `report.passed == true` and `report.canonical_events >= 1`
- `test_fails_for_nonzero_exit` — write `.apm/agents/test-fail/wrapper.sh` (mode `0o755`) exiting 1; assert `report.passed == false` and `report.exit_code == 1`
- `eject_claude_creates_script` — call `eject_wrapper(root, "claude")`; assert `.apm/agents/claude/wrapper.sh` exists and contains both `"claude"` and `"output-format"`
- `eject_refuses_existing_dir` — pre-create `.apm/agents/claude/`; assert `eject_wrapper` returns `Err`
- `eject_unknown_builtin_returns_error` — call `eject_wrapper(root, "not-a-builtin")`; assert `Err` message contains `"not a known built-in"`

---

**File change summary**

| File | Change |
|---|---|
| `apm-core/src/lib.rs` | Add `pub mod agents;` |
| `apm-core/src/wrapper/mod.rs` | Add `pub fn list_builtin_names()` |
| `apm-core/src/agents.rs` | New: `WrapperEntry`, `TestReport`, all four functions, template constants |
| `apm/src/cmd/agents.rs` | Replace with `run_list`, `run_new`, `run_test`, `run_eject` |
| `apm/src/main.rs` | Add `AgentsCommand`; change `Command::Agents` to subcommand; wire dispatch; update help text; update read-only list |
| `apm-core/tests/agents_integration.rs` | New: 10 integration tests |

### Open questions


### Amendment requests

- [x] Make explicit in the Approach that the manifest emitted by `eject` (`contract_version = 1`, `parser = "canonical"`) is the same template as `new` and is therefore recognised as v1-canonical by the resolution paths in `2c32a282` (manifest parser) and `2e772eab` (version check). State this so users don't wonder whether ejected scripts need extra setup, and so a future implementer doesn't accidentally invent a different template for ejected wrappers. One sentence in the eject section is enough.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:04Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T22:02Z | groomed | in_design | philippepascal |
| 2026-05-01T00:08Z | in_design | ammend | philippepascal |
| 2026-05-01T00:21Z | ammend | in_design | philippepascal |
| 2026-05-01T00:30Z | in_design | specd | claude-0501-0021-fd28 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:32Z | ammend | in_design | philippepascal |
| 2026-05-01T01:36Z | in_design | specd | claude-0501-0132-6a28 |
| 2026-05-01T17:38Z | specd | ready | philippepascal |
| 2026-05-01T19:27Z | ready | in_progress | philippepascal |