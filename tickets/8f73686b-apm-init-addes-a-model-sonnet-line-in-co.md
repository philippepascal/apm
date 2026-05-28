+++
id = "8f73686b"
title = "apm init addes a model sonnet line in config.toml"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8f73686b-apm-init-addes-a-model-sonnet-line-in-co"
created_at = "2026-05-28T01:54:59.492163Z"
updated_at = "2026-05-28T06:11:39.276663Z"
+++

## Spec

### Problem

`apm init` generates `.apm/config.toml` with `model = "sonnet"` as an active key under `[workers]`. This is wrong: `[workers].model` is a global fallback for all workers, but the intended pattern is to configure model per-agent in the manifest files (`.apm/agents/claude/coder.toml`, `.apm/agents/claude/spec-writer.toml`). Shipping a hardcoded `model = "sonnet"` overrides any deliberate per-agent or per-machine customisation and misleads users into thinking they need to keep it there.

The correct output from `apm init` should omit `model` from `[workers]` entirely (or at most leave it commented out as a hint), leaving model selection to the manifest files where it belongs.

### Acceptance criteria

- [ ] `apm init` on a fresh repo produces a `config.toml` that does not contain `model = "sonnet"` (or any active `model =` assignment) under `[workers]`
- [ ] The generated `config.toml` contains a commented-out `# model = "sonnet"` line under `[workers]` as a usage hint
- [ ] Re-running `apm init` on an existing repo whose `config.toml` already has `model = "sonnet"` does not overwrite that file (idempotency is preserved via the existing `.init` copy mechanism)
- [ ] A unit test in `apm-core/src/init.rs` asserts that `default_config(...)` output does not contain an active `model =` assignment under `[workers]`
- [ ] All existing tests pass (`cargo test --workspace`)

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
| 2026-05-28T01:54Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:11Z | groomed | in_design | philippepascal |