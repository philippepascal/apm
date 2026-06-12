+++
id = "92b34957"
title = "claude seem to think sync is needed to list tickets."
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/92b34957-claude-seem-to-think-sync-is-needed-to-l"
created_at = "2026-06-09T21:59:01.000886Z"
updated_at = "2026-06-12T08:08:24.428439Z"
+++

## Spec

### Problem

Agent instructions in three shipped role files teach Claude that `apm sync` must run before `apm list`. The Shell Discipline section of `apm.main-agent.md`, `apm.spec-writer.md`, and `apm.coder.md` all use `apm sync && apm list --state ready` as the canonical "wrong chaining" example, with the "right" version showing the two commands as sequential calls. This trains every Claude role — not just the main agent — to treat sync as a list prerequisite.

The main-agent startup sequence reinforces the false dependency by placing `apm sync` directly before `apm list --state in_progress` and describing it as "refresh local cache from all `ticket/*` branches". There is no filesystem cache; `apm list` reads git refs directly and returns results whether or not sync has been run. The description misleads agents into thinking list depends on a cache that sync populates.

A third, weaker signal: `apm list` prints "local ref behind origin — run `apm sync` to fast-forward" when stale refs are detected. Alone this would read as a suggestion, but combined with the instruction patterns above it reads as confirmation of the supposed dependency.

### Acceptance criteria

- [ ] The Shell Discipline section in `apm.main-agent.md` no longer uses `apm sync` and `apm list` as the sequential example
- [ ] The Shell Discipline section in `apm.spec-writer.md` no longer uses `apm sync` and `apm list` as the sequential example
- [ ] The Shell Discipline section in `apm.coder.md` no longer uses `apm sync` and `apm list` as the sequential example
- [ ] The startup sequence description of `apm sync` no longer says "refresh local cache" — it describes what sync actually does (fast-forward local branches to match remote)
- [ ] Each source file change in `apm-core/src/default/agents/claude/` is mirrored in the deployed copy under `.apm/agents/claude/`
- [ ] `cargo test --workspace` passes after the changes

### Out of scope

- Changing how `apm list` reads data — it already reads git refs directly without requiring sync
- Rewording the stale-ref suggestion in `apm list` output ("run `apm sync` to fast-forward") — it is conditional and advisory; the false dependency comes from the instructions, not from this message
- Adding `apm list` or `apm sync` to the dynamic `apm instructions` command reference — supervisor commands are documented in role files, not in the dynamic output
- Changing the startup sequence order or removing `apm sync` from it — sync is genuinely useful for freshness; the problem is the description and the Shell Discipline example, not sync's presence in the sequence

### Approach

Three source files in `apm-core/src/default/agents/claude/` are the canonical source; their deployed copies in `.apm/agents/claude/` must be updated in the same commit.

#### Shell Discipline example (all three role files)

In `apm.main-agent.md`, `apm.spec-writer.md`, and `apm.coder.md`, replace the Shell Discipline "Wrong/Right" block that chains `apm sync && apm list --state ready`. The replacement should still demonstrate the no-chaining rule using commands that do not imply a sync→list dependency. A natural substitution is:

```
  # Wrong — && chains defeat allow-list matching
  apm sync && apm next --json

  # Right — one call per operation
  apm sync
  apm next --json
```

`apm next` is a supervisor command used immediately after sync in the actual startup sequence, making this a realistic and semantically coherent example that teaches the same shell discipline lesson without coupling sync to list.

Apply the identical change to both the source copy (`apm-core/src/default/agents/claude/<file>.md`) and the deployed copy (`.apm/agents/claude/<file>.md`) for each of the three role files.

#### Startup sequence description (main-agent only)

In `apm.main-agent.md` (both source and deployed), change the startup sequence step 2 from:

```
2. `apm sync` — refresh local cache from all `ticket/*` branches
```

to:

```
2. `apm sync` — fast-forward local ticket branches to match remote
```

"Local cache" is factually wrong and is the primary source of the false dependency. The replacement is accurate: sync fetches and fast-forwards local refs; it does not populate any filesystem cache that list would then read.

#### Verification

Run `cargo test --workspace` to confirm no regressions. No Rust source files change, so compilation is fast; the test suite exercises instruction generation and should pass unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-09T21:59Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:00Z | groomed | in_design | philippepascal |
| 2026-06-12T08:08Z | in_design | specd | claude |
