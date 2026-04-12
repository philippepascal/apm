+++
id = "de5c5a31"
title = "Deduplicate init.rs settings manipulation functions"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de5c5a31-deduplicate-init-rs-settings-manipulatio"
created_at = "2026-04-12T09:02:35.167384Z"
updated_at = "2026-04-12T09:14:26.803236Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

`apm/src/cmd/init.rs` contains two near-identical private functions for patching `.claude/settings.json` files:

- `update_claude_settings(root, skip)` — patches the project-level `.claude/settings.json` with `APM_ALLOW_ENTRIES`
- `update_user_claude_settings()` — patches the user-level `~/.claude/settings.json` with `APM_USER_ALLOW_ENTRIES`

Both functions share ~60 lines of identical logic: read JSON (or create an empty object), navigate to `/permissions/allow`, diff against a target entry list, prompt the user, ensure the array path exists, append entries, and write back. The combined duplication spans ~140 lines across 305 total.

The only differences between them are: the resolved file path, the entry list, the prompt/confirmation strings, and whether a missing file causes an early-return (project case) or bootstraps an empty object (user case). All four differences are straightforward to parameterise.

The desired state is a single `fn update_settings_json(...)` helper that both callers delegate to, reducing the file by ~65 lines and making future changes (new allow entries, prompt wording, write logic) a single-site edit.

### Acceptance criteria

- [ ] `apm init` still adds all `APM_ALLOW_ENTRIES` to `.claude/settings.json` when the user answers `y`
- [ ] `apm init` still adds all `APM_USER_ALLOW_ENTRIES` to `~/.claude/settings.json` when the user answers `y`
- [ ] `apm init` still skips patching `.claude/settings.json` when the file does not exist
- [ ] `apm init` still creates `~/.claude/settings.json` (including parent dir) when the file does not exist and the user answers `y`
- [ ] `apm init` still skips patching when all target entries are already present (no prompt shown)
- [ ] `apm init` still skips patching when the user answers `n` or presses enter at the `[y/N]` prompt
- [ ] `init.rs` contains no duplicate `permissions/allow` manipulation logic — there is exactly one function that reads, diffs, prompts, and writes a settings file
- [ ] The `update_settings_json` helper is `fn update_settings_json(path: &Path, entries: &[&str], prompt_header: &str, prompt_confirm: &str, updated_msg: &str, create_if_missing: bool) -> Result<()>`
- [ ] `cargo build` succeeds with no new warnings

### Out of scope

- Changing which entries are in `APM_ALLOW_ENTRIES` or `APM_USER_ALLOW_ENTRIES`
- Moving `update_settings_json` out of `init.rs` into a shared module (separate refactor ticket)
- Adding tests for the settings-patching logic (no tests exist today; adding them is a separate concern)
- Changing the interactive prompt UX (confirm vs. auto-apply, etc.)
- Any changes to `apm init` behaviour other than the internal deduplication

### Approach

All changes are in `apm/src/cmd/init.rs`.

**1. Add the shared helper above the two existing functions (~line 158)**

```rust
fn update_settings_json(
    path: &Path,
    entries: &[&str],
    prompt_header: &str,
    prompt_confirm: &str,
    updated_msg: &str,
    create_if_missing: bool,
) -> Result<()> {
    // Resolve file content or bail/bootstrap
    let mut val: Value = if path.exists() {
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str(&raw).unwrap_or(Value::Object(Default::default()))
    } else if create_if_missing {
        Value::Object(Default::default())
    } else {
        return Ok(());
    };

    // Diff
    let allow = val.pointer_mut("/permissions/allow").and_then(|v| v.as_array_mut());
    let missing: Vec<&str> = if let Some(arr) = allow {
        entries.iter().filter(|&&e| !arr.iter().any(|v| v.as_str() == Some(e))).copied().collect()
    } else {
        entries.to_vec()
    };

    if missing.is_empty() {
        return Ok(());
    }

    // Prompt
    println!("{prompt_header}");
    for e in &missing {
        println!("  {e}");
    }
    print!("{prompt_confirm} [y/N] ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    if !line.trim().eq_ignore_ascii_case("y") {
        println!("Skipped.");
        return Ok(());
    }

    // Ensure /permissions/allow exists
    if val.pointer("/permissions/allow").is_none() {
        let perms = val
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("settings.json root is not an object"))?
            .entry("permissions")
            .or_insert_with(|| Value::Object(Default::default()));
        perms.as_object_mut().unwrap()
            .entry("allow")
            .or_insert_with(|| Value::Array(vec![]));
    }

    // Append
    let arr = val.pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut())
        .unwrap();
    for e in missing {
        arr.push(Value::String(e.to_string()));
    }

    // Write (create_dir_all is a no-op if the directory already exists)
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let updated = serde_json::to_string_pretty(&val)?;
    std::fs::write(path, updated + "\n")?;
    println!("{updated_msg}");
    Ok(())
}
```

**2. Replace `update_claude_settings` with a thin wrapper**

```rust
fn update_claude_settings(root: &Path, skip: bool) -> Result<()> {
    if skip {
        return Ok(());
    }
    update_settings_json(
        &root.join(".claude/settings.json"),
        APM_ALLOW_ENTRIES,
        "The following entries will be added to .claude/settings.json permissions.allow:",
        "Add apm commands to Claude allow list?",
        "Updated .claude/settings.json",
        false, // require file to exist
    )
}
```

**3. Replace `update_user_claude_settings` with a thin wrapper**

```rust
fn update_user_claude_settings() -> Result<()> {
    let home = match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => h,
        _ => return Ok(()),
    };
    update_settings_json(
        &PathBuf::from(&home).join(".claude/settings.json"),
        APM_USER_ALLOW_ENTRIES,
        "The following entries will be added to ~/.claude/settings.json (user-level,\nrequired so apm subagents in isolated worktrees can run git and apm commands):",
        "Add to ~/.claude/settings.json?",
        "Updated ~/.claude/settings.json",
        true, // create file if missing
    )
}
```

**4. Verify**

Run `cargo build` (no `--release` needed) from the repo root and confirm zero new warnings. The call sites at lines 59–60 of `run()` are unchanged; no other files need editing.

**Gotcha:** The original `update_user_claude_settings` used `unwrap_or(Value::Object(Default::default()))` on a parse failure (silently accepting corrupt JSON). The shared helper matches that behaviour via `.unwrap_or(...)` when the file exists; the `create_if_missing` branch for a non-existent file also starts from an empty object. Keep this identical to avoid behaviour change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:08Z | new | groomed | apm |
| 2026-04-12T09:12Z | groomed | in_design | philippepascal |