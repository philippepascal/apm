+++
id = "93ff1402"
title = "apm set <> depends_on <t> does not auto complete <t> if the user puts 4 characters"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/93ff1402-apm-set-depends-on-t-does-not-auto-compl"
created_at = "2026-06-11T05:28:47.866310Z"
updated_at = "2026-06-12T08:17:16.464529Z"
+++

## Spec

### Problem

When the user runs `apm set <id> depends_on <prefix>` with a short prefix (e.g. `93ff`) instead of the full 8-character hex ID, the prefix is stored verbatim in the `depends_on` field of the ticket frontmatter. This breaks downstream: `check_depends_on_rules` does an exact-match lookup (`t.frontmatter.id == *dep_id`) and returns "dep not found", and even if that check were skipped, the invalid short ID would be written into the ticket and silently ignored by every command that reads `depends_on` (blocking-dep checks, `apm next` ordering, dependency context bundles).

The first positional argument to `apm set` is already resolved through `resolve_id_in_slice`, which handles 4-char prefixes, plain integers, and full 8-char IDs. The dependency IDs in the value argument receive no equivalent treatment — they are split on commas and used verbatim. Adding the same prefix-resolution step to each dep ID before validation and storage fixes the inconsistency and matches user expectations.

### Acceptance criteria

- [ ] `apm set <id> depends_on <4-char-prefix>` succeeds and stores the full 8-char ID in the `depends_on` frontmatter field when the prefix uniquely matches an existing ticket
- [ ] `apm set <id> depends_on <ambiguous-prefix>` fails with an "ambiguous prefix" error when the prefix matches more than one ticket
- [ ] `apm set <id> depends_on <unknown-prefix>` fails with a "no ticket matches" error when the prefix matches no ticket
- [ ] `apm set <id> depends_on <full-8-char-id>` continues to behave exactly as before
- [ ] `apm set <id> depends_on <a>,<b>` resolves each comma-separated value independently; all must resolve successfully before any change is written
- [ ] `apm set <id> depends_on -` (clear) is unaffected by the change

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
| 2026-06-11T05:28Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T08:17Z | groomed | in_design | philippepascal |