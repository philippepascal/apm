+++
id = "9944425e"
title = "apm list with aggressive fetch should also fast-forward local ticket refs"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9944425e-apm-list-with-aggressive-fetch-should-al"
created_at = "2026-05-21T20:48:39.622072Z"
updated_at = "2026-05-21T23:20:35.429896Z"
+++

## Spec

### Problem

`apm list` reads ticket state from local `ticket/*` refs (via `git_util::ticket_branches` + `git_util::read_from_branch`). With `sync.aggressive = true` (the default in this repo), it runs `git fetch` before reading — which updates `refs/remotes/origin/ticket/*` but does **not** move local `refs/heads/ticket/*`. So the on-disk picture is:

- origin/ticket/<slug> = latest (post-push from another machine)
- ticket/<slug> (local) = stale (last cycle)
- `read_from_branch` prefers local, falls back to origin only when local is absent (`apm-core/src/git_util.rs:31-37`)

Result: `apm list` after a `git fetch` still shows stale state. The only way to reconcile is `apm sync`, which calls `sync_non_checked_out_refs` (`git_util.rs:483`) to fast-forward safe local refs to origin.

Concrete repro hit today: ticket `996fef40` transitioned `in_progress → blocked` on machine A; commit pushed to origin. On machine B, `apm list` continued to show `ready` because machine B's local `ticket/996fef40-…` ref still pointed at the older `ready` commit. Origin had the new `blocked` commit but `read_from_branch`'s local-preference policy hid it. Running `apm sync` on machine B fixed it.

This is a surprising UX. A reasonable user expects `apm list` (with aggressive sync on) to reflect what's on origin without needing a separate command.

Two viable fixes — implementer picks one and records the rationale in Approach:

**Option A — fast-forward inside aggressive sync.** Extend `fetch_if_aggressive` (or its caller chain) so the same `sync_non_checked_out_refs` logic used by `apm sync` runs on every aggressive read path. Cost: every `apm list`/`apm show` does the classify-and-update sweep across all ticket+epic refs (cheap if there are few, O(N) over branch count). Behaviour for `Ahead`/`Diverged` is already conservative (no-op + warning), so this doesn't introduce data-loss risk.

**Option B — flip `read_from_branch` to prefer origin when origin is ahead.** Change the function to consult both refs, classify, and read from whichever is strictly newer. Local-ahead still reads local (so unpushed commits are visible). Diverged falls back to local (or warns and reads local). Cost: extra `merge-base` / `rev-parse` per ticket read.

