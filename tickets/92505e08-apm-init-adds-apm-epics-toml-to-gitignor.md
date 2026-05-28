+++
id = "92505e08"
title = "apm init adds .apm/epics.toml to .gitignore even though. epics.toml doesn't exist anymore"
state = "in_design"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/92505e08-apm-init-adds-apm-epics-toml-to-gitignor"
created_at = "2026-05-28T01:48:16.673201Z"
updated_at = "2026-05-28T06:11:26.055455Z"
+++

## Spec

### Problem

`apm init` calls `ensure_gitignore()` in `apm-core/src/init.rs`, which includes `.apm/epics.toml` in its hardcoded `static_entries` list. That entry gets added to `.gitignore` on every fresh `apm init`, even though `.apm/epics.toml` is no longer a file that APM creates or reads anywhere. The file was removed as part of ticket 6e3f9e91, which replaced per-epic `max_workers` overrides with a global `max_workers_per_epic` setting in `[agents]` config.

The stale entry causes two concrete problems: new projects get a confusing `.gitignore` line pointing to a non-existent file, and re-running `apm init` on existing repos adds the entry if it isn't already there. The `README.md` configuration table also still references `epics.toml` as a live file, which is misleading.

### Acceptance criteria

- [ ] `apm init` on a fresh repo does not add `.apm/epics.toml` to `.gitignore`
- [ ] Re-running `apm init` on an existing repo that already has `.apm/epics.toml` in `.gitignore` does not add a second copy of the entry
- [ ] The repo's own `.gitignore` no longer contains `.apm/epics.toml`
- [ ] `README.md` no longer lists `epics.toml` in the configuration files table
- [ ] `cargo test --workspace` passes with no test changes required

### Out of scope

- Retroactively removing `.apm/epics.toml` from `.gitignore` files in existing user repos when `apm init` is re-run (`ensure_gitignore` is append-only; removal logic is a separate concern)
- Cleaning up `epics.toml` references in archived ticket files (those are historical records, not live documentation)
- Any changes to `apm epic` commands or config parsing

### Approach

Three single-line removals across three files. No new code, no migration needed.

**`apm-core/src/init.rs` line 387** — remove `.apm/epics.toml` from `static_entries`:
```rust
// Before
let static_entries = [".apm/local.toml", ".apm/epics.toml", ".apm/*.init", ".apm/sessions.json", ".apm/credentials.json"];

// After
let static_entries = [".apm/local.toml", ".apm/*.init", ".apm/sessions.json", ".apm/credentials.json"];
```

**`.gitignore` line 18** — remove the `.apm/epics.toml` line from the repo's own gitignore (this is the stale entry left over from before ticket 6e3f9e91 was implemented).

**`README.md` line 294** — remove the `epics.toml` row from the configuration files table:
```
| `epics.toml` | Per-epic settings (e.g. `max_workers`) — untracked |
```

No test changes are needed: the existing `ensure_gitignore_creates_file` test does not assert on `.apm/epics.toml`, and all other gitignore tests remain valid.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T01:48Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:09Z | groomed | in_design | philippepascal |