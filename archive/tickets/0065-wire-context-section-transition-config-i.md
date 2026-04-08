+++
id = 65
title = "Wire context_section transition config into apm new"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0065-wire-context-section-transition-config-i"
created_at = "2026-03-29T23:26:09.699704Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm new --context <text>` always places the context string into `### Problem`. The target section is hardcoded in `new.rs`.

`TransitionConfig` already has a `context_section: Option<String>` field (parsed from `apm.toml`) intended to control exactly this — which section receives the `--context` value when a ticket is created. But the field is never read at runtime.

The lifecycle design calls for `apm new --context` to route text to the section named by `context_section` on the `new → in_design` transition, so the mapping is declared in `apm.toml` rather than hardcoded.

This is distinct from ticket #58, which adds a `--context-section` CLI override. This ticket wires the config-driven default so that even without a CLI flag, the right section is used.

### Acceptance criteria

- [x] When `apm new --context <text>` is run, `apm` looks up the `new → in_design` transition in `config.workflow` and reads its `context_section` field
- [x] If `context_section` is set, `--context` text is placed into that section; if absent, falls back to `"Problem"` (current behaviour preserved)
- [x] The `--context-section` CLI flag (ticket #58) takes precedence over the transition config value when both are present
- [x] If the resolved section name does not exist in the ticket body, `apm new` returns an error
- [x] Unit test: with `context_section = "Approach"` on the `new → in_design` transition, `--context` text lands in `### Approach`

### Out of scope

- Adding `context_section` support to transitions other than `new → in_design`
- Changing `apm spec`

### Approach

In `apm/src/cmd/new.rs`, after loading config, resolve the target section:

1. Find the `new → in_design` transition in `config.workflow.states` where `state.id == "new"`.
2. Read `tr.context_section` from that transition.
3. Priority: `--context-section` CLI arg > `tr.context_section` > `"Problem"`.
4. Use the resolved name when inserting context text into the body template (find `### <section>` heading, insert after it).

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:26Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:26Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:31Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:44Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:00Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T00:50Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |