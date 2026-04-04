+++
id = "133a9b13"
title = "apm init: keep default templates and gitignore entries in sync with new features"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/133a9b13-apm-init-keep-default-templates-and-giti"
created_at = "2026-04-03T23:40:56.352188Z"
updated_at = "2026-04-04T16:05:55.798293Z"
+++

## Spec

### Problem

\`apm init\` installs default templates (workflow.toml, ticket.toml, config.toml, agents.md, apm.spec-writer.md, apm.worker.md) and maintains \`.gitignore\` entries. These defaults drift out of sync as new features land — the \`dep_requires\`/\`satisfies_deps\` fields were absent from \`default_workflow_toml()\` until a post-hoc fix, and the gitignore entries for \`.apm/sessions.json\` and \`.apm/credentials.json\` required a separate audit pass.

The root cause is that there are no typed-struct parse tests for the default templates. The existing test (\`default_config_escapes_special_chars\`) validates \`default_config()\` against \`toml::Value\` only, and no test parses \`default_workflow_toml()\` as a \`WorkflowFile\` struct or \`default_ticket_toml()\` as a \`TicketFile\` struct. A structural regression (missing field, wrong TOML shape) in either template would pass all tests today.

The audit itself is already done: \`ensure_gitignore\` already lists \`.apm/sessions.json\` and \`.apm/credentials.json\`; \`default_workflow_toml()\` already carries \`dep_requires\` and \`satisfies_deps\`. This ticket's implementation work is purely the regression-test layer that would have caught those drifts automatically.

### Acceptance criteria

- [ ] \`default_workflow_toml()\` parses without error when deserialized as \`WorkflowFile\` (the internal serde struct used by \`Config::load\`)
- [ ] The parsed workflow contains exactly the eleven expected state ids: new, groomed, question, specd, ammend, in_design, ready, in_progress, blocked, implemented, closed
- [ ] Every state that should carry \`dep_requires\` (groomed, ammend) has a non-None value after parsing
- [ ] Every state that should carry \`satisfies_deps\` (specd, ammend, in_design, ready, in_progress, implemented) has a non-default value after parsing
- [ ] \`default_ticket_toml()\` parses without error when deserialized as \`TicketFile\` (the internal serde struct used by \`Config::load\`)
- [ ] The parsed ticket config contains the four required sections: Problem, Acceptance criteria, Out of scope, Approach, all with \`required = true\`
- [ ] \`ensure_gitignore\` creates a file that contains all five expected entries: tickets/NEXT_ID, .apm/local.toml, .apm/*.init, .apm/sessions.json, .apm/credentials.json
- [ ] \`cargo test --workspace\` passes with no regressions

### Out of scope

- Changing the content of any default template — the audit is already done and the files are current
- Adding tests for the Markdown templates (apm.agents.md, apm.spec-writer.md, apm.worker.md) — those are prose files with no machine-parseable schema to validate against
- Automating future audits (e.g. CI checks that compare templates against a golden file) — that is a separate maintenance-process ticket
- Any changes to \`Config::load\`, \`WorkflowFile\`, or \`TicketFile\` structs

### Approach

All changes are in `apm-core/src/init.rs` inside the existing `#[cfg(test)] mod tests` block. No production code changes.

**1. Expose `WorkflowFile`, `TicketFile`, and `SatisfiesDeps` to tests**

All three types are already defined in `apm-core/src/config.rs` but are private. Mark them `pub(crate)` - minimal change, no public API surface added:

- `WorkflowFile` - needed to deserialize `default_workflow_toml()`
- `TicketFile` - needed to deserialize `default_ticket_toml()`
- `SatisfiesDeps` - needed for the `assert_ne!(s.satisfies_deps, SatisfiesDeps::Bool(false))` assertion in the workflow test

**2. Add test: `default_workflow_toml_is_valid`**

See inline code in the test file. Key assertions: parse succeeds, eleven states present, dep_requires non-None for groomed/ammend, satisfies_deps non-default for specd/ammend/in_design/ready/in_progress/implemented.

**3. Add test: `default_ticket_toml_is_valid`**

Parse succeeds, four required sections present: Problem, Acceptance criteria, Out of scope, Approach.

**4. Strengthen the existing gitignore test**

The existing `ensure_gitignore_creates_file` test (line 620) already checks for `.apm/sessions.json` and `.apm/credentials.json`. Verify it also checks for `.apm/*.init` - add an assertion if absent.

**Order of changes:**
1. Mark `WorkflowFile`, `TicketFile`, and `SatisfiesDeps` as `pub(crate)` in `config.rs`
2. Add the two new tests to `init.rs`
3. Strengthen the gitignore test if the `.apm/*.init` assertion is missing
4. Run `cargo test --workspace`

### Open questions


### Amendment requests

- [x] Add `SatisfiesDeps` enum to the list of types that need `pub(crate)` visibility in `config.rs` — the `default_workflow_toml_is_valid` test asserts against `SatisfiesDeps::Bool(false)` which won't compile if the enum is private

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-03T23:40Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:31Z | groomed | in_design | philippepascal |
| 2026-04-04T06:34Z | in_design | specd | claude-0403-spec-133a |
| 2026-04-04T07:15Z | specd | ammend | apm |
| 2026-04-04T07:28Z | ammend | in_design | philippepascal |
| 2026-04-04T07:29Z | in_design | specd | claude-0404-0900-spec1 |
| 2026-04-04T15:34Z | specd | ready | apm |
| 2026-04-04T16:05Z | ready | in_progress | philippepascal |
