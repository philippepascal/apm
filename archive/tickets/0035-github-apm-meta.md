+++
id = "0035"
title = "github-apm-meta"
state = "closed"
priority = 0
effort = 3
risk = 4
author = "apm"
agent = "claude-0330-0245-main"
branch = "ticket/0035-github-apm-meta"
created_at = "2026-03-27T21:14:43.351349Z"
updated_at = "2026-03-30T05:41:45.940173Z"
+++

## Spec

### Problem

APM currently assigns ticket IDs using a shared counter stored in `refs/heads/apm/meta`. This causes two distinct problems:

**1. GitHub noise.** Every `apm new` pushes a commit to `apm/meta`, triggering GitHub's "branch had recent pushes" banner on the repo home page. The branch also appears in the branch list, confusing contributors unfamiliar with APM.

**2. The counter approach is fundamentally broken across clones.** Any scheme that derives the next ID from shared state (a meta branch, a tag, a scan of existing branch names) fails the moment a clone is stale. An engineer who hasn't fetched recently reads an outdated max and generates a duplicate ID. Optimistic locking only prevents two concurrent pushes from the same up-to-date state; it cannot prevent two engineers on diverged clones from independently arriving at the same number. This rules out:

- Option A (meta ref): stale clone, same problem, just less visible on GitHub
- Option B (scan branch names): stale `refs/heads/ticket/NNNN-*`, same duplicate ID failure
- Option C (commit to main): pollutes main with non-code commits
- Option D (monotonic tag): stale clone, same failure, plus force-push restrictions

The only sound solution is one that requires no shared state at all.

### Acceptance criteria

- [x] Ticket IDs are 8-character hex strings derived from local entropy (timestamp + random bytes); no network access or shared state is required to generate one
- [x] The branch name format changes to `ticket/<hex8>-<slug>` (e.g. `ticket/a3f9b2c1-short-title`)
- [x] The `id` field in ticket frontmatter stores the 8-char hex string
- [x] All `apm` commands that take an ID also accept a unique prefix of at least 4 characters (`apm show a3f9` resolves to `a3f9b2c1` if unambiguous)
- [x] If a prefix matches more than one ticket, `apm` prints a disambiguation list and exits non-zero
- [x] `apm list` output shows the full 8-char ID; sorting is by creation timestamp (embedded in the ID generation, not the ID itself)
- [x] The `apm/meta` branch is no longer created or pushed on `apm new`
- [x] Collision probability with 1 000 tickets is documented and acceptable (≤ 0.01% birthday probability at 32 bits)
- [x] A one-time migration script (`scripts/migrate-ticket-ids.sh`) updates the `id` field in frontmatter for all existing numeric-ID tickets from integer form (`id = 35`) to zero-padded string form (`id = "0035"`); no branch or file renaming is performed
- [x] After the migration script runs, `apm show 0035` and `apm show 35` both resolve correctly (the integer shorthand continues to work as a convenience)

### Out of scope

- Tab-completion scripts
- Any UI work (the UI will display full IDs; hash ergonomics are a CLI-only concern)

### Amendment requests

- [x] Options A and B dismissed: any shared counter (ref or branch-scan) is
  broken when clones are stale — engineers who don't pull frequently will
  produce duplicate IDs regardless of locking protocol
- [x] Options C and D dismissed: C commits to main (unacceptable), D is the
  same stale-clone problem in a different container
- [x] Add an option that mimics git's hash: coordinator-free, works offline,
  no shared state required
- [x] Spec rewritten entirely around hash-derived IDs (Option E); preamble
  explains why all counter-based approaches are broken
- [x] CLI ergonomics (prefix match, disambiguation list) specified in acceptance
  criteria and approach; UI context noted
- [x] Migration plan added: `apm migrate` command rewrites existing numeric-ID
  tickets to hex IDs in-place; numeric ID aliases supported during transition
- [x] Migration changed from `apm migrate` command to a one-time repo script
  (`scripts/migrate-ticket-ids.sh`); numeric ID aliases kept for transition period
- [x] Migration script simplified: existing branches are already zero-padded (`ticket/0035-*`) and
  are valid hex strings — no branch or file renaming needed; script only updates `id` frontmatter field
- [x] Amendment items now formatted as checkboxes per convention; ticket #51 covers
  auto-normalisation of plain bullets in `### Amendment requests` at review time

### Approach

#### ID generation

```
raw   = sha256(unix_timestamp_nanos as u64 LE || 8 random bytes)
id    = hex(raw)[..8]   // first 8 hex chars = 32 bits of entropy
```

Using `sha256` (already a common dependency) over the concatenation of a nanosecond timestamp and 8 cryptographically random bytes. Birthday collision probability with N tickets: `N² / 2³²`. At N = 1 000 that is ~0.023% — acceptable. At N = 10 000 it reaches ~2.3%; if the project ever grows that large, the prefix length can be extended without breaking the format (old 8-char IDs remain valid, new IDs are longer).

The `apm/meta` branch is removed from `apm new`; the ID generation function replaces the counter read.

#### Branch and file naming

```
ticket/<hex8>-<slug>           branch
tickets/<hex8>-<slug>.md       file
```

The slug is derived the same way as today (title → lowercase, spaces → hyphens, truncated). The 4-digit zero-padded prefix in the current scheme (`0042-`) is replaced with the 8-char hex prefix.

#### CLI prefix resolution

Every command that accepts an ID (`show`, `state`, `set`, `start`, `take`, `review`, `worktrees --add/--remove`) runs the ID through a resolver before acting:

1. If the argument is exactly 8 hex chars and matches a known ticket exactly → use it.
2. If the argument is a prefix (4–7 hex chars) → scan loaded tickets for all IDs starting with that prefix.
   - Exactly one match → use it, silently.
   - Zero matches → error: "no ticket matches prefix `<arg>`".
   - Two or more matches → print disambiguation list and exit non-zero:
     ```
     error: prefix 'a3f9' is ambiguous
       a3f9b2c1  fix login timeout
       a3f9dd04  add retry logic
     ```
3. Reject anything that is not 4–8 hex chars.

#### `apm list` sort order

Sort by `created_at` timestamp from the frontmatter (already present), not by ID. This preserves chronological ordering even though IDs are not sequential.

#### Migration: `scripts/migrate-ticket-ids.sh`

A one-time shell script committed to this repo (not an `apm` subcommand). Existing ticket branches are already named `ticket/0035-*` with zero-padded 4-digit prefixes — these are valid hex strings and do not need renaming. Only the frontmatter `id` field needs updating from integer to string.

The script, for each ticket branch matching `ticket/[0-9][0-9][0-9][0-9]-*`:
1. Checks out the branch into a temp worktree (or reads via `git show`)
2. Updates `id = N` → `id = "NNNN"` (zero-padded 4-char string) in the frontmatter
3. Commits the change on the branch with message `"migrate: id N → NNNN"`

Prints a summary of all updated tickets on completion. No branch renaming, no file renaming.

**Integer shorthand**: `apm` commands accept a plain integer (e.g. `35`) and zero-pad it to 4 chars for lookup — this matches existing branches and remains valid after migration. No separate alias mechanism needed.

#### CLI ergonomics note

Engineers will primarily interact with tickets through the UI, where IDs are clickable and never typed. On the CLI, prefix resolution and the disambiguation list cover the common manual-command case. If further sugar is needed in the future (e.g. fuzzy title match: `apm show "login timeout"`), that is a separate ticket.

**Audited 2026-03-29:** Approach still valid. Codebase still uses `u32` IDs with NEXT_ID file counter (`apm-core/src/ticket.rs`). No hex ID generation or prefix resolution implemented yet. All referenced types and file paths are accurate.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:14Z | — | new | apm |
| 2026-03-28T01:03Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T07:45Z | specd | ammend | apm |
| 2026-03-28T18:18Z | ammend | specd | apm |
| 2026-03-28T18:28Z | specd | ammend | apm |
| 2026-03-28T18:32Z | ammend | specd | claude-0328-c72b |
| 2026-03-28T19:40Z | specd | ammend | apm |
| 2026-03-28T22:04Z | ammend | specd | claude-0328-1430-a4f2 |
| 2026-03-28T22:09Z | specd | ammend | apm |
| 2026-03-28T22:34Z | ammend | in_design | claude-0328-1430-a4f2 |
| 2026-03-28T22:35Z | in_design | specd | claude-0328-1430-a4f2 |
| 2026-03-29T19:08Z | specd | ready | claude-0329-1200-a1b2 |
| 2026-03-29T23:13Z | ready | ammend | apm |
| 2026-03-29T23:14Z | ammend | in_design | claude-0329-1430-main |
| 2026-03-29T23:16Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:18Z | specd | ready | apm |
| 2026-03-30T04:52Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T05:28Z | in_progress | implemented | claude-0329-resume |
| 2026-03-30T05:39Z | implemented | accepted | apm |
| 2026-03-30T05:41Z | accepted | closed | apm-sync |