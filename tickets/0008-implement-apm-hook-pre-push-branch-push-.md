+++
id = 8
title = "Implement apm _hook pre-push (branch_push_first event)"
state = "closed"
priority = 5
effort = 3
risk = 3
updated_at = "2026-03-27T00:06:00.679157Z"
+++

## Spec

### Problem

The `pre-push` hook installed by `apm init` calls `apm _hook pre-push "$@"`.
This subcommand does not exist yet. Its job is to detect when a `ticket/<id>-*`
branch is being pushed while the ticket is in `ready` state, and fire the
`event:branch_push_first` auto-transition (`ready → in_progress`). This is a
fallback for agents that push to the ticket branch without first running `apm start`.

Note: in the branch-per-ticket model, the ticket branch is already created and
pushed by `apm new`. The hook therefore cannot use a "null remote SHA" check to
detect a first push — it should simply check the ticket's current state.

### Acceptance criteria

- [ ] `apm _hook pre-push` reads push refs from stdin in git's pre-push format: `<local_ref> <local_sha> <remote_ref> <remote_sha>`
- [ ] For each ref whose branch name matches `ticket/<id>-*`: loads the corresponding ticket from the local cache
- [ ] If the ticket is in `ready` state: transitions it to `in_progress`, commits frontmatter update to the ticket branch via `git::commit_to_branch`
- [ ] If the ticket is not in `ready` state: no-op (idempotent)
- [ ] If no matching ticket is found for a ref: skips silently (never blocks the push)
- [ ] Hook always exits 0; a failure to update a ticket prints a warning to stderr but does not block the push

### Out of scope

- Other hook types (`post-merge`, etc.)
- Provider webhook handling
- Auto-transitions other than `event:branch_push_first`

### Approach

New subcommand `apm _hook <hook-name>` in `apm/src/cmd/hook.rs`, dispatching to
hook-specific handlers.

`pre-push` handler:
1. Read stdin lines: `<local_ref> <local_sha> <remote_ref> <remote_sha>`
2. For each line: extract branch name from `local_ref`; match against `ticket/(\d+)-.*`; extract `id`
3. Load ticket by id from local cache; if state == `"ready"`:
   - Update state to `"in_progress"`, update `updated`, append history
   - Serialize and call `git::commit_to_branch(root, &branch, &rel_path, &content, &msg)`
   - Print: `#<id>: ready → in_progress (branch push)`
4. Wrap all errors as warnings to stderr; always exit 0

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-26 | manual | ready → ready | Respec: ticket/ branches, drop null-SHA detection |
| 2026-03-26 | manual | ready → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |