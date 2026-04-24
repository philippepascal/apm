+++
id = "2e832569"
title = "apm init re-run reports false diff for user-filled fields"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2e832569-apm-init-re-run-reports-false-diff-for-u"
created_at = "2026-04-24T06:28:08.866116Z"
updated_at = "2026-04-24T08:01:27.571518Z"
+++

## Spec

### Problem

When `apm init` is run on a project that already has `.apm/config.toml`, the re-run is supposed to detect whether the live config has drifted from the current default template. If it has drifted, `apm init` writes `.apm/config.toml.init` so the user can compare the two files and decide whether to adopt any new defaults.

The bug: `setup()` at `apm-core/src/init.rs:116` reconstructs the default config by extracting `project.name`, `project.description`, and `project.default_branch` from the live file — but hardcodes `collaborators = &[]`. Because `default_config()` serializes that as `collaborators = []`, the reconstructed default never matches the live file when the user has a non-empty collaborators list (e.g. `collaborators = ["philippepascal"]`). This causes a spurious `.apm/config.toml.init` to be produced on every re-run, even when the live config has never been touched by the user.

The affected users are anyone whose collaborators list was populated during initial interactive setup (i.e. when `apm init` ran with a detected Git username). Every subsequent re-run reports a false diff, which erodes trust in the `.init` signal.

### Acceptance criteria

- [x] Re-running `apm init` on a project whose `.apm/config.toml` was created with a non-empty collaborators list (e.g. `collaborators = ["alice"]`) and has not been modified since does NOT produce `.apm/config.toml.init`
- [x] Re-running `apm init` on a project whose `.apm/config.toml` has been manually edited (e.g. a `[custom]` section added) DOES produce `.apm/config.toml.init`
- [x] The `.apm/config.toml.init` produced in the case above contains the same `collaborators` value as the live config (not an empty array)
- [x] Re-running `apm init` on a project whose `.apm/config.toml` has `collaborators = []` does not produce `.apm/config.toml.init` when no other changes exist

### Out of scope

- Other user-editable fields (`logging.enabled`, `agents.max_concurrent`, etc.) are not normalized out of the diff; if the user changes them the `.init` signal fires correctly
- The interactive TTY path (`apm init` with a live terminal prompting for username) is not changed
- Normalizing diffs for `workflow.toml`, `ticket.toml`, `agents.md`, or other managed files
- Surfacing or formatting the diff to the user (that is a separate UX concern)

### Approach

**File:** `apm-core/src/init.rs`

**Change (lines 103–116):** In the `else` branch of `setup()` (where `config.toml` already exists), extract `project.collaborators` from the parsed TOML alongside `name`, `description`, and `default_branch`, then pass those collaborators to `default_config()` instead of the hardcoded `&[]`.

```rust
// Existing extractions (unchanged):
let n = val.get("project")…unwrap_or("project");
let d = val.get("project")…unwrap_or("");
let b = val.get("project")…unwrap_or("main");

// New: extract collaborators from the live config
let collab_owned: Vec<String> = val
    .get("project")
    .and_then(|p| p.get("collaborators"))
    .and_then(|v| v.as_array())
    .map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_owned()))
            .collect()
    })
    .unwrap_or_default();
let collabs: Vec<&str> = collab_owned.iter().map(|s| s.as_str()).collect();

write_default(&config_path, &default_config(n, d, b, &collabs), ".apm/config.toml", &mut messages)?;
```

**Test:** Add a new test `setup_no_false_diff_when_collaborators_present` in the existing `#[cfg(test)]` block:
- Call `setup(tmp.path(), None, None, Some("alice"))` — creates config with `collaborators = ["alice"]`
- Call `setup(tmp.path(), None, None, None)` — re-run without username (simulates non-TTY)
- Assert `.apm/config.toml.init` does NOT exist

The existing test `setup_writes_config_init_when_modified` continues to pass unchanged (it relies on the `[custom]` section being the diff trigger, which is unaffected by this change).

No other files change. No public API changes. Fully backward-compatible.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |
| 2026-04-24T07:16Z | in_design | specd | claude-0424-0714-6040 |
| 2026-04-24T07:25Z | specd | ready | philippepascal |
| 2026-04-24T07:32Z | ready | in_progress | philippepascal |
| 2026-04-24T07:36Z | in_progress | implemented | claude-0424-0732-0108 |
| 2026-04-24T08:01Z | implemented | closed | philippepascal(apm-sync) |
