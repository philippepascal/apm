+++
id = "b906d63f"
title = "Verify apm prompt --help text is current after epic completes"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b906d63f-verify-apm-prompt-help-text-is-current-a"
created_at = "2026-05-31T02:11:27.601887Z"
updated_at = "2026-05-31T03:04:09.610752Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["7e66181a", "56500644", "68829abb", "d2a947ea"]
+++

## Spec

### Problem

Verification ticket. After d2a947ea (CLI help audit) lands as part of this epic, double-check that apm prompt --help reflects the current behaviour.

KNOWN STALENESS to verify is gone:

1. The long_about describes layer order as 'Layer 1 — APM system knowledge ... Layer 3 — Role instructions'. This is the OLD order; ticket 9ea43165 reversed it (layer 1 is now the role file, layer 3 is apm instructions). After d2a947ea, this should be corrected.

2. The --explain example block shows the OLD verbose provenance format:

     layer 1:        apm instructions (dynamic, role: worker)
     layer 2:        .apm/project.md
     layer 3:        .apm/agents/claude/apm.worker.md  (level 1 — claude-fallback file)
     skipped:        level 0 (.apm/agents/myagent/apm.worker.md — not reached)
     level 2 (built-in default — not reached)

   Tickets 48d3932b and 9ea43165 replaced this with the cleaner 'System prompt for <agent>/<role> — 3 layers composed:' format. The example block in --help must match.

3. The long_about references 'shell discipline' as part of Layer 1 content. Ticket a3c34ddc moved shell discipline into role files (now Layer 1 = role file). The reference is stale either way.

WHAT TO DO:
- Run the local apm binary's 'apm prompt --help' after d2a947ea has landed and verify the layer order labels, the --explain example, and the layer descriptions are current.
- If d2a947ea did its job, this ticket can be closed with no code change.
- If d2a947ea missed something, fix it here.

The reason this is a separate ticket: d2a947ea is broad-scoped (every help string in apm). It is easy to miss the multi-paragraph long_about of apm prompt specifically. This ticket forces a targeted verification.

OUT OF SCOPE:
- Any code behaviour changes; this is purely doc verification.

REFERENCES:
- apm/src/main.rs or apm/src/cmd/prompt.rs (wherever the long_about lives)
- Tickets 48d3932b, 9ea43165, a3c34ddc for the changes that made the current text stale
- d2a947ea (this epic) for the bulk help-text audit

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
