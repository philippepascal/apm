+++
id = "14214305"
title = "apm help ticket: render ticket.toml schema from TicketConfig struct"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/14214305-apm-help-ticket-render-ticket-toml-schem"
created_at = "2026-04-28T19:28:31.483927Z"
updated_at = "2026-04-29T07:50:45.379206Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_ticket()` function in `apm/src/cmd/help.rs` — introduced as a stub by ticket bc89e0a0 — returns a placeholder string and does nothing useful. As a result, `apm help ticket` gives users no actionable information about what fields are valid in `.apm/ticket.toml`, their types, defaults, or purpose.

The relevant types (`TicketConfig`, `TicketSection`, `SectionType`) already exist in `apm-core/src/config.rs`. Ticket 069c3403 adds `JsonSchema` derives to those types and supplies `apm_core::help_schema::render_schema::<T>()`, which walks the schema and emits a formatted table of fields with their types, defaults, and doc-comment descriptions. All that is missing is: (1) meaningful doc comments on `TicketConfig`, `TicketSection`, and `SectionType` so `render_schema` has descriptions to emit, and (2) a real body for `render_ticket()` that calls `render_schema::<TicketConfig>()` and prepends a short introductory header.

The `SectionType` enum warrants special attention: its three variants (`free`, `tasks`, `qa`) each have distinct runtime semantics — `tasks` sections integrate with `apm spec --mark` and `apm spec --add-task` — and those semantics should be visible from `apm help ticket` without reading source code.

### Acceptance criteria

- [ ] `apm help ticket` exits 0 and prints to stdout
- [ ] The output contains the string `ticket.sections[]` (array notation indicating `[[ticket.sections]]` is a TOML array-of-tables)
- [ ] The output contains a line for the `ticket.sections[].name` field with type `string`
- [ ] The output contains a line for the `ticket.sections[].type` field with type `string` and enum variants listing `free`, `tasks`, and `qa`
- [ ] The output contains a line for the `ticket.sections[].required` field with type `bool` and default `false`
- [ ] The output contains a line for the `ticket.sections[].placeholder` field
- [ ] The description shown for the `type` field (or the introductory header) mentions `apm spec --mark` and `apm spec --add-task` in the context of the `tasks` variant
- [ ] The output does not contain the placeholder stub text from ticket bc89e0a0
- [ ] `apm help ticket` output is identical to calling `render_ticket()` directly (no extra whitespace trimmed or added by the dispatcher)

### Out of scope

- Frontmatter schema (`Frontmatter` struct in `ticket_fmt.rs`) — that is the ticket *file* format, not `.apm/ticket.toml`; could be a follow-up help topic
- ANSI colour or markdown rendering in the output
- Pager integration (`less`/`more`)
- The `apm help` dispatcher and topic routing — established by ticket bc89e0a0
- The `help_schema` infrastructure (`schema_entries`, `render_schema`, `FieldEntry`) — that is ticket 069c3403
- Adding `JsonSchema` derives to `TicketConfig`, `TicketSection`, `SectionType` — that is ticket 069c3403
- Content for `render_commands()`, `render_config()`, `render_workflow()` — sibling tickets 3665e017, d486d183, 7ba021e8
- Changes to how `apm <subcommand> --help` works (clap-native help is untouched)
- Per-section examples or a tutorial on designing a ticket workflow

### Approach

This ticket has two distinct changes: adding doc comments in `apm-core`, and replacing the stub in `apm`.

---

**1. `apm-core/src/config.rs` — add doc comments**

Ticket 069c3403 already adds `JsonSchema` derives to `TicketConfig`, `TicketSection`, and `SectionType`. This ticket's job is to ensure the doc comments are present and informative enough for `render_schema` to produce useful output.

Add or replace doc comments on:

- `TicketConfig`:
  ```rust
  /// Configuration for the sections that appear on every ticket, in order.
  /// Defined in `.apm/ticket.toml` as `[[ticket.sections]]` blocks.
  ```

- `TicketSection`:
  ```rust
  /// A single section in the ticket template.
  ```
  Per field:
  - `name`: `/// Display name of the section (e.g. "Problem", "Approach").`
  - `type_` (serde: `type`): `/// Rendering mode — controls how the section body is interpreted and which apm spec flags apply.`
  - `required`: `/// Whether the section must be non-empty before the ticket can transition out of in_design.`
  - `placeholder`: `/// Hint text pre-filled into an empty section when a new ticket is created.`

- `SectionType` — the doc comment on the enum itself must carry variant semantics because `render_schema` emits a single `description` field for the `type` entry, not per-variant descriptions:
  ```rust
  /// `free` — free-form prose. `tasks` — checkbox list (`- [ ] item`);
  /// supports `apm spec --mark` and `apm spec --add-task`. `qa` — question/answer pairs.
  ```

---

**2. `apm/src/cmd/help.rs` — replace the stub**

Replace the body of `fn render_ticket() -> String` (currently returning a placeholder referencing ticket 14214305) with:

```rust
fn render_ticket() -> String {
    let header = "ticket.toml — ticket section configuration\n\
                  \n\
                  Defines the [[ticket.sections]] array: an ordered list of sections\n\
                  that appear on every ticket created in this project.\n\n";
    format!("{}{}", header, apm_core::help_schema::render_schema::<apm_core::config::TicketConfig>())
}
```

No other changes to `help.rs` are needed.

---

**Implementation order:**

1. Ensure 069c3403 is merged (or apply its changes locally in the worktree) — `TicketConfig` must have `JsonSchema` derived and `apm_core::help_schema` must exist before the `apm` crate can compile.
2. Add doc comments to `TicketConfig`, `TicketSection`, `TicketSection` fields, and `SectionType` in `apm-core/src/config.rs`.
3. Replace `render_ticket()` stub in `apm/src/cmd/help.rs`.
4. `cargo build` to confirm the crate compiles and the call resolves.
5. Smoke-test: run `apm help ticket` and verify all nine acceptance criteria pass.

---

**Constraints:**

- Do not add `use` imports beyond what the two dependencies have already introduced; call `apm_core::help_schema::render_schema` by its full path or add one targeted `use` line.
- The introductory header must end with a blank line so it is visually separated from the field table that `render_schema` emits.
- `SectionType` doc comment must be a single contiguous string (no `///` line breaks mid-sentence) so schemars concatenates it into a single `description` value without spurious whitespace.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:28Z | — | new | philippepascal |
| 2026-04-28T19:33Z | new | groomed | philippepascal |
| 2026-04-28T19:57Z | groomed | in_design | philippepascal |
| 2026-04-28T20:01Z | in_design | specd | claude-0428-1957-cfa0 |
| 2026-04-29T03:43Z | specd | ready | philippepascal |
| 2026-04-29T07:50Z | ready | in_progress | philippepascal |
