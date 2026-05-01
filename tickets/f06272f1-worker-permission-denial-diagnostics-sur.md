+++
id = "f06272f1"
title = "Worker permission-denial diagnostics: surface what was denied and classify (especially apm commands)"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f06272f1-worker-permission-denial-diagnostics-sur"
created_at = "2026-05-01T02:31:05.749604Z"
updated_at = "2026-05-01T02:31:05.749604Z"
+++

## Spec

### Problem

When a worker hits a permission denial mid-run, today the only signal is a buried `is_error: true` entry in `.apm-worker.log` (~hundreds of KB of JSONL). The supervisor has no easy way to know what was denied or whether it indicates a missing default allowlist entry.

**Specific motivation:** if an apm command (e.g. `apm worktrees`, `apm validate`, `chmod`, `mkdir` etc.) hits a permission prompt inside a worker, that is *APM's defect* — apm commands should be in the default allowlist that `apm init` writes. The existence of such a denial is signal we should be acting on.

**Concrete incident (2026-04-30):** worker for ticket 2803bf07 hit Bash permission prompts during legitimate amendment work, which triggered it to invoke the `fewer-permission-prompts` skill (a side-quest that ate ~124 KB of transcript and never resolved). Had APM surfaced the denial up front ("your worker tried to run `chmod 755 ...`; this is not in the default allowlist"), we could have updated the init template directly instead of leaving it to the worker to improvise.

**Should land after the wrapper epic (4312fbd4) — the wrapper layer captures the transcript stream and is the natural place to scan for denials in real time or on exit.**

**Scope — two integration points:**

1. **Worker-exit summary in `.apm-worker.log` (or sibling file).** When the wrapper exits, scan the captured transcript for permission-denial entries. Append a structured summary (or write a sibling file like `.apm-worker.summary.json`) listing each denied tool call with: timestamp, tool name (Bash/Edit/Read/etc.), tool input (truncated for privacy), and a classification:
   - `apm_command_denial` — denied a Bash call starting with `apm `. APM should never deny apm commands; this is a default-allowlist gap.
   - `outside_worktree` — denied a path outside the ticket worktree. Expected if the path validator (sibling ticket) is enforcing.
   - `unknown_pattern` — other denial categories.
2. **`apm workers diag <id>` command.** Reads the summary (or scans the log if no summary exists). Prints a human-readable report:
   - Total denials, broken down by category.
   - For `apm_command_denial` entries: list each unique apm command and recommend the user file an issue (or a quick fix: add the entry to `.claude/settings.json` and to APM's init template at `apm-core/src/default/...`).
   - Pointer to the full transcript path.

**Bonus (low effort given the above):** at every worker spawn, also scan the worker's transcript at exit. If any `apm_command_denial` entries are present, write a one-line warning to APM's main log (`/tmp/apm.log` per current config) so the supervisor sees it without having to run a command per worker.

**Out of scope:**
- Auto-fixing the allowlist (auto-editing `.claude/settings.json` from APM). That is the worker's failed side-quest from the incident; we explicitly do not want APM to do it either.
- Real-time intervention (catching denials as they happen and rerouting the worker). The transcript is captured after the fact; that is sufficient.
- Privacy redaction of tool inputs in the summary. Truncate at a reasonable length; users should treat the summary as containing the same content as the transcript.

**Acceptance pointers:**
- Integration test: spawn a worker that issues a denied apm command (e.g. `apm doesnotexist`); after exit, the summary identifies it as `apm_command_denial` and the diag command surfaces it.
- Integration test: spawn a worker with no denials; the summary reports zero entries and the diag command says "no denials detected."
- Integration test: a worker with a denial outside the worktree (e.g. `Edit /etc/passwd`) — categorised as `outside_worktree`, not `apm_command_denial`.

**Cross-ticket interaction:** complements the path validator (separate ticket). The validator prevents writes outside the worktree; this ticket reports them. Together, the supervisor knows what the worker tried, even when prevention worked.

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
| 2026-05-01T02:31Z | — | new | philippepascal |