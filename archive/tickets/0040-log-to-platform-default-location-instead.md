+++
id = 40
title = "Log to platform default location instead of project dir"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0328-1000-a1b2"
agent = "claude-0328-impl-a1b2"
branch = "ticket/0040-log-to-platform-default-location-instead"
created_at = "2026-03-28T08:11:44.464120Z"
updated_at = "2026-03-28T08:44:25.906896Z"
+++

## Spec

### Problem

The current `[logging]` config requires a `file` path relative to the project
directory (e.g. `apm.log` at the repo root). This is wrong for two reasons:

1. Log files belong in the OS-defined log directory, not alongside source code.
   Committing or accidentally staging them is a real risk.
2. On macOS the conventional location is `~/Library/Logs/<app>/`; on Linux it
   is `~/.local/share/<app>/` (XDG) or `~/.cache/<app>/`. Deviating from this
   makes logs hard to find and excludes them from standard log rotation.

The `file` key in `apm.toml` should be removed. Instead, apm resolves the log
path automatically based on the platform, namespaced by project name so logs
from multiple repos don't collide.

### Acceptance criteria

- [x] When `enabled = true`, apm writes logs to the platform default location:
  - macOS: `~/Library/Logs/apm/<project-name>.log`
  - Linux: `${XDG_STATE_HOME:-~/.local/state}/apm/<project-name>.log`
- [x] The directory is created automatically if it does not exist
- [x] The `file` key is removed from `LoggingConfig` and from the `apm init`
  default template; `[logging]` only needs `enabled = true/false`
- [x] `apm init` no longer writes `file = "apm.log"` in the generated `apm.toml`
- [x] Existing `apm.toml` files that still contain a `file` key continue to
  parse without error (serde `#[serde(default)]` + ignore unknown, or keep the
  field as `Option<PathBuf>` and ignore it with a deprecation note in code)
- [x] `apm verify` prints the resolved log path when logging is enabled, so
  users can find their logs

### Out of scope

- Windows support
- Log rotation or size limits
- Structured / JSON log format
- Configuring a custom log path via `apm.toml` (use a symlink if needed)

### Approach

**`apm-core/src/config.rs`**: change `LoggingConfig`:

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    // Deprecated: ignored. Path is resolved automatically.
    #[serde(default)]
    pub file: Option<std::path::PathBuf>,
}
```

**`apm-core/src/logger.rs`**: add a `default_log_path(project_name: &str) -> PathBuf` function:

```rust
pub fn default_log_path(project_name: &str) -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home)
            .join("Library/Logs/apm")
            .join(format!("{project_name}.log"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var("XDG_STATE_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                std::path::PathBuf::from(home).join(".local/state")
            });
        base.join("apm").join(format!("{project_name}.log"))
    }
}
```

**`apm/src/main.rs`**: update the logging init block to use `default_log_path`:

```rust
if config.logging.enabled {
    let log_path = apm_core::logger::default_log_path(&config.project.name);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let agent = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".to_string());
    apm_core::logger::init(&root, &log_path.to_string_lossy(), &agent);
}
```

**`apm/src/cmd/init.rs`**: remove `file = "apm.log"` from the default config template.

**`apm/src/cmd/verify.rs`**: when logging is enabled, print the resolved path.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:11Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T08:12Z | new | specd | claude-0328-1000-a1b2 |
| 2026-03-28T08:15Z | specd | ready | apm |
| 2026-03-28T08:23Z | ready | in_progress | claude-0328-impl-a1b2 |
| 2026-03-28T08:36Z | in_progress | implemented | claude-0328-impl-a1b2 |
| 2026-03-28T08:44Z | implemented | closed | claude-0328-impl-a1b2 |