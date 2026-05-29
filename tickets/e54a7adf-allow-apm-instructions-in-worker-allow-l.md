+++
id = "e54a7adf"
title = "Allow apm instructions in worker allow-list and stop denial scanner mislabeling approvals/cancellations"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e54a7adf-allow-apm-instructions-in-worker-allow-l"
created_at = "2026-05-29T18:47:07.214401Z"
updated_at = "2026-05-29T18:49:32.162940Z"
+++

## Spec

### Problem

Two fixes exposed by a worker (syn ticket 25673007) that died in_progress. Root cause investigation (from the real .apm-worker.log, not the scanner labels): the worker ran 'apm instructions', which is NOT in the worker allow-list, so Claude Code returned 'This command requires approval'. The worker had batched it in parallel with other tool calls, and Claude cancels the entire parallel batch when one member needs approval — producing 7x '<tool_use_error>Cancelled: parallel tool call Bash(apm instructions) errored</tool_use_error>'. The allow-list itself works fine (other workers succeed); the only un-approvable command was 'apm instructions'.

PART A — add 'apm instructions' to the default allow-list. In apm/src/cmd/init.rs, APM_ALLOW_ENTRIES (and APM_USER_ALLOW_ENTRIES) list the apm subcommands but NOT 'apm instructions', even though it is the mandated first startup command. Add an entry that matches BOTH the arg-less form ('apm instructions') and any args. Note the matching subtlety: a pattern like 'Bash(apm instructions *)' (trailing space+star) does NOT match the arg-less 'apm instructions' that workers actually run — syn's settings.local.json has exactly that broken pattern. Use a form that matches the bare command (e.g. 'Bash(apm instructions*)', mirroring how 'Bash(apm sync*)' etc are written without the space). Update the existing init integration tests that assert allow-list contents (apm/tests/integration.rs around the Edit/Write/Read assertions) to also assert 'apm instructions' is present. NOTE: the apm instructions OUTPUT is already baked into the worker system prompt by build_system_prompt, so running it is redundant for workers — but agents do run it out of habit, so it must be allow-listed regardless. (A separate concern — telling workers to stop running it — is out of scope here.)

PART B — stop denial.rs misclassifying non-denials as denials. apm-core/src/denial.rs scans the transcript and classifies any is_error tool_result that does not start with 'Exit code N' as a permission denial (ApmCommandDenial / OutsideWorktree / UnknownPattern). This produced 9 'denials' for 25673007 of which ZERO were actual permission denials: 2 were 'requires approval' (a permission-config gap, arguably worth surfacing but distinct from a deny) and 7 were 'Cancelled: parallel tool call ... errored' (pure collateral cancellation, NOT a denial at all). This mislabeling sent diagnosis badly astray (an engineer concluded apm had a hook overriding the allow-list, which is false). Fix: the scanner must NOT count '<tool_use_error>Cancelled: parallel tool call' results as denials (they are cancellations, not denials). It should also distinguish 'requires approval' / 'This command requires approval' / 'requires approval:' as its own class (e.g. RequiresApproval / not-allowlisted) separate from genuine deny phrases ('cannot be auto-allowed', 'but you havent granted it yet', 'was blocked. For security'). Update denial.rs classification + its unit tests (the test module already has fixtures) so the summary.json accurately reflects what happened.

Both parts are in the apm repo (apm/src/cmd/init.rs, apm/tests/integration.rs, apm-core/src/denial.rs). No apm-server/apm-ui changes.

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
| 2026-05-29T18:47Z | — | new | philippepascal |
| 2026-05-29T18:49Z | new | groomed | philippepascal |
