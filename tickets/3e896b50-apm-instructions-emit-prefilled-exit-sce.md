+++
id = "3e896b50"
title = "apm instructions: emit prefilled exit-scenario cheat sheet per worker state"
state = "implemented"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3e896b50-apm-instructions-emit-prefilled-exit-sce"
created_at = "2026-06-02T18:34:29.328988Z"
updated_at = "2026-06-02T19:48:49.117733Z"
+++

## Spec

### Problem

GOAL: replace prose teaching in role files about end-of-work transitions with a config-driven, prefilled cheat sheet that apm instructions emits dynamically. The worker stops needing to learn the apm CLI; they match their situation to a labelled scenario and copy the command verbatim. The ticket id (and any other dynamic value) is already substituted.

CURRENT PROBLEM: role files (apm.coder.md, apm.spec-writer.md, etc.) carry prose like 'when done, run apm state <id> implemented' and 'when blocked, write your question in Open questions and transition to blocked'. This duplicates information that lives in workflow.toml; it forces the worker to construct commands from descriptions; and small format changes (a renamed transition, a new state, a tweaked section name) require updating multiple role files. Workers occasionally fumble the shell quoting or forget the order of operations (write section first, then transition).

DESIGN:

When apm instructions is called with a ticket id and a role, after the existing State Machine table, emit a new section that enumerates the exit scenarios available from the ticket's current state. Each scenario is a prefilled, copy-pasteable command (or sequence of commands) with the ticket id and any known values already filled in. Free-form placeholders (the worker's question text, etc.) are marked with angle-bracket placeholders the worker fills in.

SCHEMA ADDITIONS (in apm-core/src/config.rs::TransitionConfig):

- worker_hint: Option<String> — free-form prose describing when this transition applies from the worker's perspective. Example values: 'If you completed the implementation and tests pass', 'If you lack information to proceed (write your question first)', 'If the spec is wrong and needs supervisor revision'. Used as the heading for the scenario.
- worker_pre: Option<String> — an optional single command to run BEFORE the transition. The id placeholder dollar-id (in TOML, the literal string less-than id greater-than) is substituted just like in the existing state-machine table. Example: apm spec less-than id greater-than --section 'Open questions' --append '<your question text>'. The generator emits worker_pre on one line and the apm state transition on the next.

FILTER RULE (config-driven, no hardcoded state names):

The cheat sheet enumerates outgoing transitions from the ticket's current state, including only those where worker_hint is set. The current state must have worker_profile set (since otherwise the worker would not be dispatched on this ticket — this is the existing semantic from epic 9c3c4c20). Transitions without worker_hint are not promoted in the cheat sheet; they remain available via apm state but the worker is not prompted to use them. This way the configuration alone — not hardcoded heuristics about state names or outcome types — decides what the worker sees.

OUTPUT FORMAT (in apm-core/src/instructions.rs::generate, appended after the State Machine table when a ticket id is supplied):

   ## Exit scenarios

   Choose the matching scenario and run the commands. Replace any less-than placeholder greater-than text with your own.

   ### If you completed the implementation and tests pass

   apm state abc12345 implemented

   ### If you lack information to proceed (write your question first)

   apm spec abc12345 --section 'Open questions' --append '<your question text>'
   apm state abc12345 blocked

Each scenario has the worker_hint as the heading; if worker_pre is set its substituted form is the first command; the apm state X line follows. Ticket id substitution reuses the existing dollar-id mechanism from 9ea43165.

When no ticket id is supplied (apm instructions --role coder with no id), the cheat sheet is omitted — only the static state machine table appears. This matches the current behaviour where dollar-id placeholders are left unsubstituted.

ROLE FILE SLIMMING:

After this lands, the role files (.apm/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md, plus the apm-core/src/default/agents/claude/ copies) can shed the 'When done', 'When blocked', 'Handling ammend tickets', etc. sections that today repeat the exit logic. The role files should reference the cheat sheet generically: 'At end of work, see Exit scenarios in apm instructions for the exact commands.' Role files focus on what is truly role-specific (path discipline, permitted commands subset, coding/spec-writing style).

