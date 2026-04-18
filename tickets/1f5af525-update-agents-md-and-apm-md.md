+++
id = "1f5af525"
title = "update agents.md and apm.*.md"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1f5af525-update-agents-md-and-apm-md"
created_at = "2026-04-18T18:42:11.878614Z"
updated_at = "2026-04-18T19:26:59.614667Z"
+++

## Spec

### Problem

The three agent instruction files (`agents.md`, `apm.spec-writer.md`, `apm.worker.md`) exist in two locations: the live project copy under `.apm/` and the default templates under `apm-core/src/default/`. These files have drifted out of sync with each other and with the actual CLI behaviour.

Two concrete bugs affect agents following the default templates today:

1. `apm-core/src/default/apm.agents.md` instructs agents to run `git -C <path> add` and `git -C <path> commit` after every `apm spec --set` or `--mark` call. This is wrong: `apm spec` already calls `git::commit_to_branch` internally, so the manual commits would either fail (nothing to commit) or create spurious empty commits. The live `.apm/agents.md` is correct on this point.

2. `apm-core/src/default/apm.agents.md` instructs agents to take over a ticket with `apm take <id>`. The command `apm take` does not exist in the CLI (`apm --help` shows no such subcommand; `apm assign` is the correct command). The live `.apm/agents.md` correctly uses `apm assign <id> <username>`.

Additionally, `apm-core/src/default/apm.spec-writer.md` is missing the "How to save spec sections" block (explaining `--set` vs `--set-file`) and the explicit `apm state <id> specd` call at the end of "When you are done", both of which are present in the live `.apm/apm.spec-writer.md`.

`apm.worker.md` is identical in both locations and requires no changes.

### Acceptance criteria

- [x] `apm-core/src/default/apm.agents.md` groomed-state workflow contains no manual `git add` or `git commit` steps after `apm spec` calls
- [x] `apm-core/src/default/apm.agents.md` ammend-state workflow contains no manual `git add` or `git commit` steps after `apm spec` calls
- [x] `apm-core/src/default/apm.agents.md` uses `apm assign <id> <username>` (not `apm take`) for ticket takeover
- [x] `apm-core/src/default/apm.agents.md` "Taking over another agent's ticket" section uses `apm assign <id> <username>`
- [x] `apm-core/src/default/apm.agents.md` in_design state description uses `apm assign` instead of `apm take`
- [x] `apm-core/src/default/apm.spec-writer.md` contains a "How to save spec sections" block explaining `--set` and `--set-file` usage
- [x] `apm-core/src/default/apm.spec-writer.md` "When you are done" section includes `apm state <id> specd` as the final step

### Out of scope

- Changes to .apm/agents.md or .apm/apm.spec-writer.md (live project copies are already correct)\n- Changes to apm.worker.md (identical in both locations, no discrepancies)\n- Adding new documentation sections not already present in .apm/ versions\n- Changing the Delegator vs Main Agent role design in agents.md

### Approach

All changes are text edits to two files only: `apm-core/src/default/apm.agents.md` and `apm-core/src/default/apm.spec-writer.md`.

**`apm-core/src/default/apm.agents.md` — three fixes:**

1. **groomed state, step 3** (lines 99–109): Remove the trailing `git -C <printed-path> add` / `git -C <printed-path> commit` block. Replace the comment above the `apm spec` examples with the parenthetical already used in `.apm/agents.md`: "each `--set` auto-commits to the ticket branch; no manual `git add`/`git commit` needed". Keep the `apm new --no-edit` note in step 3.

2. **ammend state, step 3** (lines 125–130): Remove the trailing `git -C <printed-path> add` / `git -C <printed-path> commit` block. The `apm spec --mark` call already commits; no manual step needed.

3. **`apm take` → `apm assign`** — two occurrences:
   - in_design state description (line 136): change `apm take <id>` to `apm assign <id> <your-username>`
   - "Taking over another agent's ticket", step 2 (line 175): change `apm take <id>` to `apm assign <id> <your-username>` and update the description from "sets agent = your name on the ticket branch" to "reassign ownership to yourself"

**`apm-core/src/default/apm.spec-writer.md` — two fixes:**

1. **Add "How to save spec sections" block** after the opening paragraph (before `## When you are done`). Copy verbatim from `.apm/apm.spec-writer.md` lines 9–24:
   ```
   ## How to save spec sections
   
   Use `apm spec` to write each section. For long content, write to a temp file
   first with the Write tool, then reference it with `--set-file`:
   ...
   Do NOT write the ticket markdown file directly. Always use `apm spec`.
   ```

2. **"When you are done" section**: Add `Then: \`apm state <id> specd\`` as the final line of the section (after the `apm set <id> risk` line), matching `.apm/apm.spec-writer.md` line 37.

No Rust code changes. No test changes needed (these are documentation-only files). The `.apm/` copies are already correct and must not be touched.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T18:42Z | — | new | philippepascal |
| 2026-04-18T18:42Z | new | groomed | philippepascal |
| 2026-04-18T18:48Z | groomed | in_design | philippepascal |
| 2026-04-18T18:52Z | in_design | specd | claude-0418-1848-54d0 |
| 2026-04-18T18:58Z | specd | ready | philippepascal |
| 2026-04-18T18:59Z | ready | in_progress | philippepascal |
| 2026-04-18T19:01Z | in_progress | implemented | claude-0418-1859-9748 |
| 2026-04-18T19:26Z | implemented | closed | philippepascal |
