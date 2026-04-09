+++
id = 43
title = "Make log file path configurable via apm.toml file key"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0328-1000-a1b2"
agent = "claude-0328-t43a"
branch = "ticket/0043-make-log-file-path-configurable-via-apm-"
created_at = "2026-03-28T08:58:38.222881Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

Ticket #40 added `default_log_path()` which resolves the log path automatically
from the platform convention (macOS: `~/Library/Logs/apm/<project>.log`, Linux:
`~/.local/state/apm/<project>.log`). The resolved path is used unconditionally —
there is no way for a user to override it in `apm.toml`.

The `file` field already exists in `LoggingConfig` as `Option<PathBuf>` but is
completely ignored in `main.rs`. Users who want logs in a custom location have
no recourse.

Additionally, `apm init` writes only `enabled = false` in the generated
`[logging]` block — the user has no idea what path will be used when they flip
the flag. Writing the platform default explicitly as `file = "..."` in the
generated config makes the default discoverable and immediately editable.

### Acceptance criteria

- [x] When `[logging]` has a `file` key, that path is used for the log file
  (after tilde expansion — `~/...` → `$HOME/...`)
- [x] When `file` is absent, the platform default from `default_log_path()` is
  used (existing behavior — no regression)
- [x] `apm init` writes a platform-aware default into the generated `apm.toml`:
  - macOS: `file = "~/Library/Logs/apm/<project-name>.log"`
  - Linux: `file = "~/.local/state/apm/<project-name>.log"`
- [x] `apm verify` prints the resolved log path when logging is enabled, using
  `resolve_log_path` so it reflects any `file` override
- [x] A unit test covers tilde expansion: `~/foo.log` → `<HOME>/foo.log`

### Out of scope

- Environment variable expansion beyond `~` (no `$VAR` substitution in paths)
- Windows support
- Log rotation or size limits

### Approach

**`apm-core/src/logger.rs`**: add a `resolve_log_path` helper that takes an
`Option<&Path>` (the `file` override) and falls back to `default_log_path`:

```rust
pub fn resolve_log_path(project_name: &str, override_path: Option<&std::path::Path>) -> std::path::PathBuf {
    if let Some(p) = override_path {
        expand_tilde(p)
    } else {
        default_log_path(project_name)
    }
}

fn expand_tilde(path: &std::path::Path) -> std::path::PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home).join(rest)
    } else {
        path.to_path_buf()
    }
}
```

**`apm/src/main.rs`**: update logging init to pass `config.logging.file`:

```rust
if config.logging.enabled {
    let log_path = apm_core::logger::resolve_log_path(
        &config.project.name,
        config.logging.file.as_deref(),
    );
    // ...
}
```

**`apm/src/cmd/init.rs`**: update `default_config()` to include a platform-aware
`file` line in the `[logging]` block using `#[cfg(target_os = "macos")]`.

**`apm/src/cmd/verify.rs`**: update to use `resolve_log_path` so the printed
path reflects any `file` override from config.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:58Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T09:03Z | new | specd | apm |
| 2026-03-28T19:18Z | specd | ready | apm |
| 2026-03-28T19:24Z | ready | in_progress | claude-0328-t43a |
| 2026-03-28T19:26Z | in_progress | implemented | claude-0328-t43a |
| 2026-03-28T19:29Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |