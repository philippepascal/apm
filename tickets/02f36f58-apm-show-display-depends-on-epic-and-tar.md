+++
id = "02f36f58"
title = "apm show: display depends_on, epic, and target_branch frontmatter fields"
state = "in_progress"
priority = 0
effort = 1
risk = 1
author = "apm"
branch = "ticket/02f36f58-apm-show-display-depends-on-epic-and-tar"
created_at = "2026-04-04T00:27:04.270671Z"
updated_at = "2026-04-04T07:17:25.765508Z"
+++

## Spec

### Problem

When running `apm show <id>`, the output header displays `state`, `priority`, `effort`, `risk`, and `branch` — but three optional frontmatter fields are silently omitted: `epic`, `target_branch`, and `depends_on`.

These fields are fully parsed and stored in the `Frontmatter` struct (see `apm-core/src/ticket.rs`), and they carry meaningful context: which epic a ticket belongs to, which branch it targets, and which other tickets must complete before it can start. Without them in `apm show`, an agent or developer reading a ticket must look at the raw file to discover dependencies or epic membership — defeating the purpose of the command.

### Acceptance criteria

- [x] `apm show <id>` prints an `epic:` line when the ticket's `epic` frontmatter field is set
- [x] `apm show <id>` prints a `target_branch:` line when the ticket's `target_branch` frontmatter field is set
- [x] `apm show <id>` prints a `depends_on:` line when the ticket's `depends_on` frontmatter field is set and non-empty
- [x] `apm show <id>` omits the `epic:` line entirely when the field is absent
- [x] `apm show <id>` omits the `target_branch:` line entirely when the field is absent
- [ ] `apm show <id>` omits the `depends_on:` line entirely when the field is absent or empty
- [ ] The three new lines appear in the header block (before the blank line that separates frontmatter from the body), after the existing `branch:` line

### Out of scope

- Displaying these fields in `apm list` output
- Adding a `--json` flag or machine-readable output to `apm show`
- Validating that `depends_on` IDs refer to existing tickets
- Any changes to how these fields are parsed, serialised, or stored in `apm-core`

### Approach

One file changes: `apm/src/cmd/show.rs`.

In the non-edit branch of `run()` (lines 27–35), after the existing `if let Some(b) = &fm.branch` line, add three analogous conditional prints:

```rust
if let Some(e) = &fm.epic         { println!("epic:         {e}"); }
if let Some(tb) = &fm.target_branch { println!("target_branch: {tb}"); }
if let Some(deps) = &fm.depends_on {
    if !deps.is_empty() {
        println!("depends_on:   {}", deps.join(", "));
    }
}
```

No changes to `apm-core` are needed; the fields are already parsed.

Add a unit/integration test in `apm/tests/integration.rs` that:
1. Creates a ticket with `epic`, `target_branch`, and `depends_on` set in frontmatter
2. Runs `apm show <id>` and asserts the three lines appear in stdout
3. Creates a second ticket with none of these fields and asserts the lines are absent

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-04T00:27Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:36Z | groomed | in_design | philippepascal |
| 2026-04-04T06:38Z | in_design | specd | claude-0403-spec-02f3 |
| 2026-04-04T07:15Z | specd | ready | apm |
| 2026-04-04T07:17Z | ready | in_progress | philippepascal |