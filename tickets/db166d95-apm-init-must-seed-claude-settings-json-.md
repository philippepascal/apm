+++
id = "db166d95"
title = "apm init must seed .claude/settings.json with worker-essential allow-list"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/db166d95-apm-init-must-seed-claude-settings-json-"
created_at = "2026-05-15T01:21:17.353568Z"
updated_at = "2026-05-22T02:33:07.490285Z"
+++

## Spec

### Problem

`apm init` calls `update_claude_settings()` and `update_user_claude_settings()` (in `apm/src/cmd/init.rs`) to seed `.claude/settings.json` with the `APM_ALLOW_ENTRIES` allow-list that every worker needs. Two bugs prevent this from working reliably.

First, `update_claude_settings` passes `create_if_missing: false` to `update_settings_json` (line 283). If `.claude/` exists but `settings.json` does not, `update_settings_json` returns early at line 218 — the file is never created and the allow-list is never written.

Second, `update_settings_json` prompts the user `[y/N]` before writing (lines 232–244). In any non-TTY context — CI pipelines, headless worker spawning — `read_line` gets EOF and the function prints "Skipped." without writing anything. Workers spawned by `apm start --spawn` therefore hit the permission gate on every `apm spec`, `apm state`, and `apm show` call. Because `apm state <id> blocked` is also gated, the worker cannot even self-report the failure — a recursive trap.

### Acceptance criteria

- [x] `apm init --yes` on a repo with `.claude/` present but no `settings.json` creates `.claude/settings.json` containing all `APM_ALLOW_ENTRIES` under `permissions.allow` without prompting.
- [x] `apm init --yes` on a repo with an existing `.claude/settings.json` that is missing some entries merges the missing entries in without duplicating entries that are already present.
- [ ] `apm init --yes` on a repo with no `.claude/` directory does not create `.claude/` or `settings.json`, and exits zero.
- [ ] `apm init` (no `--yes`) on a non-TTY stdin with `.claude/settings.json` absent still creates and seeds the file (non-interactive path does not prompt).
- [ ] `apm init --yes` updates `~/.claude/settings.json` with `APM_USER_ALLOW_ENTRIES` without prompting.
- [ ] `apm init --yes` prints `Updated .claude/settings.json` when the project file was created or modified, and prints `Updated ~/.claude/settings.json` when the user file was modified.
- [ ] `apm init --no-claude` still suppresses all settings.json writes even when `--yes` is passed.

### Out of scope

- Backfilling repos that ran a pre-fix `apm init` (migration command is a future concern).\n- Touching `settings.local.json` — that file is per-engineer and must not be written by `apm init`.\n- Creating the `.claude/` directory if it does not already exist.\n- Changing the interactive TTY flow — when stdin is a TTY and `--yes` is not passed, the existing [y/N] prompt is preserved.

### Approach

All changes are in `apm/src/cmd/init.rs`.

#### 1. Add `--yes` flag to `apm init`

In `main.rs`, add a `yes` field to the `Command::Init` variant:

```rust
/// Add allow-list entries without prompting
#[arg(long, short = 'y')]
yes: bool,
```

Pass it through the dispatch arm to `cmd::init::run`:

```rust
Command::Init { no_claude, migrate, with_docker, quiet, yes } =>
    cmd::init::run(&root, no_claude, migrate, with_docker, quiet, yes),
```

Update `run`'s signature in `init.rs` accordingly:

```rust
pub fn run(root: &Path, no_claude: bool, migrate: bool, with_docker: bool, quiet: bool, yes: bool) -> Result<()>
```

#### 2. Thread `yes` into `update_settings_json`

Add a `yes: bool` parameter to `update_settings_json`. When `yes` is `true`, skip the `[y/N]` prompt and proceed directly to writing. When `yes` is `false`, preserve the existing prompt behaviour.

```rust
fn update_settings_json(
    path: &Path,
    entries: &[&str],
    prompt_header: &str,
    prompt_confirm: &str,
    updated_msg: &str,
    create_if_missing: bool,
    yes: bool,
) -> Result<()> {
    // ... existing load logic ...
    if missing.is_empty() { return Ok(()); }

    if yes {
        // skip prompt, fall through to write
    } else {
        println!("{prompt_header}");
        for e in &missing { println!("  {e}"); }
        print!("{prompt_confirm} [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Skipped.");
            return Ok(());
        }
    }
    // ... existing write logic ...
}
```

#### 3. Fix `update_claude_settings` to create when `.claude/` exists

Replace the hard-coded `create_if_missing: false` with a runtime check: if `.claude/` directory exists, pass `true`; otherwise return early without calling `update_settings_json` at all.

```rust
fn update_claude_settings(root: &Path, skip: bool, yes: bool) -> Result<()> {
    if skip { return Ok(()); }
    let claude_dir = root.join(".claude");
    if !claude_dir.exists() {
        return Ok(());   // .claude/ absent — no-op
    }
    update_settings_json(
        &claude_dir.join("settings.json"),
        APM_ALLOW_ENTRIES,
        "The following entries will be added to .claude/settings.json permissions.allow:",
        "Add apm commands to Claude allow list?",
        "Updated .claude/settings.json",
        true,   // .claude/ exists, so create the file if missing
        yes,
    )
}
```

#### 4. Thread `yes` into `update_user_claude_settings`

```rust
fn update_user_claude_settings(yes: bool) -> Result<()> { ... }
```

Pass `yes` through to `update_settings_json`'s new parameter.

#### 5. Call-site wiring

In `run()`, thread `yes` to both helpers:

```rust
update_claude_settings(root, no_claude, yes)?;
update_user_claude_settings(yes)?;
```

#### 6. Non-TTY auto-yes (AC item 4)

When stdin is not a terminal and `--yes` was not explicitly passed, treat it as `yes = true` automatically. Add this near the top of `run()`, after the existing `is_tty` binding:

```rust
let yes = yes || !is_tty;
```

This means a non-interactive `apm init` (e.g. piped or run by a script) always writes without prompting, matching the intent of the ticket even without an explicit flag.

#### 7. Update `warn_if_settings_untracked`

No change needed — it already guards with `if !settings.exists()`.

#### 8. Update `--help` long_about for `Init`

Add a note to the long_about string:
```
Pass --yes (or -y) to skip the [y/N] confirmation prompts. This is implied
when stdin is not a terminal.
```

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-15T01:21Z | — | new | philippe|philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:08Z | groomed | in_design | philippepascal |
| 2026-05-21T23:10Z | in_design | specd | claude-0521-2308-b018 |
| 2026-05-22T02:25Z | specd | ready | philippepascal |
| 2026-05-22T02:33Z | ready | in_progress | philippepascal |