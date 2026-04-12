+++
id = "4660b156"
title = "Split ticket.rs into ticket_fmt.rs and ticket_util.rs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4660b156-split-ticket-rs-into-ticket-fmt-rs-and-t"
created_at = "2026-04-12T06:04:17.196705Z"
updated_at = "2026-04-12T06:17:18.865761Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`ticket.rs` is a 1965-line file in `apm-core/src/` that conflates two unrelated concerns:\n\n1. **File format**: TOML frontmatter parsing and serialization, markdown body parsing (`TicketDocument`), checklist parsing, ID normalization (`normalize_id_arg`, `slugify`, etc.), and body validation.\n2. **Ticket logic**: scoring (`score`, `effective_priority`), dependency graph construction (`build_reverse_index`), ticket selection (`pick_next`, `sorted_actionable`), lifecycle operations (`create`, `close`), and git-native loading (`load_all_from_git`).\n\nHaving both concerns in one file makes it hard to find the right function quickly, and it creates unnecessary coupling — a caller that only needs ID normalization still compiles the full dependency-graph logic. The fix is a mechanical split into two new files with clear responsibilities, plus a thin `ticket.rs` re-export hub that keeps every downstream `use apm_core::ticket::…` path working unchanged.

### Acceptance criteria

- [ ] `apm-core/src/ticket_fmt.rs` exists and contains: `Frontmatter`, `Ticket` (struct + `load`/`parse`/`serialize`/`save`/`score`/`document`), `TicketDocument` (struct + `parse`/`serialize`/`validate`/`unchecked_tasks`/`toggle_criterion`), `ChecklistItem`, `ValidationError`, `slugify`, `normalize_id_arg`, `id_arg_prefixes`, `resolve_id_in_slice`, and `set_field`\n- [ ] `apm-core/src/ticket_util.rs` exists and contains: `build_reverse_index`, `effective_priority`, `dep_satisfied`, `sorted_actionable`, `pick_next`, `load_all_from_git`, `state_from_branch`, `list_worktrees_with_tickets`, `close`, `create`, `check_owner`, and `list_filtered`\n- [ ] `apm-core/src/ticket.rs` contains only `pub use` re-exports from `ticket_fmt` and `ticket_util`; no type definitions or function bodies remain in it\n- [ ] `cargo build --workspace` succeeds with no errors after the split\n- [ ] `cargo test --workspace` passes with the same number of passing tests as before the split\n- [ ] No file outside `apm-core/src/ticket*.rs` requires any `use` path changes — all existing `use apm_core::ticket::*` imports continue to resolve

### Out of scope

- Changing any public function or type signatures\n- Moving or rewriting logic — this is a mechanical file split only\n- Splitting tests into a separate `tests/` directory; unit tests move with their functions into `ticket_fmt.rs` or `ticket_util.rs`\n- Making `ticket_fmt` or `ticket_util` public modules in `lib.rs`; they are internal to the `ticket` re-export layer\n- Any changes to `apm`, `apm-server`, or other `apm-core` modules\n- Adding new functionality or fixing existing bugs

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T06:04Z | — | new | philippepascal |
| 2026-04-12T06:12Z | new | groomed | apm |
| 2026-04-12T06:17Z | groomed | in_design | philippepascal |