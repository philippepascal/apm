#!/usr/bin/env bash
set -euo pipefail

# Move append_history and ensure_amendment_section to apm-core::ticket
# These are pure audit-trail helpers; living in the CLI prevents library-level testing.
apm new --no-edit "Move append_history/ensure_amendment_section to apm-core"

# Extract branch resolution helper to apm-core::ticket::resolve_branch()
# The pattern frontmatter.branch.or_else(branch_name_from_path).unwrap_or_else appears in 8+ handlers.
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
