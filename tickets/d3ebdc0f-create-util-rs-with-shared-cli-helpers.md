+++
id = "d3ebdc0f"
title = "Create util.rs with shared CLI helpers"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d3ebdc0f-create-util-rs-with-shared-cli-helpers"
created_at = "2026-04-12T09:02:33.251574Z"
updated_at = "2026-04-12T09:09:29.821591Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

Several boilerplate patterns are duplicated across 7+ command files in `apm/src/cmd/`:

1. **Aggressive fetch check** (7 files: assign.rs, show.rs, next.rs, close.rs, spec.rs, sync.rs, new.rs):
   ```rust
   let aggressive = config.sync.aggressive && !no_aggressive;
   if aggressive { git::fetch_all(root).unwrap_or_else(|e| eprintln!("warning: fetch failed: {e:#}")); }
   ```

2. **Fetch error warning** (6 files): `eprintln!("warning: fetch failed: {e:#}")` — identical string in every file.

3. **Confirmation prompt** (3+ files: assign.rs, clean.rs):
   ```rust
   print!("..."); io::stdout().flush()?; let mut input = String::new(); io::stdin().read_line(&mut input)?;
   ```

There is no shared utility module. Each command file reimplements these patterns, making them inconsistent and hard to update. Creating `apm/src/util.rs` with `fetch_if_aggressive()`, `log_fetch_warning()`, and `prompt_yes_no()` would eliminate this duplication.

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:08Z | new | groomed | apm |
| 2026-04-12T09:09Z | groomed | in_design | philippepascal |
