+++
id = 28
title = "pre-push hook passes extra args to apm _hook causing clap error"
state = "closed"
priority = 5
effort = 1
risk = 1
author = "claude-0326-2222-8071"
agent = "claude-0327-1854-10aa"
branch = "ticket/0028-pre-push-hook-passes-extra-args-to-apm-h"
created_at = "2026-03-27T05:42:59.348405Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The `.git/hooks/pre-push` script installed by `apm init` passes `"$@"` to
`apm _hook pre-push`. Git calls the pre-push hook as `pre-push <remote> <url>`,
so the shell expands to `apm _hook pre-push origin https://...`. Clap sees
`pre-push` as the hook name and then `origin` as an unexpected positional
argument, printing an error. The `|| true` in the script masks the failure so
the push succeeds, but the hook does nothing — first-push auto-transitions
(`ready → in_progress`) never fire.

### Acceptance criteria

- [ ] No clap error printed to stderr during `git push` on any ticket branch
- [ ] `apm _hook pre-push` executes successfully when called as `apm _hook pre-push origin https://...`
- [ ] The pre-push hook script installed by `apm init` is updated to the correct invocation

### Out of scope

- Changing what the hook does beyond fixing the invocation
- Other git hooks (post-merge is unaffected)

### Approach

Drop `"$@"` from the hook script — the remote name and URL are not used by
`apm _hook`. The hook reads pushed ref info from stdin, not from argv. Update
the hook template string in `cmd/init.rs` `write_hooks()`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T05:42Z | — | new | claude-0326-2222-8071 |
| 2026-03-27T06:22Z | new | specd | claude-0326-2222-8071 |
| 2026-03-28T01:00Z | specd | ready | apm |
| 2026-03-28T01:54Z | ready | in_progress | claude-0327-1854-10aa |
| 2026-03-28T01:56Z | in_progress | implemented | claude-0327-1854-10aa |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |