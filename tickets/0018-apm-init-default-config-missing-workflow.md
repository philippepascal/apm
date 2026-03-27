+++
id = 18
title = "apm init default config missing workflow states"
state = "ready"
priority = 9
effort = 2
risk = 1
branch = "ticket/0018-apm-init-default-config-missing-workflow"
updated_at = "2026-03-27T03:16:55.012287Z"
+++

## Spec

### Problem

`apm init` generates `apm.toml` with no `[[workflow.states]]` entries. Once state
validation (#14) is implemented, every `apm state` call on a freshly-initialised
repo will fail: "unknown state — valid states: (empty)". The default config must
include the standard ticker workflow states so a new project works out of the box.

### Acceptance criteria

- [ ] `apm init`-generated `apm.toml` includes all standard `[[workflow.states]]` entries: `new`, `question`, `specd`, `ammend`, `ready`, `in_progress`, `implemented`, `accepted`, `closed`
- [ ] Each state has `id`, `label`, and `color` fields
- [ ] `closed` has `terminal = true`
- [ ] Generated config parses without error via `Config::load`
- [ ] Re-running `apm init` on an existing repo does not overwrite an existing `apm.toml`

### Out of scope

- Auto-transition entries in the default config
- Transition rules per state in the default config

### Approach

Extend `default_config()` in `cmd/init.rs` to include the full `[[workflow.states]]`
block matching the ticker workflow defined in `initial_specs/SPEC.md §6`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | manual | new → specd | |
| 2026-03-27T03:16Z | specd | ready | apm |