CONFIG MIGRATION:

Default workflow.toml (apm-core/src/default/workflow.toml) gains worker_hint and worker_pre entries on the relevant transitions: groomed → in_design, ammend → in_design (spec-writer scenarios), in_progress → implemented, in_progress → blocked (coder scenarios), and any others where the worker is the actor. Specific worker_hint and worker_pre texts are spec-writer's call; sensible defaults below for reference:

- in_progress → implemented: worker_hint = 'If you completed the implementation and tests pass'; no worker_pre
- in_progress → blocked: worker_hint = 'If you lack information to proceed (write your question first)'; worker_pre = apm spec less-than id greater-than --section 'Open questions' --append '<your question text>'
- in_design → specd: worker_hint = 'If you finished writing or revising the spec'; no worker_pre (the spec-writer wrote sections via apm spec already)
- in_design → question: worker_hint = 'If you cannot proceed during design without more info'; worker_pre = apm spec less-than id greater-than --section 'Open questions' --append '<your question text>'

Project workflow.toml (.apm/workflow.toml) gets the same treatment. Other projects (e.g., syn) update via apm init (writes a .init template the user diffs against — same migration path as established in earlier epic-cleanup discussions).

ACCEPTANCE CRITERIA hints (for the spec-writer to refine):
- apm instructions less-than ticket-id greater-than --role coder emits an Exit scenarios section after the State Machine table for tickets in worker-owned states
- The section enumerates only outgoing transitions whose worker_hint is set on the ticket's current state
- Each scenario heading is the transition's worker_hint text
- If worker_pre is set, it appears as the first command line, with the ticket id substituted
- The apm state line always follows, with the ticket id substituted
- apm instructions --role coder with NO ticket id does NOT emit the Exit scenarios section (ticket-id-dependent content is suppressed without an id)
- A workflow.toml transition with no worker_hint does not appear in the cheat sheet
- The filter relies on worker_profile being set on the current state; no state names are hardcoded in the filter
- The default workflow.toml has worker_hint and worker_pre populated on the transitions named above
- Role files (default and project copies) no longer contain the 'When done', 'When blocked', 'Handling ammend tickets' sections that the cheat sheet now subsumes; a short reference to apm instructions Exit scenarios replaces them
- Unit tests for the cheat-sheet generator: workflow with two states (one worker-owned with two transitions, one supervisor-owned with one transition); call generator with the worker-owned state's ticket; assert only the two worker-owned transitions appear; assert worker_pre is emitted when set and absent when not; assert worker_hint appears as heading
- Snapshot or stable-text assertion test that the default workflow's cheat sheet for in_progress matches the expected two scenarios in order

OUT OF SCOPE:
- Schema changes to states (no new fields on StateConfig)
- Removing the static State Machine table — it remains, the cheat sheet is additive
- Multi-step worker_pre (an array of pre-commands). For now, worker_pre is a single string. If multiple commands turn out to be needed for a real scenario, promote to an array later.
- Validating worker_pre's shell syntax in apm validate. The string is treated as opaque; the worker copies and runs it.
- Changing what the worker is permitted to run (the WORKER_COMMAND_ALLOWLIST from 9c66e199 is independent and continues to gate which commands the worker can invoke in its session)
- apm-server / apm-ui surfaces for exit scenarios
- Bash completion or shell-integration for the prefilled commands

