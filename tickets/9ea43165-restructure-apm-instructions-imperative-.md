+++
id = "9ea43165"
title = "Restructure apm instructions: imperative format, role filtering, ticket-id substitution, layer reorder"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9ea43165-restructure-apm-instructions-imperative-"
created_at = "2026-05-30T16:49:25.808040Z"
updated_at = "2026-05-30T17:21:47.541231Z"
depends_on = ["a3c34ddc"]
+++

## Spec

### Problem

GOAL: make apm instructions output dramatically easier for an LLM worker to consume by (a) replacing descriptive prose with imperative tables and concrete commands, (b) filtering to only what the role can act on, (c) substituting concrete ticket IDs where currently the output uses <id> placeholders, and (d) reordering the three prompt layers so role-specific rules sit at the highest-attention position.

This ticket depends on a3c34ddc (shell-discipline relocation). It assumes a3c34ddc has landed so the apm instructions output is already shorter and agent-agnostic before this restructure begins.

PROBLEM: the current apm instructions output is descriptive, role-undifferentiated, and uses placeholder strings the worker cannot copy-paste. Example today:

  ### Ready (ready)
  Actionable by: agent
    → in_progress, trigger: command:start, role: coder

A worker has to mentally parse the state-machine grammar and re-substitute the placeholders. The actionable form is much shorter and more directly usable:

  Ready → in_progress     apm state abc12345 in_progress

REQUIREMENTS:

1) IMPERATIVE FORMAT for the state machine section. Replace the verbose per-state block with a compact transitions table: from-state → to-state, then the exact apm command. One row per allowed transition. Drop the trigger/role columns (they are not actionable by the worker). Keep enough information that the worker knows WHY each transition exists (a one-line gloss is fine) but the column layout should make the apm command unmistakable.

2) ROLE FILTERING. When apm instructions is invoked with a role (the dispatcher always knows the role), only emit transitions actionable by that role and only commands the role is permitted to invoke. A coder worker should not see spec-writer transitions, and vice versa. Same treatment for the command-reference section: filter to the role's permitted command set. The role is already passed to the helpers; this is a render-time filter.

3) TICKET-ID SUBSTITUTION. apm instructions accepts an optional ticket id argument (e.g. apm instructions abc12345 --role coder). When present, every occurrence of the literal placeholder <id> in the rendered output is substituted with the actual ticket id, so commands like apm state <id> in_progress render as apm state abc12345 in_progress. The dispatch path in apm-core/src/start.rs (build_system_prompt and its call sites) must be updated to pass the ticket id when invoking the instructions helper so workers receive the substituted form. Without a ticket id, the output keeps the <id> literal so humans inspecting the instructions still see a usable template.

4) NO-ROLE LISTING. apm instructions without a role argument prints a short index of available roles, each with a one-line description, mirroring what apm prompt already surfaces. Same shared helper underneath if possible.

5) LAYER REORDER. In apm-core/src/start.rs::build_system_prompt, swap the layer order from L1 (apm instructions) -> L2 (.apm/project.md) -> L3 (role file) to L3 -> L2 -> L1. Rationale: after a3c34ddc, the role file carries the rules a worker needs on every tool call (shell discipline, the don'ts), while apm instructions becomes purely reference material (state machine, command reference). Putting the role file first places the highest-frequency rules in the highest-attention position of the prompt. apm prompt --explain output should reflect the new order.

OUT OF SCOPE:
- The shell-discipline content (covered by a3c34ddc).
- The cascade that resolves the role file (unchanged).
- apm-server / apm-ui display of instructions content.
- New placeholder substitutions beyond <id> (the worker mostly needs the ticket id; other dynamic values can be added later if a need surfaces).
- Per-ticket personalization beyond <id> substitution (no insertion of frontmatter fields, history, etc.).

TESTS:
- apm instructions --role coder against the default workflow produces a transitions table whose rows are ONLY transitions a coder can act on; spec-writer transitions are absent.
- apm instructions abc12345 --role coder produces output where no occurrence of the literal <id> remains; every command line contains abc12345 instead.
- apm instructions (no args) prints a roles index that includes coder, spec-writer, main-agent and a one-line gloss for each.
- A snapshot or stable-assertion test for the imperative format output to guard against accidental reversion to the descriptive form.
- The build_system_prompt unit tests assert the new order (role file content appears before project content which appears before apm instructions content).
- apm prompt --explain reflects the new layer order in its labelling (layer 1 → role file, layer 2 → project, layer 3 → apm instructions). Update any tests asserting layer labels.
- A dispatch integration test that spawns a worker (or stops short of spawning) and asserts the ticket id appears substituted in the system prompt sent to the worker.

### Acceptance criteria

- [ ] `apm instructions --role coder` emits a Markdown table under `## State Machine` with rows only for transitions the coder role can act on; rows for `in_design`, `groomed`, and `specd` are absent from the table.
- [ ] `apm instructions --role spec-writer` emits a Markdown table with no rows for `in_progress` or `implemented`.
- [ ] The state machine table produced for any role has a header row with `From`, `To`, and `Command` columns; each data row contains the exact runnable apm command (`apm start <id>` for `command:start` transitions, `apm state <id> <to>` for all others).
- [ ] `apm instructions abc12345 --role coder` produces output containing no occurrence of the literal string `<id>`; every `Command` cell contains `abc12345`.
- [ ] `apm instructions` (no arguments) prints a role index that includes `coder`, `spec-writer`, and `main-agent`, each with a one-line description; the output does not contain a `## State Machine` section.
- [ ] `build_system_prompt` called with `ticket_id = Some("abc12345")` returns a string where no `<id>` literal appears in the instructions layer and `abc12345` appears in every former `<id>` position.
- [ ] `build_system_prompt` called with both a project file and a ticket id returns output in which the role-file content appears at an earlier character position than the project content, which appears before the instructions content.
- [ ] `apm prompt --explain` shows "layer 1:" identifying the role file and "layer 3:" identifying `apm instructions`; these labels are the reverse of the previous order.
- [ ] `cargo test --workspace` passes after all test additions and updates.

### Out of scope

- Shell-discipline content movement (covered by a3c34ddc).
- The cascade that resolves which role file to use (unchanged).
- apm-server / apm-ui display of instructions content.
- New placeholder substitutions beyond `<id>` (e.g., no `<branch>`, no frontmatter field insertion).
- Per-ticket personalization beyond `<id>` substitution (no history insertion, no dependency injection into the instructions layer).
- Changing the command-reference section format (it remains a flat aligned list, not a table).
- The `apm prompt` command itself beyond updating `--explain` layer labels and passing `ticket_id` through `build_system_prompt`.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T16:49Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:21Z | groomed | in_design | philippepascal |