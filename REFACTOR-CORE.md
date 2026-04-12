# apm-core Refactoring Plan

Reorganize `apm-core/src/` for cleaner module boundaries and dependency composition.

## 1. Move embedded assets to `src/default/`

Move template/config files out of `src/` root:

- `ticket.toml` → `default/ticket.toml`
- `workflow.toml` → `default/workflow.toml`
- `apm.worker.md` → `default/apm.worker.md`
- `apm.spec-writer.md` → `default/apm.spec-writer.md`
- `apm.agents.md` → `default/apm.agents.md`

Update `include_str!()` paths in `init.rs`.

## 2. Rename `git.rs` → `git_util.rs`, keep only git operations

Remove non-git functions:

| Function | Destination |
|---|---|
| `gen_hex_id()` | `ticket_fmt.rs` |
| `resolve_ticket_branch()` | `ticket_fmt.rs` |
| `branch_name_from_path()` | `ticket_fmt.rs` |
| `find_worktree_for_branch()` | `worktree.rs` |
| `list_ticket_worktrees()` | `worktree.rs` |
| `ensure_worktree()` | `worktree.rs` |
| `add_worktree()` | `worktree.rs` |
| `remove_worktree()` | `worktree.rs` |
| `sync_agent_dirs()` | `worktree.rs` |
| `copy_dir_recursive()` | `worktree.rs` |
| `find_epic_branch()` | `epic.rs` |
| `find_epic_branches()` | `epic.rs` |
| `epic_branches()` | `epic.rs` |
| `create_epic_branch()` | `epic.rs` |

What stays: all actual git plumbing — `current_branch`, `fetch_all`, `push_branch`,
`read_from_branch`, `commit_to_branch`, `commit_files_to_branch`,
`merge_branch_into_default`, `is_ancestor`, `branch_tip`, `create_branch_at`, etc.

Also absorbs `merge_into_default()` and `pull_default()` from `state.rs`.

## 3. New module: `worktree.rs`

Worktree lifecycle management, extracted from multiple modules:

**From `git.rs`:**
- `find_worktree_for_branch`, `list_ticket_worktrees`, `ensure_worktree`,
  `add_worktree`, `remove_worktree`, `sync_agent_dirs`, `copy_dir_recursive`

**From `state.rs`:**
- `provision_worktree()`

**From `ticket.rs`:**
- `list_worktrees_with_tickets()`

## 4. Split `ticket.rs` into `ticket_fmt.rs` + `ticket_util.rs`

**`ticket_fmt.rs`** — file format, parsing, serialization:
- `Frontmatter`, `Ticket` (struct + `parse`, `serialize`, `load`, `save`)
- `TicketDocument` (struct + `parse`, `serialize`, `validate`)
- `ChecklistItem`, `parse_checklist()`, `serialize_checklist()`
- `ValidationError`, `deserialize_id()`
- `slugify()`
- `normalize_id_arg()`, `id_arg_prefixes()`, `resolve_id_in_slice()`
- Receives `gen_hex_id()`, `resolve_ticket_branch()`, `branch_name_from_path()` from `git.rs`

**`ticket_util.rs`** — ticket manipulation and querying:
- `load_all_from_git()`, `state_from_branch()`
- `close()`, `create()`
- `list_filtered()`, `check_owner()`, `set_field()`
- `Ticket::score()`, `build_reverse_index()`, `effective_priority()`,
  `sorted_actionable()`, `dep_satisfied()`, `pick_next()`

**`ticket.rs`** — re-export hub:
- `pub mod ticket_fmt;` / `pub mod ticket_util;`
- Re-exports so `apm` and `apm-server` imports don't break

## 5. Trim `state.rs` to pure state machine

Move out:

| Function | Destination |
|---|---|
| `provision_worktree()` | `worktree.rs` |
| `gh_pr_create_or_update()` | `github.rs` |
| `merge_into_default()` | `git_util.rs` |
| `pull_default()` | `git_util.rs` |

What stays: `transition()`, `available_transitions()`, `append_history()`.

## 6. `review.rs` — absorb amendment logic

Move `ensure_amendment_section()` from `state.rs` into `review.rs`.

This makes `review.rs` the home for all spec-document-level operations
(split/extract/normalize/amend), distinct from `spec.rs` which handles
individual section reads/writes.

## 7. Trim `start.rs`

Move `resolve_caller_name()` → `config.rs` (it resolves identity, a config concern).

What stays: `start()`, `spawn_next_worker()`, `spawn_next_worker_direct()`,
`run_worker_in_container()`, `effective_spawn_params()`.

## 8. `epic.rs` — absorb epic branch helpers

Move from `git.rs`:
- `find_epic_branch()`, `find_epic_branches()`, `epic_branches()`, `create_epic_branch()`

These are epic-domain operations, not general git utilities.

## Resulting layout

```
apm-core/src/
  default/
    ticket.toml
    workflow.toml
    apm.worker.md
    apm.spec-writer.md
    apm.agents.md
  lib.rs
  config.rs          (+ resolve_caller_name from start.rs)
  credentials.rs
  epic.rs            (+ epic branch helpers from git.rs)
  git_util.rs        (renamed from git.rs, trimmed + merge/pull from state.rs)
  github.rs          (+ gh_pr_create_or_update from state.rs)
  init.rs            (updated include_str paths)
  logger.rs
  review.rs          (+ ensure_amendment_section from state.rs)
  spec.rs
  start.rs           (trimmed)
  state.rs           (trimmed to pure state machine)
  sync.rs
  clean.rs
  archive.rs
  ticket_fmt.rs      (new: format, parsing, serialization, ID gen)
  ticket_util.rs     (new: manipulation, querying, scoring)
  ticket.rs          (re-exports)
  validate.rs
  verify.rs
  work.rs
  worker.rs
  worktree.rs        (new: worktree lifecycle, agent dir sync)
```

## Dependency order

To minimize conflicts, tickets should be worked in this order:

```
1 (defaults dir)  ───────────────────────────────┐
7 (trim start)    ───────────────────────────────┤ independent
4 (split ticket)  ──────────┐                    │
                            ▼                    │
2 (refactor git)  ──────────┤                    │
          ┌─────────────────┤                    │
          ▼                 ▼                    │
8 (epic absorb)    3 (worktree.rs)               │
          │                 │                    │
          └────────┬────────┘                    │
                   ▼                             │
          5 (trim state)                         │
                   │                             │
                   ▼                             │
          6 (review absorb)                      │
```