REFERENCES:
- apm-core/src/config.rs::TransitionConfig (add the two optional fields)
- apm-core/src/instructions.rs::generate (extend output)
- apm-core/src/default/workflow.toml (populate defaults)
- .apm/workflow.toml (populate same)
- apm-core/src/default/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md (slim)
- .apm/agents/claude/* (mirror)
- 9ea43165 — established the less-than id greater-than substitution mechanism this reuses
- Epic 9c3c4c20 — established the state.worker_profile concept used by the filter
- Discussion in conversation history: supervisor sketched the design after observing how heavy worker instructions are today

### Acceptance criteria

- [x] `apm instructions <id>` emits `## Exit scenarios` after the State Machine table when the ticket's current state has `worker_profile` set
- [x] Each scenario heading is the transition's `worker_hint` text, formatted as a `###` heading
- [x] When `worker_pre` is set on a transition, it appears as the first command line in the scenario with `<id>` substituted with the ticket id
- [x] The `apm state <id> <to>` line appears in every scenario with `<id>` substituted
- [x] Transitions without `worker_hint` are excluded from the Exit scenarios section
- [x] `apm instructions` with no ticket id does not emit an Exit scenarios section
- [x] A ticket whose current state has no `worker_profile` produces no Exit scenarios section
- [x] Both `apm-core/src/default/workflow.toml` and `.apm/workflow.toml` have `worker_hint` (and `worker_pre` where applicable) on: `in_progress → implemented`, `in_progress → blocked`, `in_design → specd`, and `in_design → question`
- [x] The coder and spec-writer role files (both default under `apm-core/src/default/agents/claude/` and project copies under `.apm/agents/claude/`) no longer contain exit-command prose; a short reference to Exit scenarios in `apm instructions` replaces it
- [x] Unit tests cover: only hinted transitions from worker-profile states appear; supervisor-state transitions are excluded even when `worker_hint` is set; `worker_pre` is emitted before `apm state` when set, and absent when not
- [x] Stable-text test: the default workflow's cheat sheet for a ticket in `in_progress` contains the two expected scenario headings in order: implemented first, blocked second

### Out of scope

- No new fields on `StateConfig` — only `TransitionConfig` gains `worker_hint` and `worker_pre`
- The static State Machine table is unchanged; the cheat sheet is purely additive
- `worker_pre` remains a single string — multi-step pre-command arrays are not supported
- Shell syntax of `worker_pre` values is not validated by `apm validate`
- `WORKER_COMMAND_ALLOWLIST` in `instructions.rs` is unchanged
- apm-server and apm-ui do not surface exit scenarios
- No Bash completion or shell integration for the prefilled commands
- `apm init` migration templates for other projects are not updated by this ticket

### Approach

#### Step 1 — config.rs: add fields to TransitionConfig

In `apm-core/src/config.rs`, add two fields to `TransitionConfig` after the existing `outcome` field. Both must carry `#[serde(default)]` because the struct has `#[serde(deny_unknown_fields)]`:

```rust
#[serde(default)]
pub worker_hint: Option<String>,
#[serde(default)]
pub worker_pre: Option<String>,
```

#### Step 2 — instructions.rs: extend generate() and add exit_scenarios_body()

Add `current_state: Option<&str>` as a fifth parameter to `generate()`. After step 1 (State Machine), insert before the Ticket Format section:

```rust
if ticket_id.is_some() {
    let body = exit_scenarios_body(config.as_ref(), current_state);
    if !body.is_empty() {
        out.push_str("## Exit scenarios\n\n");
        out.push_str(&body);
    }
}
```

New private helper `exit_scenarios_body(config: Option<&Config>, current_state: Option<&str>) -> String`:
1. Return empty string if config or current_state is None.
2. Find `StateConfig` where `state.id == current_state`; return empty if not found or `state.worker_profile` is None.
3. Collect transitions where `transition.worker_hint.is_some()`; return empty if none.
4. Emit intro: `"Choose the matching scenario and run the commands. Replace any <placeholder> text with your own.\n\n"`.
5. For each qualifying transition: emit `### <hint>\n\n`, then `<worker_pre>\n` if set, then `apm state <id> <to>\n`. Use the literal `<id>` — the existing end-of-generate() substitution handles replacement.

Update the caller in `apm/src/cmd/instructions.rs`: when `ticket_id` is provided, load the ticket and read its current state field; pass it as `current_state`. Update all existing tests in `instructions.rs` to add `None` as the fifth argument (no existing test supplies a ticket id, so no assertion changes).

#### Step 3 — workflow.toml: annotate transitions

Add `worker_hint` (and `worker_pre` where applicable) to the four existing transitions in both `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`:

- `in_progress → implemented`: `worker_hint = "If you completed the implementation and tests pass"`
- `in_progress → blocked`: `worker_hint = "If you lack information to proceed (write your question first)"`, `worker_pre = "apm spec <id> --section 'Open questions' --append '<your question text>'"`
- `in_design → specd`: `worker_hint = "If you finished writing or revising the spec"`
- `in_design → question`: `worker_hint = "If you cannot proceed during design without more info"`, `worker_pre = "apm spec <id> --section 'Open questions' --append '<your question text>'"`

Apply the identical changes to `.apm/workflow.toml`.

#### Step 4 — slim role files

In `apm-core/src/default/agents/claude/apm.coder.md`, replace the `apm state <id> implemented` instruction at the end of **"Tests and finishing"** and the three-step procedure in **"Blocked state"** with a single line: `"At end of work, follow **Exit scenarios** in \`apm instructions\` for the exact commands."`

In `apm-core/src/default/agents/claude/apm.spec-writer.md`, replace the `apm state <id> specd` step in **"When you are done"** and the `apm state <id> question` step in **"Open questions"** with the same reference line.

Mirror both edits to `.apm/agents/claude/apm.coder.md` and `.apm/agents/claude/apm.spec-writer.md`.

#### Step 5 — tests

**Unit test** (inline in `apm-core/src/instructions.rs`): build a TOML config with two states — `in_progress` (`worker_profile = "claude/coder"`) with two transitions (`→ implemented` with `worker_hint`, `→ blocked` without), and `specd` (no `worker_profile`) with one transition (`→ closed` with `worker_hint`). Call `generate()` with `current_state = Some("in_progress")`. Assert: (a) output contains `## Exit scenarios`; (b) exactly one `###` scenario appears (the hinted `in_progress` transition); (c) the un-hinted transition is absent; (d) the `specd` transition is absent. Test a second config where `worker_pre` is set — assert it appears before the `apm state` line. Test with `current_state = None` — assert no Exit scenarios section.

**Stable-text test**: call `generate()` with the real `apm-core/src/default/workflow.toml` config and `current_state = "in_progress"`. Assert the two expected scenario headings appear in order: implemented first, blocked second.

### Open questions


### Amendment requests

- [x] Remove the addition of a new in_progress to ammend transition from this ticket's scope. The original context I supplied (the Problem section's output-format mock and the worker_hint/worker_pre defaults list) mentioned this scenario, but in_progress to ammend does not currently exist in either workflow.toml, and adding it is a workflow behavior change beyond the cheat-sheet/role-slimming scope of this ticket. Concrete changes: (1) Drop step 3's instruction to add a new in_progress to ammend transition block to both workflow.toml files. (2) Drop the in_progress to ammend example from the worker_hint/worker_pre defaults list in the Problem section. (3) Update the Output Format mock to show only the two in_progress scenarios that actually exist after this ticket (implemented and blocked); remove the third 'If the spec is wrong' scenario. (4) Update the stable-text test in step 5 from 'three expected scenario headings' to 'two expected scenario headings' for in_progress; assert implemented first, blocked second. (5) Keep all four other transition annotations as specified (in_progress to implemented, in_progress to blocked, in_design to specd, in_design to question). If at some future point we want a direct in_progress to ammend path, file a separate ticket for that workflow change.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-02T18:34Z | — | new | philippepascal |
| 2026-06-02T18:39Z | new | groomed | philippepascal |
| 2026-06-02T18:40Z | groomed | in_design | philippepascal |
| 2026-06-02T18:49Z | in_design | specd | claude |
| 2026-06-02T19:15Z | specd | ammend | philippepascal |
| 2026-06-02T19:15Z | ammend | in_design | philippepascal |
| 2026-06-02T19:23Z | in_design | specd | claude |
| 2026-06-02T19:29Z | specd | ready | philippepascal |
| 2026-06-02T19:29Z | ready | in_progress | philippepascal |
| 2026-06-02T19:48Z | in_progress | implemented | claude |
