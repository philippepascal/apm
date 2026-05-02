+++
id = "f06272f1"
title = "Worker permission-denial diagnostics: surface what was denied and classify (especially apm commands)"
state = "specd"
priority = 0
effort = 5
risk = 4
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/f06272f1-worker-permission-denial-diagnostics-sur"
created_at = "2026-05-01T02:31:05.749604Z"
updated_at = "2026-05-02T03:39:54.331906Z"
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

**Dependencies:** builds on the wrapper epic (4312fbd4). The Claude wrapper already captures `claude --output-format stream-json` stdout to `.apm-worker.log` (`apm-core/src/wrapper/builtin/claude.rs`, `spawn_local()`). Confirm the epic branch is merged before starting.

#### Step 1 — Pin the JSONL denial event format

Before writing scanner code, examine an actual `.apm-worker.log` that captured a permission denial:

- Find one in `.apm--worktrees/` from a past incident, or trigger one in a controlled test by running a worker with `--skip-permissions=false` and manually denying a tool prompt.
- In Claude's `stream-json` output, a permission denial appears as a `tool_result` event with `is_error: true`. The content field contains a human-readable message — likely something like `"This tool call was not executed"` or `"denied"`. A regular command failure also has `is_error: true`, but its content is the stderr/stdout of the failed command.
- Identify the distinguishing substring(s) in the content that uniquely mark a denial vs. a failure.
- Record the exact event shape in a block comment at the top of the new module so future maintainers understand the contract.

If this cannot be determined before coding, open an `### Open questions` entry instead of guessing.

#### Step 2 — New module `apm-core/src/denial.rs`

Expose via `apm-core/src/lib.rs` (`pub mod denial;`).

**Data structures** (derive `Serialize`, `Deserialize`, `Debug`, `Clone`):

```rust
pub enum DenialClass {
    ApmCommandDenial,
    OutsideWorktree,
    UnknownPattern,
}

pub struct DenialEntry {
    pub timestamp: String,       // ISO-8601, from the log event
    pub tool: String,            // e.g. "Bash", "Edit", "Write"
    pub input: String,           // truncated to ≤200 chars
    pub classification: DenialClass,
}

pub struct DenialSummary {
    pub ticket_id: String,
    pub worker_exited_at: String,  // ISO-8601 (current time at scan)
    pub log_path: String,          // absolute path to .apm-worker.log
    pub denial_count: usize,
    pub denials: Vec<DenialEntry>,
}
```

**`pub fn scan_transcript(log_path: &Path, worktree: &Path, ticket_id: &str) -> DenialSummary`:**

1. Read `log_path` line by line; return an empty summary if the file is missing or unreadable.
2. First pass: build `HashMap<String, (String, String, String)>` mapping `tool_use_id → (tool_name, tool_input, timestamp)` from tool_use events. In the `stream-json` format, tool use appears inside the `content` array of `assistant` message events — parse each line, check `type == "assistant"`, then walk `message.content[]` for items with `type == "tool_use"`.
3. Second pass: for each `tool_result` event with `is_error: true` whose content matches the denial indicator string (from Step 1), look up the `tool_use_id` to retrieve name, input, and timestamp.
4. Classify each match: tool is `Bash` and `input` starts with `"apm "` → `ApmCommandDenial`; tool is `Edit` or `Write` and path does not start with `worktree.to_string_lossy()` → `OutsideWorktree`; otherwise → `UnknownPattern`.
5. Truncate `input` to 200 characters.
6. Return `DenialSummary { ticket_id, worker_exited_at: now(), log_path: absolute, denial_count, denials }`.

