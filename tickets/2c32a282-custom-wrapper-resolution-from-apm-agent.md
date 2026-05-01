+++
id = "2c32a282"
title = "Custom wrapper resolution from .apm/agents/<name>/"
state = "ammend"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2c32a282-custom-wrapper-resolution-from-apm-agent"
created_at = "2026-04-30T20:02:50.794362Z"
updated_at = "2026-05-01T01:10:25.526530Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

APM can currently invoke only the built-in Claude wrapper. There is no way for a project team to integrate a different AI CLI (Aider, Codex, a company-internal tool) without modifying APM's Rust source and shipping a new binary. The `d3b93b95` wrapper contract established the `Wrapper` trait, `WrapperContext`, and `ClaudeWrapper`, but the dispatcher still always resolves to Claude — it has no mechanism to look for, load, or exec a project-defined script.

This ticket wires custom-wrapper resolution into the dispatcher. When a project places an executable script at `.apm/agents/<name>/wrapper.<ext>`, APM picks it up and runs it in place of any built-in with the same name. An optional `manifest.toml` alongside the script carries metadata: which contract version the wrapper targets and which output-parser strategy it uses. `apm validate` gains agent-resolution checks so broken or missing wrappers are caught before any real worker is spawned.

The result is a genuinely multi-agent APM: any tool that can read the APM env vars and emit JSONL on stdout can be wired in as a first-class wrapper, without touching APM's binary.

### Acceptance criteria

