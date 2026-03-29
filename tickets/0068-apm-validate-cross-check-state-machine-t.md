+++
id = 68
title = "apm validate: cross-check state machine, ticket sections, and agent instructions"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0068-apm-validate-cross-check-state-machine-t"
created_at = "2026-03-29T23:26:19.627104Z"
updated_at = "2026-03-29T23:56:30.894592Z"
+++

## Spec

### Problem

`apm validate` (ticket #54) checks ticket integrity but does not validate whether `apm.toml` itself is internally consistent. Several config fields reference external resources or other config entries, and misconfiguration fails silently at runtime:

- A state with `instructions = "apm.worker.md"` silently spawns an agent with no system prompt if the file is missing.
- A transition with `context_section = "Approach"` fails at ticket creation if `[[ticket.sections]]` has no "Approach" entry.
- A transition with `focus_section = "Code review"` has the same problem.
- A transition with `completion = "pr"` but no `[provider]` configured fails when an agent tries to open a PR — the worst possible moment.
- A non-terminal state with no outgoing transitions traps tickets permanently with no recovery path.

These problems are all detectable statically from `apm.toml` before any agent runs.

### Acceptance criteria

- [ ] `apm validate` checks that every `instructions` path on a state exists on disk relative to the repo root; reports each missing file as `config: state.<id>.instructions — file not found: <path>`
- [ ] `apm validate` checks that every `context_section` value on a transition matches a `name` in `[[ticket.sections]]` (when sections are non-empty); reports mismatches
- [ ] `apm validate` checks that every `focus_section` value on a transition matches a `name` in `[[ticket.sections]]` (when sections are non-empty); reports mismatches
- [ ] `apm validate` reports non-terminal states with no outgoing transitions as `config: state.<id> — no outgoing transitions (tickets will be stranded)`
- [ ] `apm validate` reports if `completion = "pr"` or `completion = "merge"` appears on any transition but `[provider]` is absent or has no `type`
- [ ] All config errors are printed to stderr in the format `config: <location> — <message>`
- [ ] Exit code 1 if any config errors are found (combined with existing ticket errors)
- [ ] `apm validate --config-only` skips ticket integrity checks and runs only config cross-checks
- [ ] Integration test: a config with a missing instructions file and a mismatched `context_section` produces the expected two error lines and exits 1

### Out of scope

- Auto-fixing misconfiguration (`--fix`)
- Validating transition actor values against an enum (actors are extensible)
- Graph reachability checks (whether all states are reachable from `new`)
- Validating the content of instruction files

### Approach

In `apm/src/cmd/validate.rs`, add `validate_config(config: &Config, root: &Path) -> Vec<String>`:

1. **Instructions files**: for each state where `instructions.is_some()`, check `root.join(path).exists()`.
2. **Section name references**: collect section names into a set. For each transition with `context_section` or `focus_section` set, verify the name exists in the set (skip if `ticket.sections` is empty).
3. **Dead-end states**: for each non-terminal state with an empty `transitions` vec, emit a warning.
4. **Completion without provider**: if any transition has `completion != None` and `config.provider.type_` is empty or provider is missing, emit error.

Add `--config-only` flag to the `Validate` subcommand in `main.rs`. In `run`, always call `validate_config` before or after ticket checks and merge the error lists.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:26Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:26Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:31Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:53Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
