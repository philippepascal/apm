+++
id = 49
title = "Remove all auto-transitions: state advances via explicit commands only"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0328-c72b"
agent = "claude-0328-c72b"
branch = "ticket/0049-remove-all-auto-transitions-state-advanc"
created_at = "2026-03-28T20:12:28.488323Z"
updated_at = "2026-03-28T20:16:44.050925Z"
+++

## Spec

### Problem

APM has three auto-transitions that advance ticket state in response to git
events, without any explicit `apm` command:

1. **`branch_push_first` → `ready` → `in_progress`** (pre-push hook): fires
   whenever anything is pushed to a ticket branch while the ticket is in
   `ready`. Intended to catch agents that skip `apm start` and push directly.
   In practice it fires on any push — including `apm state`'s own aggressive-mode
   push — making it impossible to transition a ticket to `ready` without it
   immediately self-advancing to `in_progress`.

2. **`pr_opened` → `in_progress` → `implemented`** (GitHub provider): fires
   when a PR is opened against the ticket branch. A PR being opened is not a
   reliable signal that implementation is complete — it could be a draft, a WIP,
   or opened for early review.

3. **`pr_all_merged` → `implemented` → `accepted`** (local git detection): fires
   when all PRs for the ticket are merged. A merge is not the same as supervisor
   sign-off — it could be a revert, a partial merge, or happen out of sequence.

All three assume they can infer intent from git events, but git activity is
ambiguous. The correct signals are explicit `apm` commands: `apm start` to begin
work, `apm state <id> implemented` when done, `apm state <id> accepted` as
supervisor sign-off. `apm sync` already detects merged branches and surfaces them
for the supervisor to act on — that is the right place for merge detection, with
a human in the loop.

### Acceptance criteria

- [x] The three `[[workflow.auto_transitions]]` blocks are removed from `apm.toml`
- [x] The `pre-push` hook no longer auto-transitions tickets; `apm _hook pre-push`
  becomes a no-op (or the hook is removed from `.git/hooks/pre-push` during
  `apm init` if it has no other purpose)
- [x] The `event:pr_opened` detection path in `apm sync` no longer triggers a
  state transition
- [x] The `event:pr_all_merged` / branch-merged detection in `apm sync` no longer
  triggers a state transition; it may still surface the information to the user
  (e.g. "PR merged — run `apm state <id> accepted` to advance") but does not act
- [x] Existing tickets already in `in_progress` or `accepted` via auto-transition
  are unaffected (no data migration needed — frontmatter states are valid)
- [x] All existing tests pass; any tests that assert auto-transition behaviour are
  updated or removed

### Out of scope

- Removing `apm sync`'s ability to detect merged branches (it should still report
  them — just not act on them automatically)
- Changing any manual transitions or the `command:start` trigger on `apm start`
- Removing the pre-push hook infrastructure entirely (it may be repurposed later)

### Approach

**`apm.toml`**: delete all three `[[workflow.auto_transitions]]` blocks.

**`apm/src/cmd/hook.rs`**: the `pre_push` function currently reads stdin and
auto-transitions `ready → in_progress`. Remove the transition logic; keep the
function as a no-op so the hook plumbing doesn't break if `.git/hooks/pre-push`
still calls `apm _hook pre-push`.

**`apm/src/cmd/sync.rs`** (and any provider code): find where `event:pr_opened`
and `event:pr_all_merged` / branch-merged are handled and remove the state
transition calls. If the merged-branch detection produces useful output, change
it to print a suggestion rather than act.

**`apm-core`**: grep for any references to `auto_transitions` in config parsing
or sync logic and remove or stub them appropriately.

**Tests**: update integration tests that assert `ready → in_progress` on push,
or `implemented → accepted` on merge.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T20:12Z | — | new | claude-0328-c72b |
| 2026-03-28T20:14Z | new | specd | claude-0328-c72b |
| 2026-03-28T20:15Z | specd | ready | apm |
| 2026-03-28T20:15Z | ready | in_progress | hook |
| 2026-03-28T20:16Z | in_progress | ready | claude-0328-c72b |
| 2026-03-28T20:16Z | ready | in_progress | claude-0328-c72b |