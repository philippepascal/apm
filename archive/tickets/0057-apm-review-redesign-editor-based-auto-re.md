+++
id = 57
title = "apm review redesign: editor-based, auto-resolves transitions"
state = "closed"
priority = 1
effort = 4
risk = 2
author = "claude-0329-1200-a1b2"
branch = "ticket/0057-apm-review-redesign-editor-based-auto-re"
created_at = "2026-03-29T19:12:01.851314Z"
updated_at = "2026-03-30T02:50:15.105899Z"
+++

## Spec

### Problem

`apm review` currently opens an editor for spec editing, then — after the editor closes — prompts the supervisor interactively at stdin for the target transition. This two-phase interaction is inconvenient and breaks non-interactive contexts (piped input, automated tooling, agents calling `apm review` from a script).

The transition prompt is also fragile: if stdin is not a terminal, `read_line` silently returns an empty string, which `prompt_transition` interprets as "keep" and skips the transition entirely.

The fix is to make the workflow fully editor-based: embed a `# transition: ` marker in the review header so the supervisor can set the target state by editing a single line inside the file. After the editor closes, the chosen transition is extracted from the header — no secondary prompt needed.

### Acceptance criteria

- [ ] The review header written to the temp file includes a `# transition: ` line that the supervisor can edit (e.g. `# transition: ready`)
- [ ] When `--to <state>` is provided, the header pre-fills the line with that state; otherwise the line is blank (`# transition: `)
- [ ] After the editor closes, `extract_transition` reads the `# transition:` line from the content before the sentinel and returns the trimmed value (empty string → `None`)
- [ ] `prompt_transition` is removed; its stdin-reading logic is not reachable after this change
- [ ] If the extracted transition is invalid (not in the available transitions list), `run` returns an error instead of proceeding silently
- [ ] Transitions to all other mechanics (`--to` pre-validation, `state::run` call) are unchanged
- [ ] Integration test: after `review` with a `# transition: ammend` line saved in the temp file, the ticket ends up in `ammend` state without any stdin interaction

### Out of scope

- Changing the editor selection logic (`$VISUAL` / `$EDITOR`)
- Changing how `state::run` applies the transition
- Adding new transition types or states
- Handling multi-step or conditional transitions

### Approach

In `apm/src/cmd/review.rs`:

1. Update `build_header` to append a `# transition: <value>` line at the end of the header block. When `fixed_to` is `Some(s)`, the value is `s`; otherwise it is empty.

2. Add `extract_transition(content: &str) -> Option<String>`: scan lines before the sentinel for one starting with `# transition:`, strip the prefix, trim whitespace. Return `Some(state)` if non-empty, `None` otherwise.

3. In `run`, after calling `open_editor` and reading `edited_raw`:
   - Call `extract_transition(&edited_raw)` to get `chosen_state`
   - If `to` was provided via CLI, ignore the extracted value (or assert they match — pick the simpler path)
   - Validate `chosen_state` against `transitions` (same check currently done for `--to`); bail on invalid
   - Remove the call to `prompt_transition`

4. Delete `prompt_transition` and its helper code.

**Audited 2026-03-29:** Approach still valid. `prompt_transition` and `build_header` both exist in `apm/src/cmd/review.rs`. No `extract_transition` function present yet. All referenced file paths and function names are accurate.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:57Z | new | in_design | claude-spec-57 |
| 2026-03-29T23:09Z | in_design | specd | claude-0329-1430-main |
| 2026-03-30T02:50Z | specd | closed | apm |