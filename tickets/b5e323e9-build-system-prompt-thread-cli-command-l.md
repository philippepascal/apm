+++
id = "b5e323e9"
title = "build_system_prompt: thread CLI command list into worker prompt Layer 3"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b5e323e9-build-system-prompt-thread-cli-command-l"
created_at = "2026-05-31T02:11:09.312521Z"
updated_at = "2026-05-31T03:04:06.894491Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["7e66181a", "56500644", "68829abb", "d2a947ea"]
+++

## Spec

### Problem

Regression to fix after the workflow schema migration completes.

PROBLEM: apm-core/src/start.rs::build_system_prompt at line 974 (current main) calls instructions::generate(root, Some(role), ticket_id, &[]) — passing an EMPTY commands slice. The CLI path apm/src/cmd/instructions.rs::run extracts the command list from clap's introspection and passes it to generate. The worker-spawn path does not. As a result, every worker dispatched via apm start / apm work / UI dispatcher receives a system prompt whose Layer 3 (apm instructions) is missing the Command Reference section. Workers in production never see the apm command list.

EVIDENCE: running 'apm prompt 36b6f742 --system' against current main shows Layer 3 ends at the Session Identity section. Running 'apm instructions 36b6f742 --role coder' standalone shows the full Command Reference at the bottom. The diff is the missing section.

CONSTRAINT: apm-core cannot depend on clap (deliberate separation; apm-core has no CLI framework dependency). The command list must come from outside.

DESIGN OPTIONS for spec-writer to choose:

(A) Thread the command list through. build_system_prompt grows a parameter commands: &[(String, String)]. Every caller in apm/src/cmd/start.rs and the spawn path passes the clap-extracted list. apm-server uses an empty list (or constructs one from a hard-coded source).

(B) Static command list in apm-core. Define a const array of (name, description) pairs maintained in apm-core. The CLI and apm-core stay in sync via convention. Simpler signature, more risk of drift.

(C) Late-binding callback. apm-core exposes a fn pointer or trait; the CLI registers it at startup. More machinery; only worth it if (A) is unwieldy.

Spec-writer to choose. Default recommendation: (A). It mirrors how the CLI already calls generate; the worker path just needs the same data flow.

OUT OF SCOPE:
- Anything outside the build_system_prompt → instructions::generate path.
- Reformatting the Command Reference itself.
- Per-role filtering of the Command Reference (covered separately).

TESTS:
- A worker dispatched against a real ticket sees Layer 3 with a populated Command Reference. Test by comparing apm prompt <id> --system to apm instructions <id> --role coder; the Command Reference content should appear in both, identical or near-identical.
- apm-server endpoints that call build_system_prompt do not crash and produce a coherent prompt.

DEPENDS_ON RATIONALE: depends on all four migration leaves so the build_system_prompt code path is settled before this regression is addressed. The signature change interacts with 7e66181a (instructions filter) and 1a13dee7 (dispatch). Best to fix this last.

REFERENCES:
- apm-core/src/start.rs::build_system_prompt (line ~950)
- apm/src/cmd/instructions.rs::run (line ~5; the working pattern)
- apm/src/cmd/prompt.rs and any prompt-rendering path

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
| 2026-05-31T02:11Z | — | new | philippepascal |
| 2026-05-31T03:04Z | new | closed | philippepascal |