**`pub fn write_summary(summary_path: &Path, summary: &DenialSummary)`:** Serialize to pretty-printed JSON via `serde_json::to_string_pretty` and write to `summary_path`. Log and swallow errors (best-effort — don't crash the wrapper exit path).

#### Step 3 — Wire into Claude wrapper exit

File: `apm-core/src/wrapper/builtin/claude.rs`, after the subprocess `wait()` call:

1. Derive `summary_path` from `ctx.log_path` by replacing `.log` extension with `.summary.json`.
2. Call `denial::scan_transcript(&ctx.log_path, &ctx.worktree_path, &ctx.ticket_id)`.
3. Call `denial::write_summary(&summary_path, &summary)`.
4. If any `ApmCommandDenial` entries are present, call `logger::log("worker-diag", "apm_command_denial", &format!("ticket {} denied apm commands: {}", ctx.ticket_id, unique_commands))` where `unique_commands` is a comma-joined list of unique `apm ...` inputs.

#### Step 4 — `apm workers diag <id>` subcommand

**`apm/src/cmd/workers.rs`** — add `pub fn run_diag(root: &Path, ticket_id: &str)`:

1. Resolve `ticket_id` to a worktree path (same `.apm-worker.pid` scan used by `run()`). If not found, print to stderr and `std::process::exit(1)`.
2. Try to deserialize `worktree/.apm-worker.summary.json`; if absent, call `denial::scan_transcript()` on `worktree/.apm-worker.log`. If both are absent, print an error and exit non-zero.
3. Print the report — total count, per-category breakdown, unique apm commands with allowlist fix hint, and absolute log path. When `denial_count == 0`, print "no denials detected" instead.

```
Worker denial report — <ticket_id>
Log: <absolute .apm-worker.log path>

Total denials: N
  apm_command_denial : X
  outside_worktree   : Y
  unknown_pattern    : Z

APM command denials (allowlist gaps):
  apm doesnotexist  (2026-05-01T12:34:50Z)
  → Add "Bash(apm doesnotexist*)" to .claude/settings.json
    and to APM_ALLOW_ENTRIES in apm-core/src/init.rs
```

**`apm/src/main.rs`:** Add `--diag <id>` flag to the `workers` subcommand (follow the existing `--log` / `--kill` clap pattern). Route to `workers::run_diag(root, id)`.

#### Step 5 — Tests

Fixture files under `apm-core/tests/fixtures/`:

- `transcript_apm_denial.jsonl` — minimal stream-json with one assistant tool_use (`Bash`, `apm doesnotexist`) and a denied tool_result. Use the exact format confirmed in Step 1.
- `transcript_no_denials.jsonl` — clean transcript, no errors.
- `transcript_outside_worktree.jsonl` — one denied Edit to `/etc/passwd`.

Unit tests in `apm-core/src/denial.rs` (or `apm-core/tests/denial_test.rs`):

- `test_apm_command_denial`: scan `transcript_apm_denial.jsonl`; assert `denial_count == 1`, `classification == ApmCommandDenial`, `tool == "Bash"`, `input` starts with `"apm "`.
- `test_no_denials`: scan `transcript_no_denials.jsonl`; assert `denial_count == 0`.
- `test_outside_worktree`: scan `transcript_outside_worktree.jsonl` with worktree = `/fake/worktree`; assert `denial_count == 1`, `classification == OutsideWorktree`.

The three integration-test acceptance criteria are covered by these unit tests against fixture transcripts — no live Claude run required.

### Dependencies

This ticket builds on the wrapper epic (4312fbd4). The Claude wrapper already captures `claude --output-format stream-json` stdout to `.apm-worker.log` (`apm-core/src/wrapper/builtin/claude.rs`, `spawn_local()`, lines 60-62). Confirm the epic branch is merged before starting.

### Step 1 — Pin the JSONL denial event format

Before writing scanner code, examine an actual `.apm-worker.log` that captured a permission denial:

- Find one in `.apm--worktrees/` from a past incident, or trigger one in a controlled test by running a worker with `--skip-permissions=false` and manually denying a tool prompt.
- In Claude's `stream-json` output, a permission denial appears as a `tool_result` (or equivalent) event with `is_error: true`. The content field contains a human-readable message — likely something like `"This tool call was not executed"` or `"denied"`. A regular command failure also has `is_error: true`, but its content is the stderr/stdout of the failed command.
- Identify the distinguishing substring(s) in the content that uniquely mark a denial vs. a failure.
- Record the exact event shape in a block comment at the top of the new module so future maintainers understand the contract.

If this cannot be determined before coding, open an `### Open questions` entry instead of guessing.

### Step 2 — New module `apm-core/src/denial.rs`

Expose via `apm-core/src/lib.rs` (`pub mod denial;`).

**Data structures** (derive `Serialize`, `Deserialize`, `Debug`, `Clone`):

```rust
pub enum DenialClass {
    ApmCommandDenial,
    OutsideWorktree,
    UnknownPattern,
}

pub struct DenialEntry {
    pub timestamp: String,       // ISO-8601, from the log event
    pub tool: String,            // e.g. "Bash", "Edit", "Write"
    pub input: String,           // truncated to ≤200 chars
    pub classification: DenialClass,
}

pub struct DenialSummary {
    pub ticket_id: String,
    pub worker_exited_at: String,  // ISO-8601 (current time at scan)
    pub log_path: String,          // absolute path to .apm-worker.log
    pub denial_count: usize,
    pub denials: Vec<DenialEntry>,
}
```

**`pub fn scan_transcript(log_path: &Path, worktree: &Path, ticket_id: &str) -> DenialSummary`:**

1. Read `log_path` line by line; return an empty summary if the file is missing or unreadable.
2. First pass: build `HashMap<String, (String, String, String)>` mapping `tool_use_id → (tool_name, tool_input, timestamp)` from tool_use events. In the `stream-json` format, tool use appears inside the `content` array of `assistant` message events — parse each line, check `type == "assistant"`, then walk `message.content[]` for items with `type == "tool_use"`.
3. Second pass: for each `tool_result` event (or equivalent) with `is_error: true` whose content matches the denial indicator string (from Step 1), look up the `tool_use_id` to retrieve name, input, and timestamp.
4. Classify each match:
   - Tool is `Bash` and `input` (trimmed) starts with `"apm "` → `ApmCommandDenial`
   - Tool is `Edit` or `Write` and the path in `input` does not start with `worktree.to_string_lossy().as_ref()` → `OutsideWorktree`
   - Otherwise → `UnknownPattern`
5. Truncate `input` to 200 characters.
6. Return `DenialSummary { ticket_id, worker_exited_at: now(), log_path: absolute, denial_count, denials }`.

**`pub fn write_summary(summary_path: &Path, summary: &DenialSummary)`:**

Serialize to pretty-printed JSON via `serde_json::to_string_pretty` and write to `summary_path`. Log and swallow errors (best-effort — don't crash the wrapper exit path).

### Step 3 — Wire into Claude wrapper exit

File: `apm-core/src/wrapper/builtin/claude.rs`

After the subprocess exits (after the `wait()` call in `spawn_local()` or its caller):

1. Derive `summary_path` from `ctx.log_path` by replacing the `.log` extension with `.summary.json` (or appending `.summary.json` if no `.log` suffix).
2. Call `denial::scan_transcript(&ctx.log_path, &ctx.worktree_path, &ctx.ticket_id)`.
3. Call `denial::write_summary(&summary_path, &summary)`.
4. If `summary.denials` contains any `ApmCommandDenial` entry, call `logger::log("worker-diag", "apm_command_denial", &format!("ticket {} denied apm commands: {}", ctx.ticket_id, unique_commands))` — where `unique_commands` is a comma-joined list of the unique `apm ...` strings from those entries.

### Step 4 — `apm workers diag <id>` subcommand

**`apm/src/cmd/workers.rs` — add `pub fn run_diag(root: &Path, ticket_id: &str)`:**

1. Resolve `ticket_id` to a worktree path using the same scan of `.apm-worker.pid` files that `run()` uses, or via the ticket index. If not found, print an error to stderr and `std::process::exit(1)`.
2. Derive paths: `log_path = worktree/.apm-worker.log`, `summary_path = worktree/.apm-worker.summary.json`.
3. Load summary: try `denial::read_summary(&summary_path)`; if absent, call `denial::scan_transcript(&log_path, &worktree, ticket_id)`. If both fail (files absent), print an error and exit non-zero.
4. Print the report:

```
Worker denial report — <ticket_id>
Log: <absolute .apm-worker.log path>

Total denials: N
  apm_command_denial : X
  outside_worktree   : Y
  unknown_pattern    : Z

APM command denials (allowlist gaps):
  apm doesnotexist  (2026-05-01T12:34:50Z)
  → Add "Bash(apm doesnotexist*)" to .claude/settings.json
    and to APM_ALLOW_ENTRIES in apm-core/src/init.rs

No denials detected.   ← replaces the block above when denial_count == 0
```

**`apm/src/main.rs`:** Add `--diag <id>` flag to the `workers` subcommand (follow the existing `--log` / `--kill` pattern in the clap setup). Route to `workers::run_diag(root, id)`.

### Step 5 — Tests

**Fixture files** under `apm-core/tests/fixtures/`:

- `transcript_apm_denial.jsonl` — minimal valid stream-json with one assistant tool_use (`Bash`, `apm doesnotexist`) followed by a denied tool_result. Use the exact format confirmed in Step 1.
- `transcript_no_denials.jsonl` — clean transcript, no errors.
- `transcript_outside_worktree.jsonl` — one denied Edit to `/etc/passwd`.

**Unit tests in `apm-core/src/denial.rs`** (or `apm-core/tests/denial_test.rs`):

- `test_apm_command_denial`: scan `transcript_apm_denial.jsonl` with worktree = `/fake/worktree`; assert `denial_count == 1`, `denials[0].classification == ApmCommandDenial`, `denials[0].tool == "Bash"`, `denials[0].input` starts with `"apm "`.
- `test_no_denials`: scan `transcript_no_denials.jsonl`; assert `denial_count == 0`.
- `test_outside_worktree`: scan `transcript_outside_worktree.jsonl` with worktree = `/fake/worktree`; assert `denial_count == 1`, `denials[0].classification == OutsideWorktree`.

The three integration-test acceptance criteria are satisfied by these unit tests operating on fixture transcripts — no live Claude run required.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T02:31Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:30Z | groomed | in_design | philippepascal |
| 2026-05-02T03:39Z | in_design | specd | claude-0502-0330-3b10 |
