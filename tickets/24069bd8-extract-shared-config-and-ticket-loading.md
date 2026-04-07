+++
id = "24069bd8"
title = "Extract shared config-and-ticket loading helper in CLI crate"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/24069bd8-extract-shared-config-and-ticket-loading"
created_at = "2026-04-07T22:30:46.572883Z"
updated_at = "2026-04-07T22:53:21.721662Z"
epic = "ac0fb648"
target_branch = "epic/ac0fb648-code-separation-and-reuse-cleanup"
+++

## Spec

### Problem

Approximately 15 command handlers in `apm/src/cmd/` repeat the same boilerplate sequence:

```rust
let config = Config::load(root)?;
let aggressive = config.sync.aggressive && !no_aggressive;
if aggressive {
    if let Err(e) = git::fetch_all(root) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
```

This appears in `list.rs`, `show.rs`, `review.rs`, `validate.rs`, `verify.rs`, `spec.rs`, `set.rs`, `sync.rs`, `new.rs`, `epic.rs`, `work.rs`, `clean.rs`, and others. Each copy is slightly different: some skip the aggressive fetch, some load tickets conditionally, some add extra steps.

The duplication means that any change to the loading sequence (e.g., adding a validation step, changing the fetch behavior) must be applied to every file independently. It also makes it hard to see what each command actually does — the real logic is buried under 5-8 lines of identical setup.

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
| 2026-04-07T22:30Z | — | new | philippepascal |
| 2026-04-07T22:44Z | new | groomed | apm |
| 2026-04-07T22:53Z | groomed | in_design | philippepascal |
