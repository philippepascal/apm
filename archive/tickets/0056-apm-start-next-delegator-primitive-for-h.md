+++
id = 56
title = "apm start --next: delegator primitive for headless dispatch"
state = "closed"
priority = 4
effort = 7
risk = 4
author = "claude-0329-1200-a1b2"
agent = "claude-0329-main"
branch = "ticket/0056-apm-start-next-delegator-primitive-for-h"
created_at = "2026-03-29T19:11:56.426262Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

There is no way to run a headless "dispatch loop" — a supervisor process that
continuously picks up the next actionable ticket and assigns it to a worker
agent. Today an orchestrator must:

1. Call `apm next --json` to find a ticket
2. Parse the JSON to extract the ID and state
3. Call `apm start <id>` to claim it and provision the worktree
4. Separately read the state's `instructions` file to know what prompt to give
   the worker
5. Optionally extract a `focus_section` hint that a supervisor may have
   attached when sending the ticket back for revision

Each of these steps is a separate process invocation with its own round of
branch reads, making scripted orchestration fragile and verbose. There is also
no standard way to attach a `focus_section` hint to a ticket in its
frontmatter so that the next worker knows what to pay attention to.

The goal is a single command — `apm start --next` — that atomically finds,
claims, and composes the full agent prompt for the highest-priority actionable
ticket.

### Acceptance criteria

- [x] `apm start --next` with no actionable ticket exits 0 and prints "No actionable tickets."
- [x] `apm start --next` claims the highest-priority actionable ticket: sets state → `in_progress`, assigns `APM_AGENT_NAME`, provisions the worktree, and merges the default branch (same behavior as `apm start <id>`)
- [x] When the claimed ticket's state config has `instructions = "path/to/file"`, the contents of that file are read from the repo root and included in the composed output
- [x] When a `focus_section` field is present in the ticket's frontmatter, `apm start --next` injects the hint "Pay special attention to section: <name>" into the composed output, then writes the ticket back with that field removed and commits the change
- [x] `apm start --next --spawn` launches a `claude` subprocess with the composed prompt as the system prompt and the ticket content as the user message (same spawn behavior as `apm start --spawn`)
- [x] `apm start --next` without `--spawn` prints the worktree path and the composed agent prompt to stdout
- [x] `apm start --next -P` (skip permissions) works when combined with `--spawn`

### Out of scope

- Looping: the command runs once and exits; a shell `while` loop wraps it for continuous dispatch
- Load balancing across multiple concurrent agents
- Choosing between spec-writer and implementation agent (that is determined by the `instructions` field in the state config)
- Adding `focus_section` to the transition config (it is already there in `TransitionConfig`); this ticket only adds it to the ticket frontmatter so a supervisor can attach it when sending a ticket back

### Approach

**1. Add `focus_section` to `Frontmatter`**

In `apm-core/src/ticket.rs`, add an optional field to `Frontmatter`:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub focus_section: Option<String>,
```

This field is set by a supervisor (e.g. via `apm review`) when sending a
ticket back to an agent with a specific focus. `apm start --next` consumes
and clears it.

**2. Extend the `Start` subcommand in `main.rs`**

Add a `--next` flag to the `Start` variant and make `id` optional:

```rust
Start {
    id: Option<u32>,  // now optional
    #[arg(long)] next: bool,
    ...
}
```

When `--next` is given, `id` must be `None`; when absent, `id` must be
`Some`. Validate this in `main.rs` before dispatching.

**3. Implement `cmd/start.rs` — `--next` path**

Extract the next-ticket selection logic (mirrors `cmd/next.rs`) directly into
`start.rs`: load all tickets, filter to agent-actionable and unclaimed, sort
by score, take the first. If none, print "No actionable tickets." and return
`Ok(())`.

Once a ticket ID is selected, run the existing start flow (claim, worktree,
merge). Then:

a. Look up the state config for the ticket's **original** state (before the
   transition) using `config.workflow.states.iter().find(|s| s.id == old_state)`.

b. If `state_config.instructions` is `Some(path)`, read the file with
   `std::fs::read_to_string(root.join(path))`. If the file is missing, emit a
   warning and continue with an empty instructions string.

c. If `ticket.frontmatter.focus_section` is `Some(name)`:
   - Append "Pay special attention to section: <name>" to the prompt.
   - Clear the field (`focus_section = None`), re-serialize the ticket, and
     commit to the ticket branch with message
     `ticket(<id>): clear focus_section`.

d. Compose the final prompt as:
   ```
   <instructions file contents>

   Pay special attention to section: <focus_section>   (if set)
   ```

e. Without `--spawn`: print `Worktree: <path>` followed by
   `Prompt:\n<composed prompt>`.

f. With `--spawn`: pass the composed prompt as `--system-prompt` and the
   serialized ticket content as the user message to `claude --print`, same
   as the existing spawn path in `run()`.

**4. Tests**

Integration tests in `apm/tests/integration.rs`:
- No actionable tickets → exits 0, output contains "No actionable tickets."
- Actionable ticket exists → ticket branch has `in_progress` state, worktree
  directory present.
- State config has `instructions` pointing to an existing file → instructions
  text appears in stdout.
- Ticket frontmatter has `focus_section` set → hint line appears in stdout,
  and the field is absent from the ticket branch after the command.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:11Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T20:36Z | new | in_design | claude-spec-56 |
| 2026-03-29T20:38Z | in_design | specd | claude-spec-56 |
| 2026-03-29T20:49Z | specd | ready | claude-0329-main |
| 2026-03-29T20:53Z | ready | in_progress | claude-0329-main |
| 2026-03-29T21:02Z | in_progress | implemented | claude-0329-main |
| 2026-03-29T22:51Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |