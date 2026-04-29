+++
id = "7ba021e8"
title = "apm help workflow: render workflow.toml schema from WorkflowConfig struct"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7ba021e8-apm-help-workflow-render-workflow-toml-s"
created_at = "2026-04-28T19:28:15.496296Z"
updated_at = "2026-04-29T07:38:06.867120Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_workflow()` function in `apm/src/cmd/help.rs` (introduced as a stub by ticket bc89e0a0) returns a placeholder string and does nothing useful. As a result, `apm help workflow` gives users no actionable information about what fields are valid in `.apm/workflow.toml` (or in the `[workflow]` section of `apm.toml`), their types, defaults, or purpose.

The auto-derive infrastructure from ticket 069c3403 can render any `JsonSchema`-annotated struct as a formatted reference table. The types that govern workflow config — `WorkflowConfig`, `StateConfig`, `TransitionConfig`, `PrioritizationConfig`, `SatisfiesDeps`, `CompletionStrategy` — already have `JsonSchema` derived on them (by 069c3403), but most of their fields carry no Rust doc comments today. Since `schemars` converts `/// doc comments` directly into the `description` column of the rendered table, the output would be almost entirely blank without first adding those comments.

This ticket does two things: (1) adds meaningful doc comments to all fields on the workflow-related config types, drawing on the existing spec in `docs/strategy-and-dependencies.md`; (2) replaces the `render_workflow()` stub with a real implementation that calls `apm_core::help_schema::render_schema::<WorkflowConfig>()`.

### Acceptance criteria

- [ ] `apm help workflow` exits 0 and prints non-empty output
- [ ] The output contains the path `workflow.states[].id`
- [ ] The output contains the path `workflow.states[].transitions[].completion`
- [ ] The output contains the path `workflow.prioritization.priority_weight`
- [ ] The `completion` field entry lists all five variants: `pr`, `merge`, `pull`, `pr_or_epic_merge`, `none`
- [ ] The `satisfies_deps` field entry shows its two forms (`bool | string`) or equivalent variant description
- [ ] Every field entry in the output has a non-empty description (no blank `#` column)
- [ ] Fields with numeric defaults (`priority_weight = 10`, `effort_weight = -2`, `risk_weight = -1`) show those defaults in the output
- [ ] The output includes a brief preamble line stating that `workflow.states` is an array of user-defined state objects

### Out of scope

- Content for `render_commands()`, `render_config()`, `render_ticket()` — those are tickets 3665e017, d486d183, and 14214305 respectively
- The `apm help` dispatcher and topic routing — established by ticket bc89e0a0
- The `help_schema` infrastructure (`schema_entries`, `render_schema`, `FieldEntry`) — that is ticket 069c3403
- Adding `JsonSchema` derives to workflow types — that is ticket 069c3403
- ANSI colour or markdown rendering in the output
- Pager integration (`less`/`more`)
- A workflow design tutorial or worked examples beyond doc-comment text
- Validation rules for workflow config (those belong to `apm validate`)
- `LocalConfig` and `LocalWorkersOverride` — internal override file, not user-facing

### Approach

**File changes: two files.**

---

**1. `apm-core/src/config.rs` — add doc comments**

Add `///` doc comments to every field (and at the struct/enum level) for the six workflow-related types below. Use `docs/strategy-and-dependencies.md` as the authoritative source for `CompletionStrategy` variant descriptions.

- `WorkflowConfig` struct:
  - struct-level: "Defines the ticket state machine and prioritization weights. Loaded from `.apm/workflow.toml` or the `[workflow]` section of `apm.toml`."
  - `states`: "Ordered list of ticket states. Users define their own state IDs and transition graph."
  - `prioritization`: "Weights used to rank tickets in `apm next` and `apm list`."

