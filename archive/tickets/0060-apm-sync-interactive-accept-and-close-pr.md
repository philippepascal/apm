+++
id = 60
title = "apm sync: interactive accept and close prompts after merge detection"
state = "closed"
priority = 1
effort = 3
risk = 2
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1430-main"
branch = "ticket/0060-apm-sync-interactive-accept-and-close-pr"
created_at = "2026-03-29T19:12:24.587299Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

When `apm sync` detects a merged ticket branch (state `implemented`, branch merged into main), it only prints:

```
#N: branch merged — run `apm state N accepted` to accept
```

The supervisor then has to manually copy-paste and run that command for each ticket. For the common case of accepting all merged tickets at once, this is unnecessary friction.

The batch-close prompt already handles the close step interactively (`prompt_close`). The accept step should be symmetric: offer an interactive prompt to accept each merged ticket immediately, gated on `std::io::IsTerminal` so the prompt is suppressed in non-interactive (script/CI) contexts.

### Acceptance criteria

- [x] When `apm sync` detects one or more merged-but-not-accepted tickets and stdout is a terminal, it prints each one and asks the supervisor to accept them (individually or in batch)
- [x] The prompt is suppressed (reverts to the current print-only behaviour) when stdout is not a terminal
- [x] `--quiet` suppresses the accept prompt (same as it suppresses the close prompt)
- [x] An `--auto-accept` flag (mirrors `--auto-close`) accepts all eligible tickets without prompting
- [x] Accepting a ticket via the sync prompt is equivalent to running `apm state <id> accepted` — it commits the state transition to the ticket branch
- [x] If the `accepted → closed` transition does not exist in the workflow, the ticket is accepted but not auto-closed (the close step remains separate)
- [x] Integration test: after a sync that detects a merged ticket, simulating `y` at the accept prompt results in the ticket being in `accepted` state

### Out of scope

- Accepting tickets that are not in `implemented` state
- Changing the close prompt or `--auto-close` behaviour
- Per-ticket accept prompts (batch only, like the existing close prompt)
- Removing the print message for non-interactive contexts

### Approach

In `apm/src/cmd/sync.rs`:

1. Collect merged-but-not-accepted tickets into an `AcceptCandidate` struct (similar to `CloseCandidate`) alongside the existing merged-branch detection loop.

2. Add `is_interactive() -> bool` using `std::io::IsTerminal` on stdout.

3. Add `prompt_accept(candidates: &[AcceptCandidate]) -> Result<bool>` modelled on `prompt_close`.

4. Add `--auto-accept` flag to `run` signature and the CLI definition in `main.rs`.

5. After the merged-branch loop, if candidates are non-empty: call `super::state::run` for each accepted ticket when confirmed (either via `--auto-accept` or the interactive prompt).

6. Gate the prompt on `!quiet && is_interactive()`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:57Z | new | in_design | claude-spec-60 |
| 2026-03-29T23:09Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:17Z | specd | ready | apm |
| 2026-03-29T23:37Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:41Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |