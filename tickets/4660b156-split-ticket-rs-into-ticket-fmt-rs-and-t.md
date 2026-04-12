+++
id = "4660b156"
title = "Split ticket.rs into ticket_fmt.rs and ticket_util.rs"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4660b156-split-ticket-rs-into-ticket-fmt-rs-and-t"
created_at = "2026-04-12T06:04:17.196705Z"
updated_at = "2026-04-12T07:00:17.749591Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`ticket.rs` is a 1965-line file in `apm-core/src/` that conflates two unrelated concerns:

1. **File format**: TOML frontmatter parsing and serialization, markdown body parsing (`TicketDocument`), checklist parsing, ID normalization (`normalize_id_arg`, `slugify`, etc.), and body validation.
2. **Ticket logic**: scoring (`score`, `effective_priority`), dependency graph construction (`build_reverse_index`), ticket selection (`pick_next`, `sorted_actionable`), lifecycle operations (`create`, `close`), and git-native loading (`load_all_from_git`).

Having both concerns in one file makes it hard to find the right function quickly, and it creates unnecessary coupling — a caller that only needs ID normalization still compiles the full dependency-graph logic. The fix is a mechanical split into two new files with clear responsibilities, plus a thin `ticket.rs` re-export hub that keeps every downstream `use apm_core::ticket::…` path working unchanged.

### Acceptance criteria

- [ ] `apm-core/src/ticket_fmt.rs` exists and contains: `Frontmatter`, `Ticket` (struct + `load`/`parse`/`serialize`/`save`/`document`), `TicketDocument` (struct + `parse`/`serialize`/`validate`), `ChecklistItem`, `ValidationError`, `slugify`, `normalize_id_arg`, `id_arg_prefixes`, and `resolve_id_in_slice`
- [ ] `apm-core/src/ticket_util.rs` exists and contains: `build_reverse_index`, `effective_priority`, `dep_satisfied`, `sorted_actionable`, `pick_next`, `load_all_from_git`, `state_from_branch`, `list_worktrees_with_tickets`, `close`, `create`, `check_owner`, `list_filtered`, `score`, `set_field`, `unchecked_tasks`, and `toggle_criterion`
- [ ] `apm-core/src/ticket.rs` contains only `pub use` re-exports from `ticket_fmt` and `ticket_util`; no type definitions or function bodies remain in it
- [ ] `cargo build --workspace` succeeds with no errors after the split
- [ ] `cargo test --workspace` passes with the same number of passing tests as before the split
- [ ] No file outside `apm-core/src/ticket*.rs` requires any `use` path changes — all existing `use apm_core::ticket::*` imports continue to resolve

### Out of scope

- Changing any public function or type signatures
- Moving or rewriting logic — this is a mechanical file split only
- Splitting tests into a separate `tests/` directory; unit tests move with their functions into `ticket_fmt.rs` or `ticket_util.rs`
- Making `ticket_fmt` or `ticket_util` public modules in `lib.rs`; they are internal to the `ticket` re-export layer
- Any changes to `apm`, `apm-server`, or other `apm-core` modules
- Adding new functionality or fixing existing bugs

### Approach

**1. Create `apm-core/src/ticket_fmt.rs`**

Move these items verbatim from `ticket.rs`:
- All `use` imports that the moved items depend on
- `Frontmatter` struct and its `Deserialize`/`Serialize` impls (including the custom `deserialize_id` helper)
- `Ticket` struct and its `impl` block (`load`, `parse`, `serialize`, `save`, `document`) — note: `score` moves to `ticket_util.rs`
- `ChecklistItem` struct
- `ValidationError` enum
- `TicketDocument` struct and its `impl` block (`parse`, `serialize`, `validate`) — note: `unchecked_tasks` and `toggle_criterion` move to `ticket_util.rs`
- Free functions: `slugify`, `normalize_id_arg`, `id_arg_prefixes`, `resolve_id_in_slice`
- All `#[cfg(test)]` blocks that test the above

**2. Create `apm-core/src/ticket_util.rs`**

Move these items verbatim from `ticket.rs`:
- All `use` imports that the moved items depend on (will include imports from `ticket_fmt`)
- Free functions: `build_reverse_index`, `effective_priority`, `dep_satisfied`, `sorted_actionable`, `pick_next`, `load_all_from_git`, `state_from_branch`, `list_worktrees_with_tickets`, `close`, `create`, `check_owner`, `list_filtered`
- `score` (currently `Ticket::score` — extracted as a free function or kept as an inherent method via `use super::ticket_fmt::Ticket` — whichever compiles cleanly)
- `set_field` free function
- `TicketDocument::unchecked_tasks` and `TicketDocument::toggle_criterion` — add an `impl TicketDocument` block in `ticket_util.rs` with just these two methods
- All `#[cfg(test)]` blocks that test the above

**3. Replace `apm-core/src/ticket.rs` with a re-export hub**

The new `ticket.rs` contains only:

```rust
mod ticket_fmt;
mod ticket_util;

pub use ticket_fmt::*;
pub use ticket_util::*;
```

No type definitions, `impl` blocks, or function bodies remain here.

**4. `apm-core/src/lib.rs` — no changes required**

`lib.rs` already declares `pub mod ticket;`. The new submodules (`ticket_fmt`, `ticket_util`) are declared inside `ticket.rs` via `mod`, so they remain internal. All callers continue to import via `apm_core::ticket::…` unchanged.

**5. Resolve cross-file `use` dependencies**

`ticket_util.rs` uses types (`Ticket`, `Frontmatter`, `TicketDocument`, etc.) and functions (`slugify`, etc.) that now live in `ticket_fmt.rs`. Add `use super::ticket_fmt::*;` (or explicit named imports) at the top of `ticket_util.rs`.

**6. Verify**

Run `cargo build --workspace` then `cargo test --workspace`. No files outside `apm-core/src/ticket*.rs` should need edits. If the compiler reports missing items in any `use apm_core::ticket::…` path, check that the moved symbol is `pub` and re-exported via `ticket.rs`.

### Open questions


### Amendment requests

- [x] Fix `\n` formatting in Problem and Acceptance criteria sections — literal backslash-n characters appear instead of real newlines. Rewrite with actual newlines.
- [x] Move `Ticket::score()` from `ticket_fmt.rs` to `ticket_util.rs` in the spec — scoring uses prioritization weights and is logic, not format/parsing.
- [x] Move `set_field()` from `ticket_fmt.rs` to `ticket_util.rs` — it's a mutation operation (modifying frontmatter fields), not parsing/serialization.
- [ ] Move `TicketDocument::unchecked_tasks` and `toggle_criterion` from `ticket_fmt.rs` to `ticket_util.rs` — these are checklist manipulation (logic), not format.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:17Z | groomed | in_design | philippepascal |
| 2026-04-12T06:21Z | in_design | specd | claude-0412-0617-eb30 |
| 2026-04-12T06:53Z | specd | ammend | claude-0411-1200-r7c3 |
| 2026-04-12T07:00Z | ammend | in_design | philippepascal |