- `StateConfig` struct:
  - struct-level: "A single state in the workflow state machine."
  - `id`: "Unique state identifier (e.g. `new`, `in_progress`). Used in ticket frontmatter and transition targets."
  - `label`: "Human-readable name shown in `apm list` and review prompts."
  - `description`: "Optional longer explanation of what this state means."
  - `terminal`: "When `true`, tickets in this state are considered done; no further transitions are expected."
  - `worker_end`: "When `true`, a worker finishing in this state is considered complete (used by the dispatcher to release the worker slot)."
  - `satisfies_deps`: "Whether reaching this state satisfies `depends_on` relationships. `false` = never, `true` = always, a string tag = satisfies deps tagged with that string."
  - `dep_requires`: "Optional string tag that must appear in a dependency's `satisfies_deps` for it to count as satisfied."
  - `transitions`: "List of outgoing transitions from this state."
  - `actionable`: "Roles that can actively pick up / act on tickets in this state. Valid values: `agent`, `supervisor`, `engineer`, `any`. Drives `apm next`, `apm start`, and `apm list --actionable`."
  - `instructions`: "Optional extra instructions injected into the worker prompt when a ticket enters this state."

- `TransitionConfig` struct:
  - struct-level: "A directed edge in the state machine: from the parent state to `to`."
  - `to`: "Target state ID after this transition fires."
  - `trigger`: "Event or command that fires this transition (e.g. `close`, `approve`)."
  - `label`: "Short label shown in the review prompt (e.g. `Approve for implementation`)."
  - `hint`: "Guidance shown in the editor header (e.g. `Add requests in ### Amendment requests`)."
  - `completion`: "How the worker's branch is integrated before or after this transition. See `CompletionStrategy`."
  - `focus_section`: "Markdown section heading the agent should focus on when acting on this transition."
  - `context_section`: "Markdown section heading included as extra context for the agent."
  - `warning`: "Optional warning message shown to the supervisor before the transition is confirmed."
  - `profile`: "Worker profile to use for the agent spawned by this transition. References a key in `[worker_profiles]`."

- `PrioritizationConfig` struct:
  - struct-level: "Weights used to compute the priority score for ticket selection in `apm next`."
  - `priority_weight`: "Multiplier applied to the ticket's `priority` field. Default: 10.0."
  - `effort_weight`: "Multiplier applied to the ticket's `effort` field (negative favours low-effort). Default: -2.0."
  - `risk_weight`: "Multiplier applied to the ticket's `risk` field (negative favours low-risk). Default: -1.0."

- `SatisfiesDeps` enum:
  - enum-level: "Controls when reaching the parent state satisfies `depends_on` relationships on other tickets."
  - `Bool(bool)`: "`false` = this state never satisfies dependencies; `true` = it always does."
  - `Tag(String)`: "Satisfies only dependencies annotated with this string tag via `dep_requires`."

- `CompletionStrategy` enum:
  - enum-level: "Determines how a worker's branch is integrated as part of a state transition."
  - `Pr`: "Opens a pull request against the default branch. Transition fires when the PR is opened, not when it merges; downstream tickets may start before upstream code lands."
  - `Merge`: "Merges directly to `target_branch`. Composes dependencies when ticket and all deps share the same `target_branch`."
  - `Pull`: "Pulls from an upstream branch into the ticket branch without opening a PR."
  - `PrOrEpicMerge`: "Recommended default. Opens a PR to the default branch when the ticket has no epic; merges to the epic branch when it does. Composes dependencies within an epic."
  - `None`: "No automatic branch integration. Downstream tickets cannot rely on upstream code being present."

---

**2. `apm/src/cmd/help.rs` — replace `render_workflow()` stub**

Replace the current stub body with a preamble string concatenated with the output of `render_schema::\<WorkflowConfig\>()` from `apm_core::help_schema`:

- First line: "workflow.toml — state-machine and prioritization configuration"
- Second line: "workflow.states is an array of user-defined state objects; each element defines one node in the ticket state machine."
- Blank line separator
- Then the full output of `render_schema::\<WorkflowConfig\>()`

No other changes to `help.rs` are needed.

---

**Implementation order:**
1. Add doc comments to the six types in `apm-core/src/config.rs` (no derive changes — 069c3403 owns those).
2. Replace the `render_workflow()` stub in `apm/src/cmd/help.rs`.
3. `cargo build` to confirm compilation (both dependency tickets must already be merged onto the epic branch).
4. Run `apm help workflow` and verify all nine acceptance criteria.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:28Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:52Z | groomed | in_design | philippepascal |
| 2026-04-28T19:57Z | in_design | specd | claude-0428-1952-7128 |
| 2026-04-29T03:42Z | specd | ready | philippepascal |
| 2026-04-29T07:38Z | ready | in_progress | philippepascal |
