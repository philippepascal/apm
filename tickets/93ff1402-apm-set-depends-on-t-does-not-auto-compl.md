+++
id = "93ff1402"
title = "apm set <> depends_on <t> does not auto complete <t> if the user puts 4 characters"
state = "implemented"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/93ff1402-apm-set-depends-on-t-does-not-auto-compl"
created_at = "2026-06-11T05:28:47.866310Z"
updated_at = "2026-06-12T23:25:13.945176Z"
+++

## Spec

### Problem

When the user runs `apm set <id> depends_on <prefix>` with a short prefix (e.g. `93ff`) instead of the full 8-character hex ID, the prefix is stored verbatim in the `depends_on` field of the ticket frontmatter. This breaks downstream: `check_depends_on_rules` does an exact-match lookup (`t.frontmatter.id == *dep_id`) and returns "dep not found", and even if that check were skipped, the invalid short ID would be written into the ticket and silently ignored by every command that reads `depends_on` (blocking-dep checks, `apm next` ordering, dependency context bundles).

The first positional argument to `apm set` is already resolved through `resolve_id_in_slice`, which handles 4-char prefixes, plain integers, and full 8-char IDs. The dependency IDs in the value argument receive no equivalent treatment — they are split on commas and used verbatim. Adding the same prefix-resolution step to each dep ID before validation and storage fixes the inconsistency and matches user expectations.

### Acceptance criteria

- [x] `apm set <id> depends_on <4-char-prefix>` succeeds and stores the full 8-char ID in the `depends_on` frontmatter field when the prefix uniquely matches an existing ticket
- [x] `apm set <id> depends_on <ambiguous-prefix>` fails with an "ambiguous prefix" error when the prefix matches more than one ticket
- [x] `apm set <id> depends_on <unknown-prefix>` fails with a "no ticket matches" error when the prefix matches no ticket
- [x] `apm set <id> depends_on <full-8-char-id>` continues to behave exactly as before
- [x] `apm set <id> depends_on <a>,<b>` resolves each comma-separated value independently; all must resolve successfully before any change is written
- [x] `apm set <id> depends_on -` (clear) is unaffected by the change

### Out of scope

- `apm new --depends-on <prefix>` has the same gap but is a separate ticket
- Shell tab completion for ticket IDs (this ticket is about ID resolution, not shell autocompletion)
- Prefix resolution for the `depends_on` field in other commands (e.g. `apm validate`, `apm start`)

### Approach

All changes are in `apm/src/cmd/set.rs`. No `apm-core` changes are needed — `ticket_fmt::resolve_id_in_slice` is already callable via `ticket::resolve_id_in_slice` (the same function used to resolve the ticket ID argument on line 9).

In the `if field == "depends_on" && value != "-"` block (lines 12–31):

1. After splitting `value` by comma into `ids`, resolve each entry by calling `ticket::resolve_id_in_slice(&tickets, dep_id)`. Collect the results into `resolved_ids: Vec<String>`, propagating any "no ticket matches" or "ambiguous prefix" error immediately.

2. Reconstruct a canonical value string: `let resolved_value = resolved_ids.join(",");`.

3. Pass `&resolved_ids` (not the original `ids`) to `check_depends_on_rules`.

4. Replace the `value` variable (or shadow it) with `resolved_value` so that `set_field`, the commit message, and the stdout `println!` all use the resolved IDs.

The `git::commit_to_branch` call and `println!` on lines 60–74 already reference `value` by reference; shadowing `value` with `resolved_value` before that block keeps the diff minimal.

Add a unit test in `apm/tests/integration.rs` that:
- Creates two tickets in a temp repo
- Runs `apm set <id> depends_on <4-char-prefix>` where the prefix uniquely identifies the second ticket
- Asserts the stored frontmatter contains the full 8-char ID
- Runs the same with an ambiguous prefix and asserts a non-zero exit code with "ambiguous" in stderr

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-11T05:28Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:17Z | groomed | in_design | philippepascal |
| 2026-06-12T08:20Z | in_design | specd | claude |
| 2026-06-12T22:53Z | specd | ready | philippepascal |
| 2026-06-12T23:16Z | ready | in_progress | philippepascal |
| 2026-06-12T23:25Z | in_progress | implemented | claude |
