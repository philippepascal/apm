+++
id = "e8ae2764"
title = "Add opt-in push from apm sync CLI prompt, flag, and UI button"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e8ae2764-add-opt-in-push-from-apm-sync-cli-prompt"
created_at = "2026-04-18T02:21:50.164931Z"
updated_at = "2026-04-18T06:42:41.556290Z"
depends_on = ["b15354a6"]
+++

## Spec

### Problem

`apm sync` currently has no way for the user to push `<default>` from inside the sync flow. The message "run `git push` when ready" instructs the user to leave sync, run `git push` manually in a separate shell, and re-run sync to close merged tickets. This is a clean separation of concerns (sync never pushes automatically, per the multi-user-safety principle), but it's friction in the common case where the user *does* want to push right now.

Add a user-authorized push path — never automatic, always opt-in — on both the CLI and the UI.

**CLI — `apm sync`:**
- In interactive mode (stdin is a TTY, output not quiet), when `<default>` is ahead of `origin/<default>`, prompt the user: "push <default> to origin now? [y/N]". On `y`, run `git push origin <default>` and re-check close candidates after. On `N` or non-interactive, print the informational message as today and proceed.
- Add a non-interactive flag (name open: `--push-main`, `--push-default`, `--auto-push`) that pushes `<default>` without prompting when it's ahead. Intended for scripts and cron jobs that want one-shot behavior.
- Same opt-in shape should also apply to ahead `ticket/*` and `epic/*` branches in `sync_non_checked_out_refs` (prompt per branch or bundled; TBD during spec), so the user doesn't have to push those separately after sync.

**UI — Sync screen / modal:**
- When the `/api/sync` response includes an "ahead" info line for `<default>`, render a `Push <default>` button next to it. Clicking the button triggers a server endpoint that runs `git push origin <default>` and then re-runs the sync flow, returning the updated result (including the now-available close candidates).
- Add a persistent user preference (setting / checkbox) — "Automatically push default branch when ahead during sync" — that, when enabled, skips the button and pushes immediately on sync. Default off (preserves the no-auto-push principle for users who haven't opted in).
- Same button pattern for ahead ticket/epic branches if scope allows.

**Guardrails (must hold in both surfaces):**
- Never push without explicit user action (interactive confirmation, flag, or persisted preference)
- Never push when `<default>` has diverged from `origin/<default>` — that case already prints the `MAIN_DIVERGED_*` guidance and requires manual resolution
- Never push mid-merge (the mid-merge bail from ticket `5cf54181` remains the top-level gate)
- Respect the existing `--offline` semantics: no push attempts in offline mode

Trigger: user hit the manual-push friction on 2026-04-17. After `apm sync` reported "main is ahead of origin/main by 16 commits", they had to alt-tab, run `git push`, then re-run sync to close the merged ticket — three context switches for one intent.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:33Z | groomed | in_design | philippepascal |
| 2026-04-18T06:38Z | in_design | ready | apm |
| 2026-04-18T06:39Z | ready | groomed | apm |
| 2026-04-18T06:39Z | groomed | in_design | philippepascal |
| 2026-04-18T06:42Z | in_design | groomed | apm |
| 2026-04-18T06:42Z | groomed | in_design | philippepascal |
