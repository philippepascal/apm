+++
id = 35
title = "github-apm-meta"
state = "specd"
priority = 0
effort = 3
risk = 4
author = "apm"
branch = "ticket/0035-github-apm-meta"
created_at = "2026-03-27T21:14:43.351349Z"
updated_at = "2026-03-28T18:32:56.877675Z"
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

- [ ] Ticket IDs are 8-character hex strings derived from local entropy (timestamp + random bytes); no network access or shared state is required to generate one
- [ ] The branch name format changes to `ticket/<hex8>-<slug>` (e.g. `ticket/a3f9b2c1-short-title`)
- [ ] The `id` field in ticket frontmatter stores the 8-char hex string
- [ ] All `apm` commands that take an ID also accept a unique prefix of at least 4 characters (`apm show a3f9` resolves to `a3f9b2c1` if unambiguous)
- [ ] If a prefix matches more than one ticket, `apm` prints a disambiguation list and exits non-zero
- [ ] `apm list` output shows the full 8-char ID; sorting is by creation timestamp (embedded in the ID generation, not the ID itself)
- [ ] The `apm/meta` branch is no longer created or pushed on `apm new`
- [ ] Collision probability with 1 000 tickets is documented and acceptable (≤ 0.01% birthday probability at 32 bits)

### Out of scope

- Migrating existing numeric-ID tickets to hex IDs (handled separately if needed)
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

#### CLI ergonomics note

Engineers will primarily interact with tickets through the UI, where IDs are clickable and never typed. On the CLI, prefix resolution and the disambiguation list cover the common manual-command case. If further sugar is needed in the future (e.g. fuzzy title match: `apm show "login timeout"`), that is a separate ticket.
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:14Z | — | new | apm |
| 2026-03-28T01:03Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T07:45Z | specd | ammend | apm |
| 2026-03-28T18:18Z | ammend | specd | apm |
| 2026-03-28T18:28Z | specd | ammend | apm |
| 2026-03-28T18:32Z | ammend | specd | claude-0328-c72b |
