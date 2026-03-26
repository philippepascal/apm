+++
id = 8
title = "Implement apm _hook pre-push (branch_push_first event)"
state = "specd"
priority = 5
effort = 3
risk = 3
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

The `pre-push` hook installed by `apm init` calls `apm _hook pre-push "$@"`.
This subcommand does not exist yet. Its job is to detect when a feature branch
matching `feature/<id>-*` is being pushed for the first time and fire the
`event:branch_push_first` auto-transition (`ready → in_progress`). Without it,
the auto-transition never fires and the agent must manually run `apm state <id> in_progress`.

### Acceptance criteria

- [ ] `apm _hook pre-push` parses stdin in the format git provides to pre-push hooks
- [ ] For each ref being pushed, if the branch name matches `feature/<id>-*` and has no upstream (first push), fire `event:branch_push_first`
- [ ] Firing the event transitions the ticket from `ready` to `in_progress` and commits the frontmatter update to main
- [ ] If the ticket is not in `ready` state, the event is a no-op (idempotent)
- [ ] If no matching ticket is found, the hook exits 0 (never blocks the push)
- [ ] Hook exits 0 on success; a failure to update the ticket prints a warning but does not block the push

### Out of scope

- Other hook types (`post-merge`, etc.)
- Provider webhook handling
- Auto-transitions other than `event:branch_push_first`

### Approach

New subcommand `apm _hook <hook-name>` dispatching to hook-specific handlers.
`pre-push` handler:
1. Read stdin lines: `<local_ref> <local_sha> <remote_ref> <remote_sha>`
2. For each line where `remote_sha` is the null SHA (`0000000...`): this is a first push
3. Extract branch name from `local_ref`; match against `feature/(\d+)-.*`
4. Load ticket by id; if state == `ready`, apply transition to `in_progress`, commit to main
5. The hook must not itself push (avoid recursion); the main push proceeds normally after

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
