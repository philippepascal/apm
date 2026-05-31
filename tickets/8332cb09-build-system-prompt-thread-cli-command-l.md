+++
id = "8332cb09"
title = "build_system_prompt: thread CLI command list into Layer 3 of worker prompt"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8332cb09-build-system-prompt-thread-cli-command-l"
created_at = "2026-05-31T02:58:36.745209Z"
updated_at = "2026-05-31T02:58:36.745209Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["9c66e199"]
+++

## Spec

### Problem

STEP 8 of the incremental workflow schema cleanup. Bug fix surfaced during earlier review.

PROBLEM: apm-core/src/start.rs::build_system_prompt at line ~974 calls instructions::generate(root, Some(role), ticket_id, &[]) — passing an EMPTY commands slice. The CLI path (apm/src/cmd/instructions.rs) extracts the clap command list and passes it in. The worker-spawn path does not. Result: every worker dispatched via apm start / apm work / UI dispatcher receives a system prompt whose Layer 3 (apm instructions) is missing the Command Reference section entirely.

EVIDENCE: 'apm prompt <id> --system' on current main shows Layer 3 ending at Session Identity. The standalone 'apm instructions <id> --role coder' shows the full Command Reference. The diff is the missing section.

CONSTRAINT: apm-core cannot depend on clap. The command list must come from outside.

DESIGN OPTIONS for the spec-writer to choose:

(A) Thread the command list through. build_system_prompt grows a parameter commands: &[(String, String)]. Every caller passes the clap-extracted list. apm-server constructs an empty list or a hard-coded one.

(B) Hard-coded constant in apm-core. After 9c66e199 unifies the command list to six commands, apm-core can know them by name without external input. Define a static const SHARED_COMMANDS in apm-core, use it in build_system_prompt. The CLI keeps using clap introspection for its own --help output, but build_system_prompt does not depend on clap.

(C) Function pointer / registration callback. Overkill.

Recommendation: (B). After 9c66e199, the worker command list is a fixed set of six commands. apm-core can carry the names + descriptions as a static const and the CLI does not need to pass them. This removes the apm-core / clap split concern entirely.

SCOPE:

1. apm-core/src/instructions.rs: add a static const that holds the six worker commands with descriptions (name, one-line about). 

2. apm-core/src/start.rs::build_system_prompt: replace &[] with that const at the instructions::generate call site.

3. Consider also updating the CLI path to use the same const for consistency. The CLI currently extracts the clap subcommand list to render the FULL apm command reference. After 9c66e199, the role-filtered output is just the six worker commands. The CLI extraction can be kept for the no-role case (which lists every apm command in the role index? actually no, no-role prints the role index, not the command reference). Verify the CLI flow after 9c66e199.

OUT OF SCOPE:
- Schema changes.
- Per-role allow-list (unified in 9c66e199; this ticket assumes that has landed).
- Help text sweep.

TESTS:
- A worker dispatched against a real ticket sees Layer 3 with a populated Command Reference. Diff apm prompt <id> --system against apm instructions <id> --role coder; the Command Reference content should appear in both.
- apm-server endpoints that call build_system_prompt produce a coherent prompt.

REFERENCES:
- apm-core/src/start.rs::build_system_prompt
- apm/src/cmd/instructions.rs::run (the clap extraction)
- 9c66e199 (this epic) for the unified command list

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
| 2026-05-31T02:58Z | — | new | philippepascal |
