+++
id = "f06272f1"
title = "Worker permission-denial diagnostics: surface what was denied and classify (especially apm commands)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f06272f1-worker-permission-denial-diagnostics-sur"
created_at = "2026-05-01T02:31:05.749604Z"
updated_at = "2026-05-02T03:30:32.013461Z"
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

- [ ] When a worker exits with ≥1 permission-denial event in its transcript, the wrapper writes `.apm-worker.summary.json` alongside `.apm-worker.log` in the ticket worktree
- [ ] When a worker exits with zero permission-denial events in its transcript, `.apm-worker.summary.json` is written with an empty `denials` array and `denial_count: 0`
- [ ] Each entry in `denials` contains: `timestamp` (ISO-8601), `tool` (tool name e.g. "Bash"), `input` (truncated to ≤200 chars), and `classification`
- [ ] A denied Bash call whose `command` starts with `apm ` is classified as `apm_command_denial`
- [ ] A denied Edit or Write call whose path falls outside the ticket worktree root is classified as `outside_worktree`
- [ ] Any denial not matching the above two patterns is classified as `unknown_pattern`
- [ ] `apm workers diag <id>` reads `.apm-worker.summary.json` if it exists; falls back to scanning `.apm-worker.log` directly if the summary is absent
- [ ] `apm workers diag <id>` prints a report with total denial count and a per-category breakdown (`apm_command_denial`, `outside_worktree`, `unknown_pattern`)
- [ ] When `apm_command_denial` entries are present, the report lists each unique denied apm command and recommends adding it to `.claude/settings.json` and to APM's init template (`apm-core/src/init.rs` `APM_ALLOW_ENTRIES`)
- [ ] The report includes the absolute path to `.apm-worker.log`
- [ ] When there are no denials, `apm workers diag <id>` prints "no denials detected"
- [ ] `apm workers diag <id>` exits non-zero with an error message if `<id>` cannot be resolved to a ticket worktree
- [ ] When a worker exits with ≥1 `apm_command_denial` entry, a one-line warning is appended to the APM main log (default `/tmp/apm.log`) via the existing `logger::log` facility
- [ ] Integration test: a fixture JSONL transcript containing a denied `apm doesnotexist` Bash call produces one `apm_command_denial` entry; `apm workers diag` surfaces it in its report
- [ ] Integration test: a fixture JSONL transcript with no denials produces a zero-entry summary; `apm workers diag` reports "no denials detected"
- [ ] Integration test: a fixture JSONL transcript containing a denied Edit to `/etc/passwd` produces one `outside_worktree` entry

### Out of scope

- Auto-editing `.claude/settings.json` to fix missing allowlist entries (that is the worker side-quest we are explicitly preventing)
- Real-time denial interception — pausing or rerouting the worker mid-run when a denial occurs; the transcript is scanned after exit, which is sufficient
- Privacy redaction of tool inputs beyond truncation to ≤200 characters
- Surfacing denials from non-Claude wrappers (mock-happy, mock-sad, debug) — only the Claude wrapper produces a parseable stream-json JSONL transcript
- Automatically filing issues or PRs for `apm_command_denial` entries; the report recommends action but takes none
- Modifying the Claude binary or Claude Code SDK's permission-prompt behaviour
- Any UI beyond the CLI report (`apm workers diag`)
- Retrospective scanning of historical worker logs written before this feature lands (no guarantee of format consistency)
- Classifying or reporting on tool errors that are not permission denials (e.g. Bash commands that exit non-zero for other reasons)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:31Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:30Z | groomed | in_design | philippepascal |