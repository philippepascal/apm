+++
id = 35
title = "github-apm-meta"
state = "new"
priority = 0
effort = 3
risk = 4
author = "apm"
branch = "ticket/0035-github-apm-meta"
created_at = "2026-03-27T21:14:43.351349Z"
updated_at = "2026-03-28T01:03:15.585381Z"
+++

## Spec

### Problem

Every `apm new` pushes a new commit to `refs/heads/apm/meta`, causing GitHub to show a "apm/meta had recent pushes" banner on the repo home page. The branch also appears in GitHub's branch list, confusing users unfamiliar with APM. The current optimistic-locking protocol (read NEXT_ID, increment, push, retry on rejection) adds complexity. This ticket proposes alternatives for supervisor to choose from.

### Acceptance criteria

- [ ] At least three alternatives are documented with trade-offs (GitHub noise, concurrency safety, offline support, implementation complexity)
- [ ] A recommended approach is identified with rationale
- [ ] The spec is sufficient for a supervisor to make an informed decision without additional research

### Out of scope

- Implementation (this is a proposal/design ticket)
- Changing the ticket file format or branch naming scheme

### Approach

**Option A — Use `refs/apm/meta` instead of `refs/heads/apm/meta`**
Move the counter to a non-heads ref. GitHub only shows "recent push" banners and lists branches for refs under `refs/heads/`. The optimistic locking logic stays identical; only the ref name changes. Risk: some hosting providers restrict pushes to non-standard refs. Lowest-effort change.

**Option B — Derive ID from existing branch names**
Scan `refs/heads/ticket/NNNN-*` (locally after `git fetch`) and take `max(NNNN) + 1`. No special branch or push needed. Race condition: two agents running simultaneously could both read the same max and generate the same ID. At typical ticket creation rate (seconds apart, not milliseconds) this is unlikely but not impossible. Offline: works with local refs only. Simplest long-term.

**Option C — Local counter file only**
Use `tickets/NEXT_ID` (already the offline fallback) as the sole counter, tracked in git on `main`. Every `apm new` commits to `main` then pushes. Pros: no special branch. Cons: requires `main` to be pushable (not always true), adds commits to `main` for every ticket, still creates GitHub noise (on main itself).

**Option D — Monotonic tag**
Use lightweight git tags (`refs/tags/apm/next-id`) as the counter. Tags don't show in GitHub's branch list and don't trigger the banner. Optimistic locking works: compare-and-swap via tag replacement. Push with `--force` on the tag ref is required, which some repos restrict.

**Recommendation: Option A** is the minimal-risk change — identical logic, just rename `refs/heads/apm/meta` → `refs/apm/meta`. If the hosting environment supports non-standard ref pushes (GitHub does), this eliminates all GitHub noise immediately. Option B is cleaner long-term but introduces a theoretical race condition. Supervisor to decide.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:14Z | — | new | apm |