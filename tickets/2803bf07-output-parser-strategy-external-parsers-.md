+++
id = "2803bf07"
title = "Output parser strategy: external parsers via manifest.toml"
state = "specd"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2803bf07-output-parser-strategy-external-parsers-"
created_at = "2026-04-30T20:05:40.844536Z"
updated_at = "2026-05-01T03:16:32.545146Z"
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
- Two parser modes:
  - `parser = "canonical"` (default) — wrapper produces JSONL stream-json directly. No transformation. Today's behaviour.
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
- `external` mode: a wrapper that emits non-JSONL text + a parser script that wraps each line in a JSONL event → log contains the parsed JSONL.
- Missing parser_command → spawn fails with a clear error citing the manifest path.

### Acceptance criteria

- [ ] When `parser` is absent from manifest.toml, or manifest.toml itself is absent, `CustomWrapper::spawn` captures the wrapper's stdout directly to the log file (equivalent to `parser = "canonical"`)
- [ ] When `parser = "canonical"`, `CustomWrapper::spawn` captures the wrapper's stdout directly to the log file with no transformation; the wrapper's stderr is also written to the same log file
- [ ] When `parser = "external"` and `parser_command` resolves to an executable binary, `CustomWrapper::spawn` creates an OS-level pipe: the wrapper's stdout is the parser's stdin; the parser's stdout is captured to the log file
- [ ] When `parser = "external"`, the wrapper's stderr is written to the log file independently (not through the parser pipe)
- [ ] When `parser = "external"`, the parser's stderr is also written to the log file
- [ ] When `parser = "external"` and `parser_command` is absent from manifest.toml, `CustomWrapper::spawn` returns `Err` before spawning any process; the error message names the manifest file path and states that `parser_command` is required
- [ ] When `parser = "external"` and `parser_command` names a binary not found in PATH and is not an absolute path to an existing file, `CustomWrapper::spawn` returns `Err` before spawning any process; the error message names the missing binary
- [ ] `apm validate` reports an error when a custom wrapper's manifest.toml declares `parser = "external"` but `parser_command` is absent
- [ ] Built-in wrappers (e.g. the `claude` built-in) always behave as `parser = "canonical"` regardless of any manifest file; no manifest is required or consulted for them
- [ ] `CustomWrapper::spawn` for external mode returns the parser's `Child` handle; the wrapper child is reaped in a background thread so it does not become a zombie
- [ ] When `parser = "external"`, the worker's exit status is taken from the parser's exit code; the wrapper's exit code is appended to the log file as a diagnostic line (e.g. `[apm] wrapper exited: exit status: 0`) but does not affect ticket state; if the wrapper exits non-zero before the parser has drained its stdin, the parser is allowed to finish naturally before APM reaps both
- [ ] When `parser = "external"`, all three streams (parser stdout, parser stderr, wrapper stderr) are written to `.apm-worker.log` without truncation, even when one stream produces output much faster than another; ordering between streams is best-effort but no bytes from any stream may be dropped

### Out of scope

- Shipping external parser binaries (e.g. `apm-output-parser-aider`, `apm-output-parser-codex`) — those are separate cargo crates distributed and installed independently
- Defining the canonical JSONL event vocabulary — already noted as an open question in `docs/agent-wrappers.md`; any parser binary this ticket tests against must emit valid JSONL but the schema is not defined here
- Multiplexing more than one parser per wrapper; manifest.toml accepts exactly one `parser` value and one optional `parser_command`
- In-wrapper translation (`parser = "canonical"` where the wrapper itself transforms output inline) — that is wrapper-author responsibility; APM does not assist with it
- `parser = "raw"` mode (verbatim pass-through capture without JSONL parsing) — projects that need this today can use `parser = "external"` with `parser_command = "cat"`; a dedicated `raw` mode may be added in a follow-up ticket after `docs/agent-wrappers.md` is updated to define it
- The `apm agents test <name>` command for smoke-testing parser compliance — ticket 71d80e40
- Per-ticket parser override via frontmatter `agent_overrides` — ticket 0ca3e019
- Non-Unix platform differences in subprocess piping (Windows `Stdio::from(ChildStdout)` semantics) — out of scope for now; the implementation targets Unix

### Approach

