+++
id = "a5cffb01"
title = "Help text and docs sweep: update every stale reference to old schema"
state = "in_progress"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a5cffb01-help-text-and-docs-sweep-update-every-st"
created_at = "2026-05-31T02:59:02.592158Z"
updated_at = "2026-06-01T01:57:54.043616Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["e05c0463", "9c66e199", "4d20ba2f"]
+++

## Spec

### Problem

STEP 9 of the incremental workflow schema cleanup. Doc audit and updates after all schema changes have landed.

KNOWN STALE LOCATIONS to audit and fix:

1. apm/src/main.rs and apm/src/cmd/*.rs — every clap long_about and short about:

   - apm prompt long_about: describes the OLD layer order ('Layer 1 — APM system knowledge ... Layer 3 — Role instructions'). After ticket 9ea43165 the layers were reversed; help text was not updated. Also shows the OLD --explain example with 'skipped:' and 'level 0 / level 2 — not reached' lines (cleaned up by 48d3932b and 9ea43165). Also references 'shell discipline' as part of Layer 1 content (moved to role file by a3c34ddc). Rewrite the long_about to reflect the current shape.

   - apm instructions short about: currently 'Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference'. The 'shell discipline,' clause is stale (a3c34ddc).

   - apm validate, apm new, apm state, apm next, apm list, and any other command whose help touches workflow concepts that have changed (actionable, worker_profile, role).

2. apm help <topic> output. Topics include commands, config, workflow, ticket. The workflow topic in particular needs a rewrite covering: state-level worker_profile, trigger uniqueness rule, no actionable field, no transition.worker_profile, mandatory workers.default.

3. apm/src/cmd/show.rs print_ticket — verify nothing references dropped fields (actionable). Today it does not seem to but verify.

4. apm-core/src/init.rs — if the init scaffold writes any prose about the workflow, audit and update. Especially the [workers] section in the scaffolded config.toml (4d20ba2f makes default mandatory).

5. README files: top-level README.md, apm-core/README.md, apm-server/README.md, apm-ui/README.md. Search for: 'transition.worker_profile', 'transition.role', 'actionable', 'role: worker', 'agent/supervisor', 'derive_transition_role'. Update.

6. docs/ directory if present.

7. apm-core/src/default/agents/claude/apm.coder.md, apm.spec-writer.md, apm.main-agent.md (and their .apm/ project copies). If any reference 'actionable' or describe transitions removed in 071886fc, update.

8. apm-core/src/default/workflow.toml comments — if the file has comments explaining the schema, they may reference the old field names.

9. Code comments in apm-core/src that explain workflow concepts. Grep for old terminology.

APPROACH:
- ripgrep for 'transition.worker_profile', 'transition.role', 'actionable', 'derive_transition_role', 'role_command_allowlist', 'Actionable by'
- Read each hit, decide if it is stale, update with new vocabulary
- Build the project; run apm <subcommand> --help for each command to spot-check rendered text
- Run cargo test --workspace

TESTS:
- 'apm instructions' short summary does not contain 'shell discipline'
- 'apm prompt --help' long_about does not contain old layer labels (where Layer 1 = APM system knowledge) and does not contain the 'skipped:' / 'level N — not reached' format
- A grep for 'transition.worker_profile' across apm-core/src/, apm/src/, apm-server/src/ returns nothing (excluding tests / archive / tickets if scanned)
- A grep for 'actionable' returns nothing in source code (excluding test fixtures that explicitly test the absence and historical tickets)
- A grep for 'derive_transition_role' returns nothing in source code

OUT OF SCOPE:
- Functional code changes (handled by earlier tickets in the epic).
- apm-server / apm-ui surfaces (next ticket).
- Updating historical ticket markdown files in tickets/ or archive/ (those are records, not docs).

REFERENCES:
- apm/src/main.rs
- apm/src/cmd/*
- apm-core/src/init.rs
- README files
- apm-core/src/default/agents/claude/*.md

### Acceptance criteria

- [x] `apm instructions --help` output does not contain the phrase "shell discipline"
- [x] `apm prompt --help` SYSTEM PROMPT section lists role instructions as Layer 1 and APM system knowledge as Layer 3
- [x] `apm prompt --help` `--explain` example shows the current numbered format (`1  …`, `2  …`, `3  …`) and contains no `skipped:` or `level N —` lines
- [x] `grep -rn "shell discipline" apm/src/ apm-core/src/ --exclude-dir=target --exclude-dir=archive --exclude-dir=tickets --exclude-dir=.apm--worktrees` returns no hits
- [ ] `apm-core/src/default/agents/claude/apm.spec-writer.md` describes spec-writers as picking up tickets in `groomed` (or `ammend`) state, not `new`
- [ ] `README.md` `agents.md` row is removed and the two role-file rows are replaced with a single `agents/<agent>/apm.<role>.md` row described as "Role-specific agent instructions (generated per role by `apm init`)"; the phrase "shell discipline" no longer appears in the table
- [ ] `grep -rn "transition\.worker_profile\|derive_transition_role" apm/src/ apm-core/src/ apm-server/src/ --exclude-dir=target --exclude-dir=tests --exclude-dir=archive --exclude-dir=tickets --exclude-dir=.apm--worktrees` returns no hits
- [ ] `apm help workflow` output does not contain the string `transitions.worker_profile`
- [ ] `cargo test --workspace` passes

### Out of scope

- Functional code changes — removing `transition.worker_profile` from `validate.rs`, deleting `derive_transition_role`, unifying `role_command_allowlist` — handled by e05c0463, 9c66e199, and 4d20ba2f
- `apm-server` and `apm-ui` web UI surfaces (separate follow-on ticket in the epic)
- Historical records: `tickets/` and `archive/` Markdown files
- The `apm.coder.md` "Permitted apm commands" section — rewritten by 9c66e199
- `apm-core/src/default/workflow.toml` data fields — `worker_profile` moves from transition to state blocks via e05c0463; that file has no prose comments to update
- The static fallback state machine table in `apm-core/src/instructions.rs` (shows `apm state` instead of `apm start` for spawn transitions) — a functional correction, not a doc sweep change

### Approach

This ticket runs after e05c0463, 9c66e199, and 4d20ba2f are merged into the epic branch.

#### Phase 1 — Grep sweep

Phase 1 runs **after** the three dependencies are merged. Its purpose is to verify clean landing and catch any stale references in files not covered by those tickets. The `transition.worker_profile` and `derive_transition_role` greps must return zero hits; any remaining hit belongs to this ticket's scope.

Run these searches from the repo root:

```
grep -rn "shell discipline"           apm/src/ apm-core/src/ --exclude-dir=target
grep -n  "Layer 1"                    apm/src/main.rs
grep -n  "skipped:"                   apm/src/main.rs
grep -rn "transition\.worker_profile" apm/src/ apm-core/src/ apm-server/src/ --exclude-dir=target --exclude-dir=tests
grep -rn "derive_transition_role"     apm/src/ apm-core/src/ --exclude-dir=target --exclude-dir=tests
```

The last two should return zero hits. Any `shell discipline` hits in `main.rs` are fixed in Phase 2.

#### Phase 2 — File-by-file updates

**`apm/src/main.rs` — `apm prompt` long_about (~lines 873–914)**

Rewrite the SYSTEM PROMPT three-layer block to reflect the current order (role first, APM knowledge last):

- Layer 1 — Role instructions (cascade: per-agent file → claude fallback → built-in → error)
- Layer 2 — Project context (path at `[agents].project` in config; typically `.apm/project.md`; omitted if unset)
- Layer 3 — APM system knowledge (`apm instructions`, dynamic, role-scoped): covers state machine, ticket format, session identity, and command reference

Remove `"shell discipline,"` from the Layer 3 description.

Rewrite the `--explain` example to match `format_provenance`'s current output (see `apm-core/src/prompt.rs:230–252`). The current format is:

```
System prompt for claude/worker — 3 layers composed:

  1  .apm/agents/claude/apm.worker.md
  2  .apm/project.md
  3  apm instructions (dynamic)
```

Fallback lines appear inline under layer 1 (`(fallback — <path> not found)`), not as a separate `skipped:` entry.

Update the "Without a ticket ID" paragraph: replace "layer 3 levels 1 and 2 are skipped" with wording that uses correct layer numbers (the cascade is within Layer 1; layers 2 and 3 are always included when --agent and --role are both provided).

**`apm/src/main.rs` — `apm instructions` about (line 948)**

Remove `"shell discipline, "`:
```
Before: "state machine, ticket format, shell discipline, session identity, and command reference"
After:  "state machine, ticket format, session identity, and command reference"
```

**`apm-core/src/default/agents/claude/apm.spec-writer.md` line 3**

```
Before: This file applies when you pick up a ticket in **`new`** or **`ammend`** state.
After:  This file applies when you pick up a ticket in **`groomed`** or **`ammend`** state.
```

Spec-writers enter via `groomed → in_design`; there is no `new → in_design` transition for spec-writers.

**`README.md` — Configuration table (~lines 289–297)**

Remove the `agents.md` row (this file does not exist in the scaffold or in `.apm/`) and replace the two stale role-file rows with a single row covering the general pattern:

Before (three rows):
```
| `agents.md`           | Agent instructions: roles, workflow rules, shell discipline |
| `apm.spec-writer.md`  | Instructions fed to agents during the spec phase            |
| `apm.worker.md`       | Instructions fed to agents during the implementation phase  |
```

After (one row):
```
| `agents/<agent>/apm.<role>.md` | Role-specific agent instructions (generated per role by `apm init`) |
```

The `apm help workflow` output is auto-generated from the `WorkflowConfig` struct via `schema_entries::<WorkflowConfig>()` in `apm/src/cmd/help.rs`. Once e05c0463 removes `worker_profile` from `TransitionConfig`, the field `transitions.worker_profile` disappears from the output automatically — no manual edit to `help.rs` is needed.

#### Phase 3 — Validation

```bash
cargo build --workspace
cargo test --workspace
apm instructions | grep "shell discipline"       # must print nothing
apm prompt --help | grep "Layer 1 — APM"        # must print nothing
apm prompt --help | grep "skipped:"             # must print nothing
apm help workflow | grep "transitions.worker_profile"  # must print nothing
```

### Open questions


### Amendment requests

- [x] Add explicit exclusion paths to every grep returns nothing AC. Use --exclude-dir tests, --exclude-dir archive, --exclude-dir tickets, --exclude-dir .apm--worktrees, --exclude-dir target. Without these, the ACs will fail on legitimate test fixtures, archived tickets, and worktree copies.
- [x] Clarify ordering: is the Phase 1 grep sweep run before or after dependent tickets are merged into the branch on which this ticket runs. If before, the sweep finds stale references that should be flagged but not yet fixable. If after, the sweep verifies the dependent work landed cleanly. Specify which.
- [x] Add AC verifying apm help workflow output reflects the new schema after dependent tickets land. Either include a smoke test that runs apm help workflow and checks for absence of old field names, or document that the output is auto-generated from the schema source and therefore correct by construction.
- [x] Specify the exact text for the README agents.md row update or removal. The current AC says 'updated or removed' which is too vague. If the row is removed entirely (because agents.md no longer exists as a concept), state so. If updated, provide the exact new row text describing what role files now contain.
- [x] Amendment 1 was only partially addressed. The two grep ACs added --exclude-dir=target and --exclude-dir=tests but are missing --exclude-dir=archive, --exclude-dir=tickets, and --exclude-dir=.apm--worktrees. Add the three missing exclusions to both grep ACs so they do not fail on archived tickets, ticket markdown files, and worktree copies.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:59Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:46Z | groomed | in_design | philippepascal |
| 2026-05-31T07:55Z | in_design | specd | claude |
| 2026-05-31T19:36Z | specd | ammend | philippepascal |
| 2026-05-31T19:59Z | ammend | in_design | philippepascal |
| 2026-05-31T20:03Z | in_design | specd | claude |
| 2026-05-31T20:53Z | specd | ammend | philippepascal |
| 2026-05-31T20:55Z | ammend | in_design | philippepascal |
| 2026-05-31T20:56Z | in_design | specd | claude |
| 2026-05-31T21:04Z | specd | ready | philippepascal |
| 2026-06-01T01:57Z | ready | in_progress | philippepascal |