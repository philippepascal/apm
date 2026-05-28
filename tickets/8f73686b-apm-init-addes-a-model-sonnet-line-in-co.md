+++
id = "8f73686b"
title = "apm init addes a model sonnet line in config.toml"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8f73686b-apm-init-addes-a-model-sonnet-line-in-co"
created_at = "2026-05-28T01:54:59.492163Z"
updated_at = "2026-05-28T06:43:26.916948Z"
+++

## Spec

### Problem

`apm init` generates `.apm/config.toml` with `model = "sonnet"` as an active key under `[workers]`. This is wrong: `[workers].model` is a global fallback for all workers, but the intended pattern is to configure model per-agent in the manifest files (`.apm/agents/claude/coder.toml`, `.apm/agents/claude/spec-writer.toml`). Shipping a hardcoded `model = "sonnet"` overrides any deliberate per-agent or per-machine customisation and misleads users into thinking they need to keep it there.

The correct output from `apm init` should omit `model` from `[workers]` entirely (or at most leave it commented out as a hint), leaving model selection to the manifest files where it belongs.

### Acceptance criteria

- [x] `apm init` on a fresh repo produces a `config.toml` that does not contain `model = "sonnet"` (or any active `model =` assignment) under `[workers]`
- [x] The generated `config.toml` contains a commented-out `# model = "sonnet"` line under `[workers]` as a usage hint
- [x] Re-running `apm init` on an existing repo whose `config.toml` already has `model = "sonnet"` does not overwrite that file (idempotency is preserved via the existing `.init` copy mechanism)
- [x] A unit test in `apm-core/src/init.rs` asserts that `default_config(...)` output does not contain an active `model =` assignment under `[workers]`
- [x] All existing tests pass (`cargo test --workspace`)

### Out of scope

- Removing `WorkersConfig.model` from the config struct — the field is valid and users may set it explicitly
- Migrating existing `config.toml` files that already contain `model = "sonnet"` (no active migration is needed; the field continues to work if present)
- Changing model resolution logic in `apm-core/src/start.rs` — the cascade `workers.model` → manifest override is correct
- Updating `SPEC_WRITER_MANIFEST_STUB` or `CODER_MANIFEST_STUB` comments in `init.rs` — they are already accurate

### Approach

**File:** `apm-core/src/init.rs` — `default_config` function (around line 496)

Remove the active `model = "sonnet"` line and replace it with a commented-out hint in the `[workers]` block.

Before (active assignment):
```
[workers]
default = "{workers_default}"
model = "sonnet"
# container = "apm-worker"   ...
```

After (commented hint):
```
[workers]
default = "{workers_default}"
# model = "sonnet"            # default model for all workers; set per-agent in .apm/agents/<agent>/<role>.toml instead
# container = "apm-worker"   ...
```

The format string uses `{{}}` for literal braces in the existing commented lines — keep that unchanged; only the `model = "sonnet"` line itself is removed and added back as a comment.

**Test to add** (inline in the `#[cfg(test)]` block in `apm-core/src/init.rs`):

```rust
#[test]
fn default_config_has_no_active_model_line() {
    let config = default_config("proj", "desc", "main", &[], "claude/coder");
    assert!(
        !config.lines().any(|l| l.trim_start().starts_with("model =")),
        "default_config must not emit an active model = line: {config}"
    );
}
```

This assertion catches any future re-introduction of an active `model =` key regardless of value, not just `"sonnet"`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T01:54Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:11Z | groomed | in_design | philippepascal |
| 2026-05-28T06:13Z | in_design | specd | claude |
| 2026-05-28T06:27Z | specd | ready | philippepascal |
| 2026-05-28T06:31Z | ready | in_progress | philippepascal |
| 2026-05-28T06:37Z | in_progress | implemented | claude |
| 2026-05-28T06:43Z | implemented | closed | philippepascal(apm-sync) |
