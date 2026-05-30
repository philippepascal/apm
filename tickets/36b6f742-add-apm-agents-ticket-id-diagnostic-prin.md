+++
id = "36b6f742"
title = "Add apm agents <ticket-id> diagnostic: print resolved agent, role, model, manifest with provenance"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/36b6f742-add-apm-agents-ticket-id-diagnostic-prin"
created_at = "2026-05-30T16:50:01.198693Z"
updated_at = "2026-05-30T17:29:53.756055Z"
+++

## Spec

### Problem

GOAL: make the worker-assignment cascade observable. Today the only way to know which agent, role, model, manifest, container, env, etc. would be used when dispatching a given ticket is to read the cascade logic in apm-core/src/start.rs. The cascade has at least four layers (project workers.default, transition.worker_profile in workflow.toml, ticket frontmatter.agent / agent_overrides, and the per-agent manifest at .apm/agents/<agent>/<role>.toml), and the result depends on the ticket's state, target_branch, and frontmatter. A diagnostic command would let a supervisor (or another agent) ask 'if I dispatched this ticket right now, what would actually run?' before doing it.

USAGE: apm agents <ticket-id>. Resolves the same way the dispatcher would for that ticket in its current state. Prints a short, structured report. Read-only — no side effects, no state transitions, no spawning.

REPORT FIELDS:
- agent (e.g. claude, mock-agent, phi4)
- role (e.g. coder, spec-writer)
- model (e.g. sonnet, opus — whatever the cascade resolves)
- worker_profile string the cascade resolved (the literal agent/role pair)
- per-agent manifest path (.apm/agents/<agent>/<role>.toml or absent)
- container, env, keychain entries, command override — whatever the manifest contributes
- PROVENANCE for each field: which layer supplied it. The four layers as labels: workers.default, workflow.toml transition <from>→<to>, ticket frontmatter, frontmatter agent_overrides (with the matched profile key), per-agent manifest. Identical in spirit to apm prompt --explain's provenance display.

REPORT SHAPE (suggestion; spec-writer can refine):

  Agent assignment for <ticket-id> (state: <state>):
    agent          claude            (workflow.toml: ready→in_progress.worker_profile)
    role           coder             (workflow.toml: ready→in_progress.worker_profile)
    model          sonnet            (.apm/agents/claude/coder.toml)
    container      —
    manifest       .apm/agents/claude/coder.toml
    env            (none)
    keychain       (none)

When a layer overrides another, both the chosen and overridden source should be visible so the user understands why the cascade resolved as it did. agent_overrides entries should explicitly call out the key match.

BEHAVIOR DETAILS:
- The command runs the exact same resolution helpers used by the spawn path (resolve_worker_profile, apply_profile_manifest, apply_frontmatter_agent — and any future additions). The spec-writer should refactor so the cascade is callable without spawning (return a ResolvedWorkerProfile + provenance rather than entering the spawn flow).
- If the ticket is in a state with no command:start transition (i.e. not dispatchable to a worker right now), the command still shows what WOULD run for the most natural next transition, or clearly states 'no worker dispatch defined from this state.'
- If the ticket id resolves ambiguously or not at all, error with a clear message — same UX as other apm commands taking an id.
- A --json flag emits the same data as a single JSON object for scripts.

OUT OF SCOPE:
- Spawning a worker. This command is read-only.
- Validating that the resolved agent/model would actually succeed (no network calls to verify the manifest's container exists, no model availability checks).
- Modifying frontmatter or any state.
- Changes to apm-server / apm-ui.
- A list-of-all-tickets variant (apm agents <ticket> is per-ticket; a bulk view is a separate concern if needed).
- Renaming or refactoring the existing apm agents subcommands (list, new, test, eject) — this adds a per-ticket form alongside them.

INTEGRATION POINT:
The existing apm agents command takes subcommands (list, new, test, eject). The new shape can be either:
- a new subcommand: apm agents resolve <ticket-id>
- or a positional invocation: apm agents <ticket-id>
Spec-writer to decide based on clap ergonomics. Either is acceptable; document the chosen shape clearly in apm help.

TESTS:
- Unit test (in apm-core): build a Config with workers.default = 'claude/coder', a transition.worker_profile of 'claude/coder', a manifest containing model='sonnet'. Resolve for a synthetic ticket in 'ready' state. Assert: agent=claude, role=coder, model=sonnet, manifest path matches, provenance for agent and model is reported correctly.
- Override test: same setup but with ticket frontmatter agent_overrides set to override the model. Assert the resolved model reflects the override and provenance points at agent_overrides with the matched key.
- Manifest-absent test: no .apm/agents/<agent>/<role>.toml exists. Assert model and container fields are reported as unset, manifest path shows the expected absent path, and provenance lists workers.default for agent/role.
- Integration test: full apm agents <ticket-id> against a temp repo with the default workflow; assert stdout contains the expected structured fields. JSON variant: assert valid JSON with the expected keys.

### Acceptance criteria

- [ ] `apm agents resolve <ticket-id>` prints the resolved agent name with its provenance source in parentheses
- [ ] `apm agents resolve <ticket-id>` prints the resolved role with its provenance source
- [ ] `apm agents resolve <ticket-id>` prints the resolved model (or `—` if unset) with its provenance source
- [ ] `apm agents resolve <ticket-id>` prints the manifest path and whether it is present or absent
- [ ] `apm agents resolve <ticket-id>` prints the container value (or `—` if unset) with its provenance source
- [ ] `apm agents resolve <ticket-id>` prints each env key/value pair and the layer that supplied it (workers config or manifest)
- [ ] `apm agents resolve <ticket-id>` prints the keychain map entries (names only, not resolved secrets)
- [ ] When `frontmatter.agent_overrides` supplies the agent, the provenance line names the matched key and identifies the layer it overrode
- [ ] When the ticket's current state has no `command:start` transition, the output includes a note identifying the non-dispatchable state and names the state whose transition was used for resolution
- [ ] `apm agents resolve <ticket-id> --json` emits a valid JSON object containing all the same fields, including provenance values as `<field>_source` keys
- [ ] Passing an unknown or ambiguous ticket ID produces a clear error on stderr and exits non-zero, consistent with other `apm` commands that accept a ticket ID
- [ ] Running `apm agents resolve <ticket-id>` makes no git commits, no state transitions, and no file writes

### Out of scope

- Spawning a worker or triggering any state transition
- Validating that the resolved agent binary exists or that the model is available (no network calls, no binary probing)
- Resolving keychain secret values — only the name→item mapping from config is displayed, not the actual secrets
- Rendering the full system prompt or user message text — use `apm prompt` for that
- Container existence or pull checks
- Modifying ticket frontmatter or any project file
- Changes to `apm-server` or `apm-ui`
- A bulk form that shows resolutions across all tickets — `apm agents resolve` is per-ticket only
- Renaming or removing any existing `apm agents` subcommands (`list`, `new`, `test`, `eject`)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T16:50Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:29Z | groomed | in_design | philippepascal |