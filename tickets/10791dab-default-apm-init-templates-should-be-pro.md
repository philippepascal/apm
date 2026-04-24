+++
id = "10791dab"
title = "Default apm init templates should be project-agnostic"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/10791dab-default-apm-init-templates-should-be-pro"
created_at = "2026-04-24T06:28:34.301755Z"
updated_at = "2026-04-24T07:39:34.566444Z"
+++

## Spec

### Problem

The three default templates shipped by `apm init` — `apm.agents.md`, `apm.spec-writer.md`, and `apm.worker.md` — contain hardcoded references to the APM project's own codebase. Specifically:

- `apm.worker.md` names `apm-core/src/` and `apm-core/tests/` as the locations for unit tests, and `apm/tests/integration.rs` as the integration test file. It also hard-codes `cargo test --workspace` as the test command.
- `apm.agents.md` hard-codes `cargo test --workspace` in both the Development workflow list and the shell-discipline section's `bash -c` example.

When a user runs `apm init` in a new project (e.g. a Python service, a Go CLI, or the `ticker` repo), these files land verbatim in `.apm/`. The agent that reads them gets wrong path references and a wrong test command. The user must manually rewrite three files every time.

The desired behaviour: the defaults should be project-agnostic placeholders. Cargo- and APM-path-specific text should be replaced with phrasing like "Run your project's test suite" and "Write tests appropriate for your project's structure." The `## Repo structure` section of `apm.agents.md` is already generic (`_Fill in your project's structure here._`) and is the model for the rest.

A second gap: the templates do not document the `####` subsection convention. Supervisors and spec-writers use `####` headings inside long sections (e.g. `### Approach`) as editing handles — targeted `apm spec --section` calls can update a named subsection without rewriting the whole section. This convention exists in the ticker fork but is absent from the defaults.

Affected users: any developer who runs `apm init` on a non-APM project — the primary use case for `apm init`. The friction is immediate and requires manual cleanup of three files.

### Acceptance criteria

- [x] `apm.agents.md` Development workflow no longer contains `cargo test --workspace`; it reads "Run your project's test suite" (or equivalent generic phrasing)
- [x] `apm.agents.md` shell-discipline `bash -c` example no longer contains `cargo test --workspace`
- [x] `apm.worker.md` Tests section no longer references `apm-core/src/`, `apm-core/tests/`, or `apm/tests/integration.rs`
- [x] `apm.worker.md` Tests section uses generic phrasing ("Write tests appropriate for your project's structure")
- [x] `apm.worker.md` "Finishing implementation" section no longer contains `cargo test --workspace`; it reads "Run your project's test suite"
- [x] `apm.worker.md` shell-discipline `bash -c` example no longer contains `cargo test --workspace`
- [x] `apm.spec-writer.md` or `apm.agents.md` contains a note explaining that `####` headings may be used inside long spec sections (e.g. `### Approach`) as editing subsection markers
- [x] Running `grep -r "apm-core" apm-core/src/default/` returns no matches
- [ ] Running `grep -r "apm/tests" apm-core/src/default/` returns no matches

### Out of scope

- Updating `.apm/agents.md` files already written to existing projects — `apm init` only writes defaults to new projects
- Adding a project-specific test-command placeholder to the generated config — that is a separate feature
- The supervisor-only transitions feature for `apm.agents.md` — handled by a related downstream ticket
- Any changes to `workflow.toml` or `ticket.toml` defaults
- Changes to how `apm init` generates the config file dynamically

### Approach

Three files change; all edits are pure text substitutions or small additions.

#### `apm-core/src/default/apm.agents.md`

1. **Development workflow** — replace step 4:
   - Before: `Run \`cargo test --workspace\` — all tests must pass before calling \`apm state <id> implemented\``
   - After: `Run your project's test suite — all tests must pass before calling \`apm state <id> implemented\``

2. **Shell discipline `bash -c` example** — replace the command inside the example:
   - Before: `bash -c "cd $wt && cargo test --workspace 2>&1"`
   - After: `bash -c "cd $wt && <your-test-command> 2>&1"`

3. **`####` convention note** — add a short note at the end of the `## Spec quality bar` section (or as a standalone `## Spec subsection convention` section near the top of the spec-quality block):
   ```
   #### Subsection markers

   Within long sections such as `### Approach` or `### Acceptance criteria`,
   use `####` headings as named editing handles. This lets `apm spec <id>
   --section "Approach > Phase 2"` target a subsection without overwriting the
   whole section.
   ```

#### `apm-core/src/default/apm.worker.md`

4. **Tests section** — replace the three bullet points:
   - Before:
     ```
     - Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
     - Integration tests in `apm/tests/integration.rs` — use temp git repos, no
       fixture files needed
     - Run `cargo test --workspace` — all tests must pass before calling `apm state <id> implemented`
     ```
   - After:
     ```
     - Write tests appropriate for your project's structure and conventions
     - Run your project's test suite — all tests must pass before calling `apm state <id> implemented`
     ```

5. **"Finishing implementation" section** — replace the run line:
   - Before: `Run \`cargo test --workspace\` — all tests must pass.`
   - After: `Run your project's test suite — all tests must pass.`

6. **Shell discipline `bash -c` example** — same substitution as in `apm.agents.md`:
   - Before: `bash -c "cd $wt && cargo test --workspace 2>&1"`
   - After: `bash -c "cd $wt && <your-test-command> 2>&1"`

#### `apm-core/src/default/apm.spec-writer.md`

7. No apm-specific paths exist here. Add only the `####` subsection convention note at the end of the `## Approach` section (mirrors the note added to `apm.agents.md`):
   ```
   Use `####` headings within long sections to create named subsections that
   serve as editing handles. Example: inside `### Approach`, add `#### Phase 1`
   so a future `apm spec <id> --section "Approach > Phase 1"` can update that
   block without touching the rest.
   ```

#### Verification

After editing, confirm:
```
grep -r "apm-core" apm-core/src/default/   # must return nothing
grep -r "apm/tests" apm-core/src/default/  # must return nothing
grep -r "cargo test" apm-core/src/default/ # must return nothing
```

No Rust tests are added or modified — this is a pure content change to embedded markdown strings. The existing test suite (`cargo test --workspace`) verifies that `apm init` still writes all three files correctly; no new tests are needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:13Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |
| 2026-04-24T07:19Z | in_design | specd | claude-0424-0714-f230 |
| 2026-04-24T07:25Z | specd | ready | philippepascal |
| 2026-04-24T07:39Z | ready | in_progress | philippepascal |