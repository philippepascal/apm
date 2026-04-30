+++
id = "2c32a282"
title = "Custom wrapper resolution from .apm/agents/<name>/"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2c32a282-custom-wrapper-resolution-from-apm-agent"
created_at = "2026-04-30T20:02:50.794362Z"
updated_at = "2026-04-30T21:02:24.185414Z"
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

Checkboxes; each one independently testable.

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
