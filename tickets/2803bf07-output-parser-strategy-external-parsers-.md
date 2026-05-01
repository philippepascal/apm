+++
id = "2803bf07"
title = "Output parser strategy: external parsers via manifest.toml"
state = "in_design"
priority = 0
effort = 5
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2803bf07-output-parser-strategy-external-parsers-"
created_at = "2026-04-30T20:05:40.844536Z"
updated_at = "2026-05-01T00:41:41.266824Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["2c32a282", "2e772eab"]
+++

## Spec

### Problem

Support agents whose output is too far from APM's canonical JSONL stream-json to translate inline. The wrapper declares an external parser binary in `manifest.toml`; APM pipes the wrapper's stdout through it before capturing.

**Reference spec:** `docs/agent-wrappers.md` — section 'Output parser strategy', 'Custom wrappers / manifest.toml'.

**Scope:**
- `manifest.toml` already parses `parser` and `parser_command` (added in ticket 2c32a282). This ticket wires them into spawn.
- Three parser modes:
  - `parser = "canonical"` (default) — wrapper produces JSONL stream-json directly. No transformation. Today's behaviour.
  - `parser = "raw"` — wrapper output is captured as-is to log; no canonical-event parsing. Useful for agents whose output is unstructured or only meant for human reading. Worker-state events still drive off the wrapper's exit code and any `apm state` calls it makes.
  - `parser = "external"` — wrapper output is piped through the binary at `parser_command` (must be in PATH or absolute path); the parser's stdout becomes APM's captured stream. Parser must produce canonical JSONL.