- [ ] `resolve_wrapper(root, "claude")` returns `Some(WrapperKind::Custom { script_path, .. })` when `.apm/agents/claude/wrapper.sh` exists and is executable, shadowing the built-in
- [ ] `resolve_wrapper(root, "claude")` returns `Some(WrapperKind::Builtin("claude"))` when no project script exists for `"claude"`
- [ ] `resolve_wrapper(root, "bogus")` returns `Ok(None)` when neither a project script nor a built-in with that name exists
- [ ] A `wrapper.*` file that exists but is not executable (Unix: mode `& 0o111 == 0`) is invisible to `resolve_wrapper`; the function falls through to the built-in or returns `None`
- [ ] `apm validate` emits an error of the form `"agent 'foo' not found: checked built-ins {claude} and '.apm/agents/foo/'"` when the configured agent cannot be resolved
- [ ] `apm validate` emits a warning (not an error) when a `.apm/agents/<name>/wrapper.*` file exists but lacks the executable bit
- [ ] A valid `manifest.toml` parses without error; `contract_version = 1` and `parser = "canonical"` are stored on the `Manifest` struct
- [ ] A `manifest.toml` with only `[wrapper]` and no explicit fields parses to defaults: `contract_version = 1`, `parser = "canonical"`, `parser_command = None`
- [ ] A `manifest.toml` with an unknown key causes `apm validate` to emit a warning (not an error); the manifest still parses and the wrapper is usable
- [ ] A syntactically invalid `manifest.toml` causes `apm validate` to emit an error; `resolve_wrapper` also returns an error
- [ ] A `manifest.toml` with `contract_version = 2` causes `apm validate` to emit an error directing the user to upgrade APM
- [ ] `CustomWrapper::spawn()` returns an error (does not spawn the process) when `manifest.contract_version > 1`
- [ ] The dispatcher in `start.rs` exec'''s a custom wrapper script directly via `Command::new(&script_path)` with no shell interpreter interposed, and all APM contract env vars are present in the child environment
- [ ] Integration: a fixture `.apm/agents/echo-test/wrapper.sh` (executable, emits one valid JSONL line, exits 0) is spawned by the dispatcher; its output is captured to the log file and the child exits 0
- [ ] Unit tests `resolve_wrapper_nonexecutable_invisible`, `resolve_wrapper_fallback_to_builtin`, `resolve_wrapper_missing_returns_none`, `manifest_parse_valid`, `manifest_parse_defaults`, `manifest_parse_invalid_toml`, and `manifest_missing` all pass

### Out of scope

- Per-agent instruction file resolution (`apm.worker.md` etc. under `.apm/agents/<name>/`) — ticket 7f5f73d5
- The `apm agents new/list/test/eject` subcommand family — ticket 71d80e40
- Wrapper-contract version compatibility checks beyond rejecting `contract_version > 1` at parse/validate time — ticket 2e772eab; this ticket stores the field but only enforces the v1 ceiling
- External parser invocation (`parser = "external"` piping stdout through `parser_command`) — ticket 2803bf07; this ticket parses and stores `parser` and `parser_command` but does not act on them at spawn time
- Reading the `agent` config field introduced by ticket 6cac8518; `start.rs` after this ticket still passes the hardcoded string `"claude"` to `resolve_wrapper`
- Frontmatter `agent` / `agent_overrides` override — ticket 0ca3e019
- Mock built-in wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — ticket 25c92daa
- `apm migrate --fix` automated config rewrite — ticket 3048d7e9
- Windows execute-bit semantics (on non-Unix platforms, `find_script` treats any `wrapper.*` file as executable)

### Approach

**New types -- apm-core/src/wrapper/custom.rs (new file)**

Manifest struct (deserialised from [wrapper] in manifest.toml):
- name: Option<String>
- contract_version: u32 with serde default 1
- parser: String with serde default canonical
- parser_command: Option<String> (only meaningful when parser = external)

WrapperKind enum (defined in wrapper/custom.rs, re-exported from wrapper/mod.rs):
- Custom variant: script_path: PathBuf, manifest: Option<Manifest>
- Builtin variant: String (the name)

CustomWrapper struct holds script_path and manifest and implements the Wrapper trait from d3b93b95.

---

**wrapper/custom.rs -- private helpers**

find_script(root: &Path, name: &str) -> Option<PathBuf>
- Read entries under root/.apm/agents/<name>/; return None if the directory is absent or unreadable
- Keep entries whose file name starts with wrapper. (any extension after the dot)
- Unix: keep only entries where metadata().permissions().mode() & 0o111 != 0
- Non-Unix: treat any matching file as executable
- Return the first match in alphabetical order (deterministic when multiple wrapper.* files coexist)

parse_manifest(root: &Path, name: &str) -> anyhow::Result<Option<Manifest>>
- Path is root/.apm/agents/<name>/manifest.toml; return Ok(None) if absent
- Read and parse as TOML; deserialise the [wrapper] table into Manifest via serde
- Propagate IO or TOML parse errors via anyhow::Context

manifest_unknown_keys(root: &Path, name: &str) -> anyhow::Result<Vec<String>>
- Parse manifest as toml::Value, navigate to the [wrapper] table, collect key names
- Return any key not in the known set: name, contract_version, parser, parser_command
- Called by apm validate to emit warnings without failing the parse

---

**wrapper/mod.rs -- resolve_wrapper**

Add pub mod custom; and re-export WrapperKind and Manifest.

Signature: pub fn resolve_wrapper(root: &Path, name: &str) -> anyhow::Result<Option<WrapperKind>>

Algorithm:
1. Call find_script(root, name); if a script is found, call parse_manifest(root, name)? and
   return Ok(Some(WrapperKind::Custom containing script_path and manifest))
2. Else if resolve_builtin(name).is_some(), return Ok(Some(WrapperKind::Builtin(name.to_owned())))
3. Else return Ok(None)

---

**CustomWrapper::spawn (implements Wrapper trait)**

1. Check self.manifest contract_version (defaulting to 1 when manifest is None); if > 1, bail
   with a message stating the declared version is unsupported and directing the user to upgrade APM
2. Build Command::new(&self.script_path) -- no shell interpreter, the script is exec-d directly
3. Set all APM contract env vars (identical set to ClaudeWrapper; see d3b93b95 approach table)
4. Forward ctx.extra_env entries (user-configured env from [workers] env)
5. .current_dir(&ctx.worktree_path)
6. Redirect stdout + stderr to File::create(&ctx.log_path)?; try_clone() for stderr fd
7. .process_group(0) then .spawn()

---

**start.rs -- dispatcher wiring**

In spawn_worker (introduced by d3b93b95), add project_root: &Path as a second parameter.
Update the three call sites (run, run_next, spawn_next_worker) to pass root, which is already
in scope at each site.

Replace the hardcoded resolve_builtin(claude)...spawn(&ctx) call with a match on
resolve_wrapper(project_root, claude)?:

- Custom variant -> construct CustomWrapper from script_path and manifest, call .spawn(&ctx)?
- Builtin(name) variant -> resolve_builtin(&name).expect(known built-in).spawn(&ctx)?
- None -> anyhow::bail with message: agent not found, checked built-ins and .apm/agents/claude/

The hardcoded claude string is replaced by config.workers.agent when ticket 6cac8518 lands;
the shape of this call does not change at that point.

---

**validate.rs -- validate_agents helper**

Add fn validate_agents(config: &Config, root: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>)
and call it from validate_config.

Steps:

1. Collect agent names to check.
   Pre-6cac8518: use config.workers.command (defaults to claude) as the single name.
   When 6cac8518 lands: switch to config.workers.agent and add per-profile agent names.
   De-duplicate.

2. For each name call resolve_wrapper(root, name):
   - Ok(None) -> push error: agent NAME not found: checked built-ins (claude) and .apm/agents/NAME/
   - Err(e) -> push error: agent NAME: {e}
   - Ok(Some(_)) -> ok

3. Scan .apm/agents/ (skip if absent); for each subdirectory NAME:
   - If any wrapper.* file exists but none is executable (Unix only) ->
     push warning: agent NAME: .apm/agents/NAME/wrapper.* exists but is not executable; run chmod +x
   - If manifest.toml exists:
     - TOML parse error -> push error: agent NAME: manifest.toml is not valid TOML: {e}
     - contract_version > 1 -> push error: agent NAME: manifest.toml declares contract_version V;
       this APM build supports version 1 only -- upgrade APM
     - Unknown keys via manifest_unknown_keys -> one warning per key:
       agent NAME: manifest.toml: unknown key K

---

**Tests**

Unit tests in wrapper/custom.rs under #[cfg(test)]:

- resolve_wrapper_custom_shadows_builtin: temp dir with executable .apm/agents/claude/wrapper.sh;
  assert resolve_wrapper(root, claude) returns the Custom variant
- resolve_wrapper_fallback_to_builtin: no .apm/agents/claude/ dir;
  assert result is Builtin(claude)
- resolve_wrapper_missing_returns_none: no script, not a built-in name;
  assert Ok(None)
- resolve_wrapper_nonexecutable_invisible: wrapper.sh present with mode 0o644;
  assert result is Builtin(claude) (non-executable script is invisible, falls through)
- manifest_parse_valid: write a complete valid manifest.toml; assert struct fields match declared values
- manifest_parse_defaults: write [wrapper] with no keys;
  assert contract_version == 1, parser == canonical, parser_command == None
- manifest_parse_invalid_toml: write syntactically broken TOML;
  assert parse_manifest returns Err
- manifest_missing: no manifest.toml present;
  assert parse_manifest returns Ok(None)
- manifest_unknown_keys_detected: write [wrapper] with an extra unknown_key = foo;
  assert manifest_unknown_keys returns a vec containing unknown_key
- spawn_rejects_contract_version_gt_1: CustomWrapper with manifest.contract_version = 2;
  assert spawn() returns Err containing the string upgrade APM

Integration test in apm-core/tests/custom_wrapper_integration.rs:

- integration_echo_test_wrapper: fixture project contains .apm/agents/echo-test/wrapper.sh
  with a shebang line and a single printf emitting one valid JSONL line to stdout; mode 0o755.
  Build a minimal WrapperContext pointing at a temp worktree and log file.
  Call resolve_wrapper(root, echo-test), assert it returns the Custom variant.
  Call CustomWrapper::spawn, wait for child exit 0.
  Read log file and assert it contains the emitted JSONL line.

---

**File change summary**

| File | Change |
|---|---|
| apm-core/src/wrapper/mod.rs | Add pub mod custom; re-export WrapperKind; add resolve_wrapper() |
| apm-core/src/wrapper/custom.rs | New: Manifest, WrapperKind, CustomWrapper + Wrapper impl, helpers, unit tests |
| apm-core/src/start.rs | Add project_root param to spawn_worker; replace resolve_builtin call with resolve_wrapper dispatch |
| apm-core/src/validate.rs | Add validate_agents() helper; call from validate_config |
| apm-core/tests/custom_wrapper_integration.rs | New integration test with echo-test fixture |

### Open questions


### Amendment requests

- [ ] Make the two-layer manifest validation explicit in the Approach: (a) `apm validate` parses every project wrapper's `manifest.toml` and reports errors at validate time so users find problems early, before any worker is spawned; (b) `CustomWrapper::spawn` re-checks the manifest at spawn time as a safety net (the file could have been edited between validate and spawn). Both layers are necessary — early surfacing is the load-bearing UX, but spawn-time check prevents silent crashes when a user edits a manifest mid-session. The current spec implements both but doesn't say so explicitly; an implementer might be tempted to drop one.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:24Z | groomed | in_design | philippepascal |
| 2026-04-30T21:36Z | in_design | specd | claude-0430-2124-5738 |
| 2026-05-01T00:09Z | specd | ammend | philippepascal |
| 2026-05-01T00:42Z | ammend | in_design | philippepascal |
| 2026-05-01T00:45Z | in_design | specd | claude-0501-0042-ee50 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
