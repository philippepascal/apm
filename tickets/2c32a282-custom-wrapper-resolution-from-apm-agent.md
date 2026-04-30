+++
id = "2c32a282"
title = "Custom wrapper resolution from .apm/agents/<name>/"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2c32a282-custom-wrapper-resolution-from-apm-agent"
created_at = "2026-04-30T20:02:50.794362Z"
updated_at = "2026-04-30T21:24:01.213094Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95"]
+++

## Spec

### Problem

Implement custom-wrapper resolution from `.apm/agents/<name>/` so projects can ship their own agent integrations alongside built-ins. A project script at `.apm/agents/<name>/wrapper.<ext>` shadows any built-in of the same name.

**Reference spec:** `docs/agent-wrappers.md` — sections 'Custom wrappers', 'manifest.toml (optional)'.

**Scope:**
- New module `apm-core/src/wrapper/custom.rs` (or similar). Public API: `pub fn resolve_wrapper(root: &Path, name: &str) -> Option<WrapperKind>` returning either a `Custom { script_path: PathBuf, manifest: Option<Manifest> }` or `Builtin(name)`.
- Resolution order: project script first (any executable file matching `wrapper.*` in `.apm/agents/<name>/`), then built-in.
- Custom wrappers are exec'd directly (not via shell). The wrapper script must have its shebang and execute bit set; APM does not interpret extensions or pick interpreters.
- Parse optional `manifest.toml` in the wrapper directory: `[wrapper] name`, `contract_version` (default 1), `parser` (default "canonical"), `parser_command` (only when parser = "external"). Strict parsing; unknown keys are warnings.
- Wire the dispatcher (from d3b93b95) to call into custom-wrapper exec when the resolved kind is `Custom`.
- Extend `apm validate` to:
  - Confirm the configured agent (global, per-profile) resolves either to a built-in or a project script.
  - Validate `manifest.toml` if present (parses, declared `contract_version` is supported by this APM build).
  - Error message format: "agent 'foo' not found: checked built-ins {claude, ...} and `.apm/agents/foo/`".

**Out of scope:**
- Per-agent instructions (`apm.worker.md` etc. per agent dir) — separate ticket.
- The `apm agents new/list/test/eject` subcommand — separate ticket.
- Wrapper-contract version checking at spawn time — separate ticket; this ticket only parses the field.
- External parser invocation — separate ticket; this ticket only stores the manifest fields.

**Tests:**
- Resolution test: project script shadows built-in.
- Resolution test: missing wrapper returns None; validate fails with the expected error.
- Manifest parsing tests (valid, invalid, missing).
- Integration test: a fixture project with a `.apm/agents/echo-test/wrapper.sh` that just echoes a JSONL event and exits 0; dispatcher runs it, output captured to log.

**Wrapper-contract version 1** is the only one this ticket supports; manifest.toml declaring contract_version > 1 should be rejected with a clear upgrade-APM message.

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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:02Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:24Z | groomed | in_design | philippepascal |