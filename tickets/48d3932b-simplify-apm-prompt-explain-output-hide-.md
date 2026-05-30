+++
id = "48d3932b"
title = "Simplify apm prompt --explain output: hide cascade detail when no fallback fired"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/48d3932b-simplify-apm-prompt-explain-output-hide-"
created_at = "2026-05-30T07:40:46.558546Z"
updated_at = "2026-05-30T17:13:20.695474Z"
+++

## Spec

### Problem

PROBLEM: the current apm prompt --explain output is confusing in three ways:
1. 'layer' and 'level' are presented as sibling concepts (level appears at the same indent as layer) when in fact the cascade levels are nested inside layer 3.
2. A 'skipped:' section appears at the top level alongside the layers, suggesting it is a fourth thing instead of a sub-detail of layer 3.
3. For the most common case (agent is claude AND the agent-specific role file exists), the output still prints two skipped cascade lines that add noise without information. The user has to mentally decode why 'not reached' was reported for entries that were structurally never going to apply.

GOAL: collapse the output to the minimum that communicates what was used and, when something fell back, why. Show the cascade detail only when a fallback actually fired. When the agent has its own role file on disk, print only the file path on layer 3 with no cascade explanation.

DESIRED OUTPUT (three cases the formatter must handle):

CASE 1 — natural resolution. agent=claude with .apm/agents/claude/apm.coder.md present, OR any agent with its own per-agent role file present:

  System prompt for claude/coder — 3 layers composed:

    1  apm instructions (dynamic)
    2  .apm/project.md
    3  .apm/agents/claude/apm.coder.md

No cascade block. No 'skipped' section. The role line on layer 3 is the chosen path; no parenthetical.

CASE 2 — one fallback fired. The agent's own role file is missing, but the claude default role file exists:

  System prompt for phi4/coder — 3 layers composed:

    1  apm instructions (dynamic)
    2  .apm/project.md
    3  .apm/agents/claude/apm.coder.md
       (fallback — .apm/agents/phi4/apm.coder.md not found)

A single sub-line under the layer-3 path names the missing agent-specific path that triggered the fallback.

CASE 3 — both on-disk candidates missing. The cascade falls through to the binary's built-in default. Both missing paths are listed in the fallback note:

  System prompt for my-bot/coder — 3 layers composed:

    1  apm instructions (dynamic)
    2  .apm/project.md
    3  built-in claude/coder default
       (fallback — .apm/agents/my-bot/apm.coder.md not found,
                   .apm/agents/claude/apm.coder.md not found)

Both missing on-disk paths are listed on the sub-line, separated by a comma and aligned visually so the chain is readable.

RULE (made explicit for the spec-writer):

| Resolution | Layer 3 line | Fallback sub-line |
|---|---|---|
| Agent-specific file exists | Path to that file | None |
| Fell back once (to claude default) | Path to claude default | (fallback — <agent-specific path> not found) |
| Fell back twice (to bundled default) | 'built-in <agent>/<role> default' | (fallback — <agent-specific path> not found, <claude default path> not found) |

For agent=claude, the claude-default level is structurally the same path as the agent-specific level, so it can never appear as a fallback step. Case 1 covers it cleanly with no special-case logic in the formatter — the agent-specific candidate is also the only on-disk candidate, so success there suppresses any cascade block.

SCOPE: changes are limited to the output formatter in apm-core/src/prompt.rs and the PromptProvenance display path it consumes. The cascade resolution logic in apm-core/src/start.rs (the function that picks the winner and reports the skipped entries) is unchanged. The data structure may need to evolve so the formatter can distinguish 'agent-specific did not exist' from 'cascade did not reach this level because something higher won', but no behavior of the cascade changes.

OUT OF SCOPE:
- Changing layer 1 or layer 2 output (only layer 3 has a cascade; the simplification applies only there).
- Removing the layer 2 line when .apm/project.md is unset (separate UX decision, not asked).
- Changing the cascade resolution order or adding new cascade levels.
- Changing JSON output of apm prompt (if any exists for --explain) — this is purely the human-readable text output.
- apm-server / apm-ui surfacing — the prompt explanation is a CLI concern.
- Translations / colorization / TTY-detection beyond the existing baseline.

TESTS:
- A unit test for each of the three cases above, asserting the exact text output (or at minimum the presence of the expected lines and absence of the cascade block in case 1).
- A test that for agent=claude with the agent-specific file present, the word 'skipped' and the word 'cascade' do not appear in the output (regression guard for the noise that motivated this ticket).
- A test that in case 3 both missing paths are listed in the fallback note, with the second indented to align under '— ' so the rendered chain is readable.

EXISTING TESTS: a small handful in prompt.rs assert specific phrases in the output (per-agent path appears, built-in default appears, claude/coder is named in the bundled-default case). Those tests need to be updated to match the new wording or replaced with the new-shape assertions. The new wording must still let those tests express their original intent (the chosen source is visible).

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
| 2026-05-30T07:40Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:13Z | groomed | in_design | philippepascal |