Acceptance:
- After this change, on a fresh clone (or a machine that has not run `apm sync` recently) `apm list` reflects the state on origin without requiring a separate `apm sync` invocation.
- `apm list --no-aggressive` continues to read whatever the local refs currently say (escape hatch unchanged).
- Unpushed local commits on a ticket branch are still visible in `apm list` / `apm show` (no clobbering of local-ahead state).
- Diverged ticket branches do not silently choose a side — either a warning is surfaced or behaviour is documented in Approach.
- The fix path replicates the `apm sync` ref-classification logic from `git_util::sync_non_checked_out_refs` (or `read_from_branch`'s equivalent), so a single classification source-of-truth governs both flows.

Out of scope:
- Auto-pushing local-ahead refs.
- Changing the `merged-into-main → auto-close` logic in `apm sync`.
- UI changes (this is purely a CLI-read behaviour fix).

### Acceptance criteria

- [ ] `apm list` (aggressive on) reads origin's commit content for a ticket whose local ref is strictly behind origin — the displayed state matches origin, not the stale local ref
- [ ] `apm list --no-aggressive` continues to read local refs unchanged; no classification overhead runs
- [ ] A ticket whose local ref is ahead of origin still shows the local (unpushed) state in `apm list` and `apm show`
- [ ] A ticket whose local ref has diverged from origin shows local state; a warning line is printed (not silently swallowed)
- [ ] In `apm list` output, each ticket read from origin because local was strictly behind is marked with `*` adjacent to its ID
- [ ] A stale-ref summary block is printed at the bottom of `apm list` output whenever any `*` tickets are present, naming each one and suggesting `apm sync` to reconcile local refs
- [ ] `apm show <id>` (aggressive on) reads from origin when local is behind and prints a note line stating the local ref is behind origin and naming `apm sync` as the remedy
- [ ] The `/api/tickets` response includes a `local_stale` boolean field; `TicketCard` in the web UI renders a visible badge on tickets where `local_stale` is true

### Out of scope

- Auto-pushing local-ahead refs to origin
- Changing the `merged-into-main → auto-close` logic in `apm sync`
- Modifying the `--no-aggressive` flag semantics or defaults
- Any changes to `apm sync` itself — this ticket is about read-time behaviour only
- Option A (fast-forwarding local refs inside the aggressive-fetch path)

### Approach

Option B is chosen (per amendment request): classify refs at read time and prefer origin content when origin is strictly ahead. Local refs are never modified by this path — that remains `apm sync`'s job. The benefit over Option A is that the `*` indicator persists, giving the user explicit signal that local refs are behind rather than silently reconciling them.

#### Phase 1 — new `read_from_branch_with_class` in `apm-core/src/git_util.rs`

Add:
```rust
pub fn read_from_branch_with_class(
    root: &Path, branch: &str, rel_path: &str,
) -> Result<(String, BranchClass)> {
    let local_ref  = format!("refs/heads/{branch}");
    let remote_ref = format!("origin/{branch}");
    let class = classify_branch(root, &local_ref, &remote_ref);
    let content = match &class {
        BranchClass::Behind | BranchClass::RemoteOnly | BranchClass::Equal => {
            run(root, &["show", &format!("{remote_ref}:{rel_path}")])
                .or_else(|_| run(root, &["show", &format!("{branch}:{rel_path}")]))?
        }
        BranchClass::Ahead | BranchClass::NoRemote | BranchClass::Diverged => {
            run(root, &["show", &format!("{branch}:{rel_path}")])
                .or_else(|_| run(root, &["show", &format!("{remote_ref}:{rel_path}")]))?
        }
    };
    Ok((content, class))
}
```

Keep the existing `read_from_branch` as a thin wrapper that discards the class (backward compat for callers that don't care).

#### Phase 2 — `local_stale` field on `Ticket` in `apm-core/src/ticket/`

Add an ephemeral (not serialized, skipped by `Ticket::parse`) boolean field:
```rust
pub local_stale: bool,   // true when BranchClass::Behind at read time
```

Modify `load_all_from_git` (`ticket_util.rs`) to call `read_from_branch_with_class`; set `t.local_stale = matches!(class, BranchClass::Behind)`. `RemoteOnly` is a first-fetch, not a staleness signal — do not mark it. `Diverged` is not marked stale either (local content was used; a warning is emitted separately).

#### Phase 3 — `apm list` asterisk + summary (`apm/src/cmd/list.rs`)

When rendering each row, prepend `*` to the ID column when `t.local_stale`. Collect stale tickets into a `Vec<(id, title)>`. After the table, if any:
```
  * local ref behind origin — run `apm sync` to fast-forward:
      *9944425e  apm list with aggressive fetch ...
```

No asterisk or block when `--no-aggressive` (the flag already bypasses classification in Phase 1).

#### Phase 4 — `apm show` note (`apm/src/cmd/show.rs`)

Replace the `read_from_branch` call with `read_from_branch_with_class`. If `BranchClass::Behind`, print before the ticket body:
```
note: local ref is behind origin — showing origin content (run `apm sync` to fast-forward)
```
If `BranchClass::Diverged`, print:
```
warning: local ref has diverged from origin — showing local content
```

#### Phase 5 — web UI `local_stale` badge

- `apm-server/src/handlers/tickets.rs`: add `local_stale: bool` to the ticket response DTO; populate from `t.local_stale`.
- `apm-ui/src/components/supervisor/types.ts`: add `local_stale?: boolean` to the `Ticket` interface.
- `apm-ui/src/components/supervisor/TicketCard.tsx`: render a small badge (reuse the existing `?` / `A` badge pattern, label it `↑` or `stale`) when `local_stale` is true.

### Open questions


### Amendment requests

- [x] take option B, and do provide a feedback to user in cli and ux if the local branch is older than head. and asterisk in the cli list, followed by details at bottom of list for example. Something similar in UI

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-21T20:48Z | — | new | philippe|philippepascal |
| 2026-05-21T22:56Z | new | groomed | philippepascal |
| 2026-05-21T23:15Z | groomed | in_design | philippepascal |