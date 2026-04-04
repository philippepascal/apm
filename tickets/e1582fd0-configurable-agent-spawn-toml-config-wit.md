+++
id = "e1582fd0"
title = "Configurable agent spawn: TOML config with local overrides replaces hardcoded Command"
state = "closed"
priority = 7
effort = 5
risk = 3
author = "apm"
branch = "ticket/e1582fd0-configurable-agent-spawn-toml-config-wit"
created_at = "2026-04-03T21:53:31.381487Z"
updated_at = "2026-04-04T06:02:19.819474Z"
+++

## Spec

### Problem

The worker spawn command is hardcoded in `apm-core/src/start.rs`. Three nearly identical blocks (in `run`, `run_next`, `spawn_next_worker`) each build `Command::new("claude")` with `--print`, `--system-prompt`, and optionally `--dangerously-skip-permissions`. Users cannot:

- Change the model (`--model opus`)
- Add extra CLI flags or env vars
- Swap `claude` for a different agent CLI (Codex, Aider, custom wrapper)
- Override per-machine without recompiling

The container path (`docker run ... claude`) has the same problem.

The fix is to move the spawn command definition into tracked TOML config (`[workers]` in `.apm/agents.toml` or `workflow.toml`) with per-machine overrides via a gitignored `local.toml`. apm reads the config and builds the `Command` at runtime — no shell scripts, no OS-specific files, cross-platform by default.

### Acceptance criteria

- [x] `WorkersConfig` gains `command: Option<String>` (default `"claude"`), `args: Vec<String>` (default `["--print"]`), `model: Option<String>`, and `env: HashMap<String, String>`
- [x] When `workers.command` is set in tracked config, `apm start --spawn` uses it instead of hardcoded `"claude"`
- [x] When `workers.model` is set, `--model <value>` is prepended to the args passed to the agent CLI
- [x] When `workers.env` contains entries, each is injected as an env var on the spawned process
- [x] `apm init` writes a default `[workers]` section with `command = "claude"` and `args = ["--print"]` into the tracked config
- [x] A `.apm/local.toml` file (gitignored) can contain `[workers]` with the same fields; values in `local.toml` override/extend the tracked config
- [x] `apm init` adds `.apm/local.toml` to `.gitignore` if not already present
- [x] The three native spawn sites (`run`, `run_next`, `spawn_next_worker`) are consolidated into a single `build_spawn_command` function that reads the merged config
- [x] The container spawn path (`spawn_container_worker`) is unchanged by this ticket
- [x] Existing behavior with no config changes is identical to current hardcoded behavior (backward compatible)

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

#### 1. `apm-core/src/config.rs` — extend `WorkersConfig`

Add fields to the existing `WorkersConfig`:

```rust
pub struct WorkersConfig {
    pub container: Option<String>,
    #[serde(default)]
    pub keychain: std::collections::HashMap<String, String>,
    #[serde(default = "default_command")]
    pub command: String,
    #[serde(default = "default_args")]
    pub args: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_command() -> String { "claude".to_string() }
fn default_args() -> Vec<String> { vec!["--print".to_string()] }
```

When `container` is `Some`, the container path is used (unchanged). When `container` is `None`, the new fields drive the native spawn.

#### 2. `apm-core/src/config.rs` — add `LocalConfig` and merge logic

Add a minimal struct for `.apm/local.toml`:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct LocalConfig {
    #[serde(default)]
    pub workers: LocalWorkersOverride,
}

#[derive(Debug, Deserialize, Default)]
pub struct LocalWorkersOverride {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}
```

Add `Config::load_local(root: &Path) -> Option<LocalConfig>` that reads `.apm/local.toml` if present. Add `WorkersConfig::merge_local(&mut self, local: &LocalWorkersOverride)` that overrides non-None fields and extends the env map.

Call `merge_local` in `Config::load` after loading the tracked config, so all downstream code sees the merged result transparently.

#### 3. `apm-core/src/start.rs` — extract `build_spawn_command`

Create a single function that replaces the three identical native-spawn blocks:

```rust
fn build_spawn_command(
    config: &Config,
    wt: &Path,
    worker_name: &str,
    worker_system: &str,
    ticket_content: &str,
    skip_permissions: bool,
    log_path: &Path,
) -> Result<std::process::Child> {
    let wc = &config.workers;
    let mut cmd = std::process::Command::new(&wc.command);
    for arg in &wc.args {
        cmd.arg(arg);
    }
    if let Some(ref model) = wc.model {
        cmd.args(["--model", model]);
    }
    cmd.args(["--system-prompt", worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(ticket_content);
    cmd.env("APM_AGENT_NAME", worker_name);
    for (k, v) in &wc.env {
        cmd.env(k, v);
    }
    cmd.current_dir(wt);

    let log_file = std::fs::File::create(log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);
    cmd.process_group(0);

    Ok(cmd.spawn()?)
}
```

Replace the native-spawn blocks in `run` (line ~263), `run_next` (line ~416), and `spawn_next_worker` (line ~570) with calls to `build_spawn_command`. The container path (`spawn_container_worker`) is unchanged.

#### 4. `apm/src/cmd/init.rs` — write defaults and gitignore

- In the init flow, write `[workers]\ncommand = "claude"\nargs = ["--print"]\n` into the tracked config file if the `[workers]` section is absent
- Add `.apm/local.toml` to `.gitignore` if not already present

#### 5. Tests

Unit tests in `apm-core/src/config.rs`:
- Parse `WorkersConfig` with all new fields set
- Parse `WorkersConfig` with no fields (verify defaults: command="claude", args=["--print"])
- Parse `LocalConfig` with `[workers]` override
- `merge_local` overrides command, extends env, leaves model None when not in local

Unit test in `apm-core/src/start.rs`:
- `build_spawn_command` is called with custom command/args from config (test that the function constructs correctly — can verify by inspecting the Command struct, or test end-to-end with a dummy script)

#### Order of changes

1. Extend `WorkersConfig` with new fields + defaults
2. Add `LocalConfig` struct + load + merge
3. Extract `build_spawn_command` in `start.rs`, replace three call sites
4. Update `init.rs` for defaults + gitignore
5. Add tests
6. Run `cargo test --workspace`

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-03T21:53Z | — | new | apm |
| 2026-04-03T21:54Z | new | groomed | apm |
| 2026-04-03T21:54Z | groomed | in_design | apm |
| 2026-04-03T21:58Z | in_design | specd | apm |
| 2026-04-03T21:58Z | specd | ready | apm |
| 2026-04-03T21:58Z | ready | in_progress | philippepascal |
| 2026-04-03T22:05Z | in_progress | implemented | apm |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
