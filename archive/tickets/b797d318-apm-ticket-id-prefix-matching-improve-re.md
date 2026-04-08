+++
id = "b797d318"
title = "apm ticket ID prefix matching: improve resolution and error messages"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "3729"
branch = "ticket/b797d318-apm-ticket-id-prefix-matching-improve-re"
created_at = "2026-03-30T16:56:24.264985Z"
updated_at = "2026-03-30T19:54:59.896223Z"
+++

## Spec

### Problem

When resolving a ticket ID from a short prefix, APM has two bugs:

1. **Unique prefix not resolved**: `apm review 314` fails with "no ticket matches '0314'" even when exactly one ticket has an ID starting with `314`. The prefix is unique — there is no ambiguity — but APM rejects it instead of resolving it.

2. **Ambiguous prefix error is unhelpful**: When multiple tickets share the same prefix, the error message does not list the candidates. The user has no way to disambiguate without running `apm list` separately and scanning manually.

The correct behaviour:
- Unique prefix → resolve silently (already works for longer prefixes like `3142`, broken for shorter ones like `314`)
- Ambiguous prefix → list all matching ticket IDs and titles, ask the user to be more specific

### Acceptance criteria

- [x] `apm show 314` resolves a ticket whose ID starts with `314` (e.g. `314abcde`) when that is the only match
- [x] `apm show 1` still resolves ticket `0001` (integer zero-padding kept for backward compat)
- [x] `apm show 3142` still resolves a ticket whose ID starts with `3142` (4-digit input, no regression)
- [x] When multiple tickets share a short prefix, the error message lists each matching ID and title
- [x] When no ticket matches the supplied prefix, the error message includes the prefix that was tried
- [x] All of the above apply equally to commands using `resolve_ticket_branch` (e.g. `apm show`, `apm spec`)

### Out of scope

- Fuzzy or edit-distance matching
- Changes to the ticket ID format itself
- Handling of hex-only inputs (4–8 char strings with non-digit hex chars) — those already work correctly
- New CLI flags or interactive disambiguation prompts

### Approach

**Root cause of Bug 1**

`normalize_id_arg` converts any all-digit input to a zero-padded 4-char string.
For inputs shorter than 4 digits (e.g. `314`), this produces `0314`, which does
not match a hex ticket ID like `314abcde`. Four-digit inputs happen to be
unchanged by padding (e.g. `3142` → `3142`), which is why they already work.

**Root cause of Bug 2**

`resolve_id_in_slice` already formats a candidate list in its ambiguous-prefix
error. `resolve_ticket_branch` also does this (with branch names). If this
behaviour is absent in the version being implemented, add it; if present,
confirm it is not regressed by the Bug 1 fix.

**Fix**

Add a helper `id_arg_prefixes(arg: &str) -> Result<Vec<String>>` in
`apm-core/src/ticket.rs`:

- Calls `normalize_id_arg(arg)` for validation and the canonical prefix.
- For all-digit inputs of length 1–3, also includes the raw digit string as a
  second candidate (since digits are valid hex and the raw string is the correct
  hex prefix). Example: `"314"` → `["0314", "314"]`.
- For all-digit inputs of length ≥ 4, the zero-padded form equals the raw form,
  so only one prefix is returned. Example: `"3142"` → `["3142"]`.
- For hex-only inputs (letters present), delegates entirely to `normalize_id_arg`
  and returns a single-element vec. Example: `"a3f9"` → `["a3f9"]`.

Update `resolve_id_in_slice` (`apm-core/src/ticket.rs`):

- Replace the single `prefix` string with the `Vec<String>` from `id_arg_prefixes`.
- Filter tickets where `id.starts_with` any of the candidate prefixes.
- Deduplicate matches by ticket ID (a ticket could theoretically satisfy both
  `0314` and `314` if its ID were `03140000`, though that is contrived).
- Error messages: keep the existing format ("no ticket matches …" / ambiguous
  list with ID and title).

Update `resolve_ticket_branch` (`apm-core/src/git.rs`) in the same way.

Update the `close()` function in `apm-core/src/ticket.rs`, which currently calls
`normalize_id_arg` directly — change it to use `id_arg_prefixes` and match
against all candidates.

**Tests to add** (inline in `apm-core/src/ticket.rs`):

- `id_arg_prefixes("314")` returns two prefixes: `"0314"` and `"314"`.
- `id_arg_prefixes("3142")` returns one prefix: `"3142"`.
- `id_arg_prefixes("a3f9")` returns one prefix: `"a3f9"`.
- `resolve_id_in_slice` with one ticket ID `314abcde` and input `"314"` resolves correctly.
- `resolve_id_in_slice` with ticket `0001` and input `"1"` still resolves correctly.
- `resolve_id_in_slice` with two tickets `314abcde` and `3142xxxx` and input `"314"` returns ambiguous error listing both.

**Files changed**

- `apm-core/src/ticket.rs` — add `id_arg_prefixes`, update `resolve_id_in_slice` and `close`
- `apm-core/src/git.rs` — update `resolve_ticket_branch`

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T16:56Z | — | new | philippepascal |
| 2026-03-30T17:20Z | new | in_design | philippepascal |
| 2026-03-30T17:25Z | in_design | specd | claude-0330-1720-spec1 |
| 2026-03-30T19:18Z | specd | ready | apm |
| 2026-03-30T19:24Z | ready | in_progress | philippepascal |
| 2026-03-30T19:28Z | in_progress | implemented | claude-0330-1930-w001 |
| 2026-03-30T19:48Z | implemented | accepted | apm-sync |
| 2026-03-30T19:54Z | accepted | closed | apm-sync |