- Spawn glue: when `parser = "external"`, spawn the wrapper and the parser as a pipe (wrapper.stdout → parser.stdin). Capture parser.stdout (canonical events) and parser.stderr (parser's diagnostics) to `.apm-worker.log`. The wrapper's stderr also goes to the log directly.
- Validate `parser_command` exists when `parser = "external"`. Fail at spawn with a clear error if not.
- Built-in wrappers always default to canonical (no manifest needed).

**Out of scope:**
- Shipping any external parser binaries (e.g. `apm-output-parser-aider`). Those are separate cargo crates.
- A formal canonical event vocabulary doc — already noted as an open question in the spec doc.
- Multiplexing two parsers on the same wrapper. Pick one strategy per wrapper.

**Tests:**
- `canonical` mode: existing wrapper tests unchanged.
- `raw` mode: a wrapper that emits "hello world" → log contains "hello world" verbatim, no JSONL parse warnings.
- `external` mode: a wrapper that emits non-JSONL text + a parser script that wraps each line in a JSONL event → log contains the parsed JSONL, original text only in the parser's stderr.
- Missing parser_command → spawn fails with a clear error citing the manifest path.

### Acceptance criteria

- [ ] When `parser` is absent from manifest.toml, or manifest.toml itself is absent, `CustomWrapper::spawn` captures the wrapper's stdout directly to the log file (equivalent to `parser = "canonical"`)
- [ ] When `parser = "canonical"`, `CustomWrapper::spawn` captures the wrapper's stdout directly to the log file with no transformation; the wrapper's stderr is also written to the same log file
- [ ] When `parser = "raw"`, `CustomWrapper::spawn` captures the wrapper's stdout verbatim to the log file; a wrapper emitting `"hello world\n"` produces a log whose content includes that text with no `[apm] warning:` lines injected by APM
- [ ] When `parser = "external"` and `parser_command` resolves to an executable binary, `CustomWrapper::spawn` creates an OS-level pipe: the wrapper's stdout is the parser's stdin; the parser's stdout is captured to the log file
- [ ] When `parser = "external"`, the wrapper's stderr is written to the log file independently (not through the parser pipe)
- [ ] When `parser = "external"`, the parser's stderr is also written to the log file
- [ ] When `parser = "external"` and `parser_command` is absent from manifest.toml, `CustomWrapper::spawn` returns `Err` before spawning any process; the error message names the manifest file path and states that `parser_command` is required
- [ ] When `parser = "external"` and `parser_command` names a binary not found in PATH and is not an absolute path to an existing file, `CustomWrapper::spawn` returns `Err` before spawning any process; the error message names the missing binary
- [ ] `apm validate` reports an error when a custom wrapper's manifest.toml declares `parser = "external"` but `parser_command` is absent
- [ ] Built-in wrappers (e.g. the `claude` built-in) always behave as `parser = "canonical"` regardless of any manifest file; no manifest is required or consulted for them
- [ ] `CustomWrapper::spawn` for external mode returns the parser's `Child` handle; the wrapper child is reaped in a background thread so it does not become a zombie

### Out of scope

- Shipping external parser binaries (e.g. `apm-output-parser-aider`, `apm-output-parser-codex`) — those are separate cargo crates distributed and installed independently
- Defining the canonical JSONL event vocabulary — already noted as an open question in `docs/agent-wrappers.md`; any parser binary this ticket tests against must emit valid JSONL but the schema is not defined here
- Multiplexing more than one parser per wrapper; manifest.toml accepts exactly one `parser` value and one optional `parser_command`
- In-wrapper translation (`parser = "canonical"` where the wrapper itself transforms output inline) — that is wrapper-author responsibility; APM does not assist with it
- The `apm agents test <name>` command for smoke-testing parser compliance — ticket 71d80e40
- Per-ticket parser override via frontmatter `agent_overrides` — ticket 0ca3e019
- Propagating the wrapper child's exit code to APM when both wrapper and parser run; the parser child's exit code is the effective worker exit code for this ticket
- Non-Unix platform differences in subprocess piping (Windows `Stdio::from(ChildStdout)` semantics) — out of scope for now; the implementation targets Unix
- Updating `docs/agent-wrappers.md` to document the `raw` parser mode — should be a follow-up to this ticket once the behaviour is validated

### Approach

### Files changed

| File | Change |
|---|---|
| `apm-core/src/wrapper/custom.rs` | Add `ParserStrategy` enum; refactor `CustomWrapper::spawn` to dispatch by strategy; implement OS-level pipe for `external` mode |
| `apm-core/src/validate.rs` | Extend `validate_agents` to push an error when `parser = "external"` and `parser_command` is absent |
| `apm-core/tests/custom_wrapper_integration.rs` | Add three integration tests covering canonical, raw, and external modes |

---

### `wrapper/custom.rs` — `ParserStrategy` enum

Add a private enum above `CustomWrapper`:

```rust
#[derive(Debug, Clone, PartialEq)]
enum ParserStrategy { Canonical, Raw, External }

impl ParserStrategy {
    fn from_manifest(m: Option<&Manifest>) -> Self {
        match m.and_then(|m| Some(m.parser.as_str())) {
            Some("external") => Self::External,
            Some("raw")      => Self::Raw,
            _                => Self::Canonical,   // absent, "canonical", or unknown value
        }
    }
}
```

---

### `wrapper/custom.rs` — refactor `CustomWrapper::spawn`

After the existing `check_contract_version(...)` call and the env-var block (both established by prior tickets), derive the strategy and branch:

```rust
let strategy = ParserStrategy::from_manifest(self.manifest.as_ref());
```

**Canonical / Raw (stdout -> log directly — identical spawn path):**

Keep the existing spawn sequence unchanged for these two modes:
- `File::create(&ctx.log_path)?` → `log_file`; `log_file.try_clone()?` → `log_clone`
- `Command::new(&self.script_path).envs(...).current_dir(...).stdout(log_file).stderr(log_clone).process_group(0).spawn()`

The distinction between `canonical` and `raw` is a concern for the orchestration layer (whether to JSONL-parse the log for event streaming); at spawn time the subprocess setup is identical. APM detects `raw` mode by reading the manifest's `parser` field after spawn.

**External — validate, then pipe:**

1. Derive the manifest path for error messages: `self.script_path.parent().unwrap().join("manifest.toml")`.

2. Require `parser_command`: call `.ok_or_else(|| anyhow!("...: parser = \"external\" but parser_command is not set"))` on `self.manifest.as_ref().and_then(|m| m.parser_command.as_deref())`. Return `Err` immediately if absent — no subprocess is started.

3. Validate the binary is findable before spawning the wrapper. Use `which::which(parser_cmd)` (see dependency note below). Return `Err` naming the missing binary if not found. Again, no subprocess is started yet.

4. Spawn the wrapper with `stdout(Stdio::piped())` and `stderr(log_clone)` (wrapper stderr goes directly to the log). Call `.process_group(0).spawn()?`. Take `wrapper_child.stdout.take()`.

5. Spawn the parser: `stdin(Stdio::from(wrapper_stdout))`, `stdout(parser_log_out)`, `stderr(parser_log_err)`, both log clones from `File::create(&ctx.log_path)?`. Call `.process_group(0).spawn()?`.

6. Reap the wrapper in a background thread: `std::thread::spawn(move || { let _ = wrapper_child.wait(); });`.

7. Return `Ok(parser_child)`. APM monitors the parser child for exit.

**Dependency note:** Add `which = "6"` to `apm-core/Cargo.toml` if not already present. As a fallback without the crate: walk `std::env::var("PATH")` entries and check `Path::new(entry).join(parser_cmd).is_file()` for relative names; accept the path as-is when `parser_cmd` starts with `/`.

---

### `validate.rs` — extend `validate_agents`

In the `Ok(Some(WrapperKind::Custom { manifest, .. }))` match arm, after existing manifest checks, add:

```rust
if let Some(m) = &manifest {
    if m.parser == "external" && m.parser_command.is_none() {
        errors.push(format!(
            "agent {name}: manifest.toml declares parser = \"external\" \
             but parser_command is absent"
        ));
    }
}
```

This mirrors the runtime check in `spawn` so `apm validate` catches the misconfiguration before any worker starts.

---

### Tests

**Unit tests in `wrapper/custom.rs` under `#[cfg(test)]`:**

- `parser_strategy_defaults_to_canonical` — `ParserStrategy::from_manifest(None)` equals `Canonical`
- `parser_strategy_explicit_canonical` — manifest with `parser = "canonical"` → `Canonical`
- `parser_strategy_raw` — manifest with `parser = "raw"` → `Raw`
- `parser_strategy_external` — manifest with `parser = "external"` → `External`
- `parser_strategy_unknown_falls_back_to_canonical` — manifest with `parser = "foobar"` → `Canonical`
- `spawn_external_missing_parser_command` — `CustomWrapper` with manifest `parser = "external"`, `parser_command = None`; assert `spawn()` returns `Err` whose message contains `"parser_command"` and `"not set"`
- `spawn_external_binary_not_found` — `parser_command = Some("nonexistent-binary-xyzzy-2803")`; assert `spawn()` returns `Err` naming that binary

**Integration tests in `apm-core/tests/custom_wrapper_integration.rs`** (extend the file introduced by 2c32a282):

- `integration_canonical_mode` — wrapper script emits one valid JSONL line; assert log contains that line verbatim; assert spawn returns `Ok`
- `integration_raw_mode` — wrapper script emits `"hello world\n"` (not JSONL); manifest declares `parser = "raw"`; assert log contains `"hello world"`; assert no line in the log starts with `"[apm] warning:"`
- `integration_external_parser_pipe` — wrapper script emits `"raw line\n"` on stdout and exits 0; a second fixture script (the parser, mode 0o755, `#!/bin/sh`) reads each stdin line and emits `{"text":"<line>"}` on stdout; manifest declares `parser = "external"` and `parser_command` set to the absolute path of the parser fixture (not a PATH name); assert spawn returns `Ok`; wait for the returned parser child to exit 0; read the log; assert log contains the string `raw line` wrapped in JSON

Use absolute paths for `parser_command` in the integration test to avoid depending on test-harness PATH configuration. The `which` crate accepts absolute paths to existing executable files directly.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:05Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-05-01T00:34Z | groomed | in_design | philippepascal |