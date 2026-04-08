+++
id = 64
title = "Wire ticket.sections config into apm new and apm spec"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0064-wire-ticket-sections-config-into-apm-new"
created_at = "2026-03-29T23:26:06.417584Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`[[ticket.sections]]` is fully parsed by `apm-core/src/config.rs` (`TicketSection` with `name`, `type`, `required`, `placeholder`) but nothing uses it at runtime.

**`apm new`** hardcodes the ticket body template — `### Problem`, `### Acceptance criteria`, `### Out of scope`, `### Approach` — regardless of what `[[ticket.sections]]` declares. The `placeholder` field is never shown. Projects with different section layouts get the wrong scaffold.

**`apm spec`** validates section names against a hardcoded `KNOWN_SECTIONS` constant that excludes "Amendment requests" and "Code review". It rejects those sections and hardcodes type behaviour (checkbox formatting, Q&A formatting) in match arms instead of reading from config.

### Acceptance criteria

- [x] `apm new` builds the ticket body by iterating `config.ticket.sections`; each section becomes `### <name>\n\n<placeholder or empty>\n\n`
- [x] If `config.ticket.sections` is empty, `apm new` falls back to the current hardcoded template (no regression for unconfigured repos)
- [x] `apm spec --section <name>` accepts any section name present in `config.ticket.sections`; unknown sections error with "not defined in [ticket.sections]"
- [x] `apm spec --section <name> --set <value>` for a `tasks`-type section wraps each non-checkbox line as `- [ ] <line>`
- [x] `apm spec --section <name> --set <value>` for a `qa`-type section wraps each line as `**Q:** <line>`
- [x] `apm spec --section <name> --set <value>` for a `free`-type section writes prose as-is (current behaviour)
- [x] If `config.ticket.sections` is empty, `apm spec` falls back to the current hardcoded KNOWN_SECTIONS behaviour
- [x] Unit test: body scaffold matches section definitions from a test config

### Out of scope

- Precondition enforcement based on `required` (already handled by `apm state` preconditions)
- Full round-trip of arbitrary sections not in `TicketDocument` struct (deeper refactor)
- Changing `apm spec --mark` (ticket #66)

### Approach

1. In `apm/src/cmd/new.rs`, after loading config, if `config.ticket.sections` is non-empty build the body by iterating sections: `### <name>\n\n<placeholder>\n\n`. Keep the existing hardcoded template as fallback.

2. In `apm/src/cmd/spec.rs`, replace `KNOWN_SECTIONS` with a lookup from `config.ticket.sections` when non-empty. Replace hardcoded match arms in `set_section` with a config-driven dispatch: `Tasks` → auto-checkbox, `Qa` → `**Q:**` prefix, `Free` → as-is.

3. For sections not in `TicketDocument` named fields, fall back to raw body manipulation: find `### <name>` heading, replace content up to the next `##` heading.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:26Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:26Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:31Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:43Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:04Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T00:50Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |