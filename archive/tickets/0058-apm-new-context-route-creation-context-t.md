+++
id = 58
title = "apm new --context: route creation context to configured section"
state = "closed"
priority = 3
effort = 2
risk = 1
author = "claude-0329-1200-a1b2"
agent = "claude-0329-1430-main"
branch = "ticket/0058-apm-new-context-route-creation-context-t"
created_at = "2026-03-29T19:12:09.185811Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm new --context <text>` always places the context string into `### Problem`. This is reasonable as a default, but some projects use different section layouts — the `### Problem` name may differ, or agents may want to seed a different section (e.g. `### Open questions` or `### Approach`) for a side-ticket workflow.

There is no way to override the target section without editing the ticket after creation. A `--context-section` flag and a configurable default in `apm.toml` would let each project control where seeded context lands.

### Acceptance criteria

- [x] `apm new --context-section <name>` routes `--context` text to `### <name>` instead of `### Problem`
- [x] If `--context-section` is not provided, the target section defaults to the first entry in `tickets.sections` from `apm.toml`; if `tickets.sections` is absent or empty, the default is `Problem` (preserving current behaviour)
- [x] If the specified section does not exist in the ticket body template, `apm new` returns an error
- [x] `--context-section` without `--context` is an error
- [x] The `tickets.sections` config field is optional; omitting it preserves all existing behaviour
- [x] Unit test: `--context-section Approach` places text under `### Approach` in the created ticket body

### Out of scope

- Changing any other behaviour of `apm new`
- Adding section definitions to ticket parsing or validation
- Validating section names against a fixed allow-list

### Approach

1. Add `sections: Vec<String>` (default empty) to `TicketsConfig` in `apm-core/src/config.rs`.

2. Add `--context-section <name>` argument to the `New` command in `apm/src/main.rs`.

3. In `apm/src/cmd/new.rs`, resolve the target section:
   - If `--context-section` is provided, use that value.
   - Else if `config.tickets.sections` is non-empty, use the first entry.
   - Else use `"Problem"`.

4. When constructing the body template, replace the hardcoded `### Problem\n\n{ctx}\n\n` lookup with a search-and-replace on `### <section>` in the template string. Bail if the section heading is not found.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T19:12Z | — | new | claude-0329-1200-a1b2 |
| 2026-03-29T22:57Z | new | in_design | claude-spec-58 |
| 2026-03-29T23:09Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:15Z | specd | ready | apm |
| 2026-03-29T23:36Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:41Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |