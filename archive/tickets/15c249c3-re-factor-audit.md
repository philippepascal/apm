+++
id = "15c249c3"
title = "re-factor audit"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "apm"
agent = "claude-0330-0245-main"
branch = "ticket/15c249c3-re-factor-audit"
created_at = "2026-03-30T06:01:35.844292Z"
updated_at = "2026-03-30T18:07:57.966113Z"
+++

## Spec

### Problem

The `apm` CLI crate contains business logic and utility helpers that have leaked
out of `apm-core`, making them untestable at the library level and forcing
repetition across command handlers. Six patterns appear in 5–8 files each:

1. **`append_history` / `ensure_amendment_section`** — pure string manipulation
   for the audit trail, currently defined in `apm/src/cmd/state.rs` and
   re-exported to `start.rs` and `take.rs`.
2. **Branch resolution** — `t.frontmatter.branch.clone().or_else(…).unwrap_or_else(…)`
   repeated verbatim in 8+ command handlers.
3. **Ticket-relative path** — `format!("{}/{}", config.tickets.dir, filename)`
   repeated in 7+ handlers.
4. **Worktree provisioning** — `ensure_worktree()` defined independently in
   both `take.rs` and `start.rs` with identical logic.
5. **Load-and-resolve** — the three-liner that loads all tickets then resolves
   an id arg repeated in 5+ handlers.
6. **`rand_u16()`** — a small utility currently embedded in `start.rs`.

The desired state is that each pattern lives in `apm-core` (either
`ticket.rs` or `git.rs`) and every command handler calls the shared helper.
This makes the logic testable in unit tests and keeps the CLI layer thin.

The deliverable for **this ticket** is not the refactoring itself, but a
shell script — `refactor-tickets.sh` — that, when run, calls `apm new
--no-edit` to create one focused ticket per refactoring item. The user
reviews the script, deletes entries they don't want, then runs it.

### Acceptance criteria

- [x] `refactor-tickets.sh` exists at the repo root and is executable
- [x] Running the script with `bash -n refactor-tickets.sh` (syntax check) exits 0
- [x] The script contains exactly one `apm new --no-edit` call per identified refactoring item (minimum 6, one per pattern listed in the Problem section)
- [x] Each `apm new` call has a title that identifies the crate move and the function/pattern being relocated (e.g. `"Move append_history to apm-core"`)
- [x] The script is plain `sh`-compatible (no bash-isms beyond `#!/usr/bin/env bash` shebang) and has no external dependencies beyond `apm`
- [x] A comment above each `apm new` line gives one sentence explaining why the move matters

### Out of scope

- Actually performing any of the refactoring moves (each generated ticket covers one move)
- Adding tests for moved code (the implementation tickets should include that)
- Changing any existing behaviour — this ticket produces only a script
- Auditing for performance issues or architectural concerns unrelated to duplication / crate boundaries

### Approach

The audit has already been performed (see exploration results above). The
implementation is writing the script file.

**File to create:** `refactor-tickets.sh` at the repo root.

**Script structure:**

```sh
#!/usr/bin/env bash
set -euo pipefail

# Move append_history and ensure_amendment_section to apm-core::ticket
# These are pure audit-trail helpers; living in the CLI prevents library-level testing.
apm new --no-edit "Move append_history/ensure_amendment_section to apm-core"

# Extract branch resolution helper to apm-core::ticket::resolve_branch()
# The pattern `frontmatter.branch.or_else(branch_name_from_path).unwrap_or_else(…)` appears in 8+ handlers.
apm new --no-edit "Extract resolve_branch helper to apm-core"

# Extract ticket relative-path helper to apm-core::ticket::rel_path_for_ticket()
# Repeated format! pattern in 7+ handlers; single source of truth eliminates drift.
apm new --no-edit "Extract rel_path_for_ticket helper to apm-core"

# Deduplicate ensure_worktree into apm-core::git::ensure_worktree_for_branch()
# Identical function defined in both take.rs and start.rs.
apm new --no-edit "Deduplicate ensure_worktree into apm-core::git"

# Extract load-and-resolve ticket helper to apm-core::ticket
# Three-liner (load_all_from_git + resolve_id_in_slice + find) repeated in 5+ handlers.
apm new --no-edit "Extract load_and_resolve ticket helper to apm-core"

# Move rand_u16 to apm-core
# Small utility embedded in start.rs; moving it allows other consumers to use it.
apm new --no-edit "Move rand_u16 utility to apm-core"
```

**Steps:**
1. Write `refactor-tickets.sh` with the content above
2. `chmod +x refactor-tickets.sh`
3. Commit to the ticket branch
4. Open PR targeting `main`

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:01Z | — | new | apm |
| 2026-03-30T06:16Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T06:19Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T06:23Z | specd | ready | apm |
| 2026-03-30T06:25Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T06:26Z | in_progress | implemented | claude-0330-0245-main |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |