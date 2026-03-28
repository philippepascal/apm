+++
id = 35
title = "github-apm-meta"
state = "ammend"
priority = 0
effort = 3
risk = 4
author = "apm"
branch = "ticket/0035-github-apm-meta"
created_at = "2026-03-27T21:14:43.351349Z"
updated_at = "2026-03-28T18:28:18.863957Z"
+++

## Spec

### Ammend
aren't A and B not really safe anyway? if some other engineer works in another clone, if they don't pull often, they will miss changes to the meta branch and have wrong numbers anyway... so A and B are not really strong, work locally, which could be done with a gitignored file.
C is out of the question as it uses main.
D seems just a hacky A/B.
Nothing is satisfying... how about mimicing git's hash?

take option E and rewrite this entire spec around it, with a preamble explaining the issues with othe numbering systems. 
include in spec facilities to help user in command line (can we accept first few char if unique?, can we offer a sublist with short names if first few char is not?). propose other way to help with command line. 
one key insight here is that we will also have a UI, in which we will not neet to type the hash-id, so this will be alleviated. we only need some sugar on manual cli commands

### Problem

Every `apm new` pushes a new commit to `refs/heads/apm/meta`, causing GitHub to show a "apm/meta had recent pushes" banner on the repo home page. The branch also appears in GitHub's branch list, confusing users unfamiliar with APM. The current optimistic-locking protocol (read NEXT_ID, increment, push, retry on rejection) adds complexity. This ticket proposes alternatives for supervisor to choose from.

### Acceptance criteria

- [ ] At least three alternatives are documented with trade-offs (GitHub noise, concurrency safety, offline support, implementation complexity)
- [ ] A recommended approach is identified with rationale
- [ ] The spec is sufficient for a supervisor to make an informed decision without additional research

### Out of scope

- Implementation (this is a proposal/design ticket)
- Changing the ticket file format or branch naming scheme

### Amendment requests

- [x] Options A and B dismissed: any shared counter (ref or branch-scan) is
  broken when clones are stale — engineers who don't pull frequently will
  produce duplicate IDs regardless of locking protocol
- [x] Options C and D dismissed: C commits to main (unacceptable), D is the
  same stale-clone problem in a different container
- [x] Add an option that mimics git's hash: coordinator-free, works offline,
  no shared state required

### Approach

**Option A — Use `refs/apm/meta` instead of `refs/heads/apm/meta`** _(dismissed)_
Eliminates GitHub noise but the stale-clone problem remains: a clone that
hasn't fetched recently reads an outdated counter and generates a duplicate ID.
The optimistic-locking retry only helps for concurrent pushes, not for offline
or stale-clone creation.

**Option B — Derive ID from existing branch names** _(dismissed)_
Same root failure: scanning local `refs/heads/ticket/NNNN-*` without fetching
returns the stale max. Two engineers on different clones both get the same `max
+ 1`. Locking is impossible without coordination.

**Option C — Commit counter to `main`** _(dismissed)_
Pollutes `main` with non-code commits and requires `main` to be pushable.

**Option D — Monotonic tag** _(dismissed)_
Structurally identical to A: same stale-clone failure mode, requires `--force`
tag pushes which many repos restrict.

---

**Option E — Hash-derived ticket IDs (recommended)**

Mimic git object IDs: derive the ticket ID from a hash of local entropy
(timestamp + random bytes), making it globally unique without any shared
counter or network access.

```
id = sha1(unix_timestamp_ns + 8 random bytes) → take first 8 hex chars
```

Example ID: `a3f9b2c1`. Branch name: `ticket/a3f9b2c1-short-title`.

Trade-offs:

| Property | Current (counter) | Option E (hash) |
|---|---|---|
| GitHub noise | Yes (apm/meta branch) | None |
| Stale-clone safe | No | Yes — no shared state |
| Offline | Fallback only | Always works |
| Collision probability | Zero (counter) | ~1 in 4 billion per 8-char hex |
| Sequential / sortable | Yes | No (timestamp prefix helps) |
| `apm show 35` syntax | Works | Needs prefix search |
| Format change | No | Breaking — all existing IDs change |

Collision risk at 8 hex chars (32 bits): with 1000 tickets the birthday
probability is `~0.01%` — acceptable for project-scale ticket counts.

Partial-match lookup (`apm show a3f9` → finds `a3f9b2c1`) matches git's UX.

**Recommendation: Option E** if the format change is acceptable. It is the only
option that is truly coordination-free and immune to stale clones. The UX shift
from numeric IDs to short hashes is the main cost.

If the format change is not acceptable, there is no good solution — all counter
approaches are broken when clones diverge. Supervisor to decide whether to adopt
hash IDs or accept the limitations of any counter approach.
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:14Z | — | new | apm |
| 2026-03-28T01:03Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T07:45Z | specd | ammend | apm |
| 2026-03-28T18:18Z | ammend | specd | apm |
| 2026-03-28T18:28Z | specd | ammend | apm |