Wire the `parser` and `parser_command` fields (already parsed from manifest.toml by ticket 2c32a282) into `CustomWrapper::spawn`. Add a `ParserStrategy` enum that dispatches between two modes: `canonical` redirects wrapper stdout directly to the log file; `external` spawns an OS-level pipe so wrapper stdout feeds parser stdin and the parser's stdout is captured to the log. Add `which`-based pre-validation for the parser binary. Extend `validate_agents` to catch a missing `parser_command` at config-check time.

**Files changed**

| File | Change |
|---|---|
| `apm-core/src/wrapper/custom.rs` | Add `ParserStrategy` enum; refactor `CustomWrapper::spawn` to dispatch by strategy; implement OS-level pipe for `external` mode |
| `apm-core/src/validate.rs` | Extend `validate_agents` to push an error when `parser = "external"` and `parser_command` is absent |
| `apm-core/tests/custom_wrapper_integration.rs` | Add two integration tests covering canonical and external modes |
| `docs/agent-wrappers.md` | Verify/update TOML examples in "Custom wrappers / manifest.toml" and "Output parser strategy" sections to list exactly the two supported parser modes (`canonical`, `external`) and remove any modes that are not part of this implementation |

---

### `wrapper/custom.rs` -- `ParserStrategy` enum

Add a private enum above `CustomWrapper`:

```rust
#[derive(Debug, Clone, PartialEq)]
enum ParserStrategy { Canonical, External }

impl ParserStrategy {
    fn from_manifest(m: Option<&Manifest>) -> Self {
        match m.and_then(|m| Some(m.parser.as_str())) {
            Some("external") => Self::External,
            _                => Self::Canonical,   // absent, "canonical", or any unrecognised value
        }
    }
}
```

---

### `wrapper/custom.rs` -- refactor `CustomWrapper::spawn`

After the existing `check_contract_version(...)` call and the env-var block (both established by prior tickets), derive the strategy and branch:

```rust
let strategy = ParserStrategy::from_manifest(self.manifest.as_ref());
```

**Canonical (stdout -> log directly):**

Keep the existing spawn sequence unchanged for this mode:
- `File::create(&ctx.log_path)?` -> `log_file`; `log_file.try_clone()?` -> `log_clone`
- `Command::new(&self.script_path).envs(...).current_dir(...).stdout(log_file).stderr(log_clone).process_group(0).spawn()`

**External -- validate, then pipe:**

1. Derive the manifest path for error messages: `self.script_path.parent().unwrap().join("manifest.toml")`.

2. Require `parser_command`: call `.ok_or_else(|| anyhow!("...: parser = \"external\" but parser_command is not set"))` on `self.manifest.as_ref().and_then(|m| m.parser_command.as_deref())`. Return `Err` immediately if absent -- no subprocess is started.

3. Validate the binary is findable before spawning the wrapper. Use `which::which(parser_cmd)` (see dependency note below). Return `Err` naming the missing binary if not found. Again, no subprocess is started yet.

4. Spawn the wrapper with `stdout(Stdio::piped())` and `stderr(log_clone)` (wrapper stderr goes directly to the log). Call `.process_group(0).spawn()?`. Take `wrapper_child.stdout.take()`.

5. Spawn the parser: `stdin(Stdio::from(wrapper_stdout))`, `stdout(parser_log_out)`, `stderr(parser_log_err)`, both log clones from `File::create(&ctx.log_path)?`. Call `.process_group(0).spawn()?`.

6. Reap the wrapper in a background thread that waits for it, then appends a diagnostic line to the log: `[apm] wrapper exited: <status>`. The wrapper exit code is informational only and does not affect ticket state.

7. Return `Ok(parser_child)`. APM monitors the parser child for exit; the parser's exit code is the worker's exit status.

**Dependency note:** Add `which = "6"` to `apm-core/Cargo.toml` if not already present. As a fallback without the crate: walk `std::env::var("PATH")` entries and check `Path::new(entry).join(parser_cmd).is_file()` for relative names; accept the path as-is when `parser_cmd` starts with `/`.

---

### `validate.rs` -- extend `validate_agents`

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

