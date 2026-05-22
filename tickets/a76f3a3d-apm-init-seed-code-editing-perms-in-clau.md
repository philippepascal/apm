+++
id = "a76f3a3d"
title = "apm init: seed code-editing perms in .claude/settings.json"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a76f3a3d-apm-init-seed-code-editing-perms-in-clau"
created_at = "2026-05-17T19:52:57.296867Z"
updated_at = "2026-05-22T02:40:37.389182Z"
depends_on = ["db166d95"]
+++

## Spec

### Problem

`apm init` seeds `APM_ALLOW_ENTRIES` into `.claude/settings.json` so workers can invoke `apm` subcommands without permission prompts. That list (`apm/src/cmd/init.rs:138-167`) covers only `Bash(apm …)` patterns. Sibling ticket `db166d95` fixes the create-vs-update gap so the file is written when `.claude/` is present but `settings.json` is not; this ticket covers the *content* of that file.

The gap is concrete: ticket `996fef40` triggered a worker that successfully called `apm spec` and `apm state` (both allowed), then stalled the moment it tried to open a file with the `Edit` tool — not on the list. The worker's graceful-exit path worked, but every implementation ticket hits the same wall. Workers also need read helpers (`grep`, `find`, `cat`, etc.), text manipulation (`sed`, `awk`), safe file ops (`mv`, `cp`, `/tmp` writes), and git ops inside their worktree (`git -C …`). Language-specific toolchain commands (`cargo`, `npm`, `python3`) complete the picture for build and test steps.

The desired end state: after `db166d95` + this ticket, `apm init` in a project with `.claude/` present writes a `settings.json` that lets a worker edit source files, run the project's test suite, and complete a typical implementation ticket without hitting a permission prompt.

### Acceptance criteria

- [ ] `APM_ALLOW_ENTRIES` includes `Edit` and `Write` tool entries
- [ ] `APM_ALLOW_ENTRIES` includes the language-agnostic bash baseline: `Bash(git -C *)`, `Bash(ls *)`, `Bash(rg *)`, `Bash(grep *)`, `Bash(find *)`, `Bash(cat *)`, `Bash(head *)`, `Bash(tail *)`, `Bash(wc *)`, `Bash(sort *)`, `Bash(uniq *)`, `Bash(diff *)`, `Bash(which *)`, `Bash(sed *)`, `Bash(awk *)`, `Bash(mv *)`, `Bash(cp *)`, `Bash(rm /tmp/*)`, `Bash(mkdir -p /tmp/*)`, `Bash(echo *)`, `Bash(test *)`, `Bash(true)`, `Bash(false)`
- [ ] When `Cargo.toml` exists at the repo root, `apm init` adds `Bash(cargo *)` to the project `.claude/settings.json` entries
- [ ] When `package.json` exists at the repo root, `apm init` adds `Bash(npm *)` and `Bash(npx *)` to the project `.claude/settings.json` entries
- [ ] When `pyproject.toml` or `requirements.txt` exists at the repo root, `apm init` adds `Bash(python3 *)` to the project `.claude/settings.json` entries
- [ ] `APM_USER_ALLOW_ENTRIES` includes `Edit`, `Write`, the same language-agnostic bash baseline, and unconditionally includes all common toolchain entries (`Bash(cargo *)`, `Bash(npm *)`, `Bash(npx *)`, `Bash(python3 *)`)
- [ ] Running `apm init` (with `db166d95` applied) in a Rust project with `.claude/` present but no `settings.json` produces a file containing `Edit`, `Write`, `Bash(git -C *)`, read helpers, and `Bash(cargo *)`

### Out of scope

- Creating `.claude/settings.json` when `.claude/` is absent — covered by `db166d95`
- The `--yes` flag and non-TTY auto-yes behaviour — covered by `db166d95`
- Toolchain entries for uncommon build systems (make, bazel, go, ruby, etc.) — supervisor adds manually
- Adding `permissions.ask` entries (destructive git ops, force-push guards, etc.)
- Removing or migrating entries that were previously seeded by hand
- Interactive selection of which toolchain entries to include

### Approach

All changes are in `apm/src/cmd/init.rs`.

#### Design decision: language toolchains

Project-level `.claude/settings.json` gets the language-agnostic baseline plus toolchain entries detected from manifest files present at the repo root. User-level `~/.claude/settings.json` gets the same baseline plus all common toolchains unconditionally — user settings serve workers across every repo on the machine, so detection makes no sense there.

#### 1. Expand `APM_ALLOW_ENTRIES`

Append to the existing constant after the last `apm` entry:

```rust
// code editing
"Edit",
"Write",
// git ops in worktree
"Bash(git -C *)",
// read helpers
"Bash(ls *)",
"Bash(rg *)",
"Bash(grep *)",
"Bash(find *)",
"Bash(cat *)",
"Bash(head *)",
"Bash(tail *)",
"Bash(wc *)",
"Bash(sort *)",
"Bash(uniq *)",
"Bash(diff *)",
"Bash(which *)",
// text manipulation
"Bash(sed *)",
"Bash(awk *)",
// file ops (safe areas)
"Bash(mv *)",
"Bash(cp *)",
"Bash(rm /tmp/*)",
"Bash(mkdir -p /tmp/*)",
// shell building blocks
"Bash(echo *)",
"Bash(test *)",
"Bash(true)",
"Bash(false)",
```

#### 2. Add toolchain detection function

```rust
fn detected_toolchain_entries(root: &Path) -> Vec<&'static str> {
    let mut entries = Vec::new();
    if root.join("Cargo.toml").exists() {
        entries.push("Bash(cargo *)");
    }
    if root.join("package.json").exists() {
        entries.extend_from_slice(&["Bash(npm *)", "Bash(npx *)"]);
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        entries.push("Bash(python3 *)");
    }
    entries
}
```

#### 3. Wire detection into `update_claude_settings`

After `db166d95` adds `yes: bool` and the `.claude/` existence guard, merge detected entries before the call:

```rust
fn update_claude_settings(root: &Path, skip: bool, yes: bool) -> Result<()> {
    if skip { return Ok(()); }
    let claude_dir = root.join(".claude");
    if !claude_dir.exists() { return Ok(()); }

    let mut entries: Vec<&str> = APM_ALLOW_ENTRIES.to_vec();
    entries.extend(detected_toolchain_entries(root));

    update_settings_json(
        &claude_dir.join("settings.json"),
        &entries,
        "The following entries will be added to .claude/settings.json permissions.allow:",
        "Add apm commands to Claude allow list?",
        "Updated .claude/settings.json",
        true,
        yes,
    )
}
```

`update_settings_json` de-duplicates against existing entries, so running `apm init` again is idempotent.

#### 4. Expand `APM_USER_ALLOW_ENTRIES`

Add the same language-agnostic baseline plus all common toolchains unconditionally. `APM_USER_ALLOW_ENTRIES` already contains `Bash(git add*)`, `Bash(git commit*)`, `Bash(git -C*)` — add the remaining new entries in the same groups used above for `APM_ALLOW_ENTRIES`, plus:

```rust
"Bash(cargo *)",
"Bash(npm *)",
"Bash(npx *)",
"Bash(python3 *)",
```

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-17T19:52Z | — | new | philippe|philippepascal |
| 2026-05-21T22:59Z | new | groomed | philippepascal |
| 2026-05-21T23:10Z | groomed | in_design | philippepascal |
| 2026-05-21T23:14Z | in_design | specd | claude-0521-2310-3a68 |
| 2026-05-22T02:25Z | specd | ready | philippepascal |
| 2026-05-22T02:40Z | ready | in_progress | philippepascal |