- `parser_strategy_defaults_to_canonical` -- `ParserStrategy::from_manifest(None)` equals `Canonical`
- `parser_strategy_explicit_canonical` -- manifest with `parser = "canonical"` -> `Canonical`
- `parser_strategy_external` -- manifest with `parser = "external"` -> `External`
- `parser_strategy_unknown_falls_back_to_canonical` -- manifest with `parser = "foobar"` -> `Canonical`
- `spawn_external_missing_parser_command` -- `CustomWrapper` with manifest `parser = "external"`, `parser_command = None`; assert `spawn()` returns `Err` whose message contains `"parser_command"` and `"not set"`
- `spawn_external_binary_not_found` -- `parser_command = Some("nonexistent-binary-xyzzy-2803")`; assert `spawn()` returns `Err` naming that binary

**Integration tests in `apm-core/tests/custom_wrapper_integration.rs`** (extend the file introduced by 2c32a282):

- `integration_canonical_mode` -- wrapper script emits one valid JSONL line; assert log contains that line verbatim; assert spawn returns `Ok`
- `integration_external_parser_pipe` -- wrapper script emits one line of non-JSONL text on stdout and exits 0; a second fixture script (the parser, mode 0o755, `#!/bin/sh`) reads each stdin line and emits a JSONL object wrapping the line on stdout; manifest declares `parser = "external"` and `parser_command` set to the absolute path of the parser fixture (not a PATH name); assert spawn returns `Ok`; wait for the returned parser child to exit 0; read the log; assert log contains the input text wrapped in JSON

Use absolute paths for `parser_command` in the integration test to avoid depending on test-harness PATH configuration. The `which` crate accepts absolute paths to existing executable files directly.

### Open questions


### Amendment requests

- [x] Drop the `raw` parser mode from this ticket entirely. The reference spec at `docs/agent-wrappers.md` defines three strategies (canonical, in-wrapper translation, external) and never mentions a `raw` mode. Adding it here without first updating the design doc creates a documentation drift and adds a third behaviour to validate that nobody asked for. Anyone wanting raw output today can use `parser = "external"` with `parser_command = "cat"` (or any pass-through). If `raw` proves genuinely useful later, file a follow-up ticket that updates the design doc first. Remove the `raw` AC, the `raw` integration test, and any Approach text describing it. **Verification: after the amendment round, the strings `raw`, `Raw`, and `parser_strategy::Raw` must not appear anywhere in the Problem, Acceptance criteria, or Approach sections.**
- [x] Clarify child exit-code semantics for `parser = "external"`. The wrapper and the parser are two children connected by a pipe (wrapper.stdout → parser.stdin). Add an AC: "the worker's exit status is taken from the **parser's** exit code; the wrapper's exit code is logged for diagnostics but does not affect ticket state. If the wrapper exits non-zero before the parser drains, the parser is allowed to finish naturally before APM reaps both." This removes ambiguity for the implementer about which Child handle to wait on. **Verification: after the amendment round, an AC line containing the substring "parser's exit code" must exist in the Acceptance criteria section.**
- [x] Add an AC for stream-capture loss-prevention: "all three streams (parser.stdout, parser.stderr, wrapper.stderr) are written to `.apm-worker.log` without truncation, even when one stream produces output much faster than another. Ordering is best-effort; lossless capture is required." This guards against a real bug class (pipe buffer races) while staying realistic about ordering guarantees. **Verification: after the amendment round, an AC line containing the substring "without truncation" must exist in the Acceptance criteria section.**
- [x] Update the TOML example in `docs/agent-wrappers.md` (sections "Custom wrappers / manifest.toml" and "Output parser strategy") to mention all parser modes the implementation will support — currently `canonical` and `external`. This keeps the spec doc in sync with the implementation. Out of scope for this ticket: no other doc changes.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:05Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-05-01T00:34Z | groomed | in_design | philippepascal |
| 2026-05-01T00:42Z | in_design | specd | claude-0501-0034-4620 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:37Z | ammend | in_design | philippepascal |
| 2026-05-01T02:31Z | in_design | specd | philippepascal |
| 2026-05-01T02:53Z | specd | ammend | philippepascal |
| 2026-05-01T02:53Z | ammend | in_design | philippepascal |
| 2026-05-01T03:00Z | in_design | specd | claude-0501-0253-bf98 |
| 2026-05-01T03:08Z | specd | ammend | philippepascal |
| 2026-05-01T03:09Z | ammend | in_design | philippepascal |
| 2026-05-01T03:16Z | in_design | specd | claude-0501-0309-1140 |
