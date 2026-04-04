+++
id = "133a9b13"
title = "apm init: keep default templates and gitignore entries in sync with new features"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/133a9b13-apm-init-keep-default-templates-and-giti"
created_at = "2026-04-03T23:40:56.352188Z"
updated_at = "2026-04-04T07:28:25.723869Z"
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

All changes are in \`apm-core/src/init.rs\` inside the existing \`#[cfg(test)] mod tests\` block. No production code changes.

**1. Expose \`WorkflowFile\` and \`TicketFile\` to tests**

Both structs are already defined in \`apm-core/src/config.rs\` but are private. The test module in \`init.rs\` imports \`super::*\` but \`WorkflowFile\`/\`TicketFile\` live in \`crate::config\`. Two options:
- Make \`WorkflowFile\` and \`TicketFile\` \`pub(crate)\` in \`config.rs\` — minimal change, no API surface added
- Inline the serde structs in the test module — avoids touching \`config.rs\` but duplicates definitions

Prefer option A (pub(crate) in config.rs) — it is the honest representation and avoids duplication.

**2. Add test: \`default_workflow_toml_is_valid\`**

```rust
#[test]
fn default_workflow_toml_is_valid() {
    use crate::config::WorkflowFile;
    let wf: WorkflowFile = toml::from_str(default_workflow_toml())
        .expect("default_workflow_toml must parse as WorkflowFile");
    let state_ids: Vec<&str> = wf.workflow.states.iter().map(|s| s.id.as_str()).collect();
    let expected = ["new","groomed","question","specd","ammend","in_design",
                    "ready","in_progress","blocked","implemented","closed"];
    for id in &expected {
        assert!(state_ids.contains(id), "missing state: {id}");
    }
    assert_eq!(wf.workflow.states.len(), expected.len());
    // States that must carry dep_requires
    for id in &["groomed","ammend"] {
        let s = wf.workflow.states.iter().find(|s| s.id == *id).unwrap();
        assert!(s.dep_requires.is_some(), "state {id} must have dep_requires");
    }
    // States that must satisfy deps (non-default satisfies_deps)
    use crate::config::SatisfiesDeps;
    for id in &["specd","ammend","in_design","ready","in_progress","implemented"] {
        let s = wf.workflow.states.iter().find(|s| s.id == *id).unwrap();
        assert_ne!(s.satisfies_deps, SatisfiesDeps::Bool(false),
                   "state {id} must have satisfies_deps");
    }
}
```

**3. Add test: \`default_ticket_toml_is_valid\`**

```rust
#[test]
fn default_ticket_toml_is_valid() {
    use crate::config::TicketFile;
    let tf: TicketFile = toml::from_str(default_ticket_toml())
        .expect("default_ticket_toml must parse as TicketFile");
    let required: Vec<&str> = tf.ticket.sections.iter()
        .filter(|s| s.required)
        .map(|s| s.name.as_str())
        .collect();
    for name in &["Problem","Acceptance criteria","Out of scope","Approach"] {
        assert!(required.contains(name), "required section missing: {name}");
    }
}
```

**4. Strengthen the existing gitignore test**

The existing \`ensure_gitignore_creates_file\` test (line 620) already checks for \`.apm/sessions.json\` and \`.apm/credentials.json\`. Verify it also checks for \`.apm/*.init\` — add an assertion if absent.

**Order of changes:**
1. Mark \`WorkflowFile\` and \`TicketFile\` as \`pub(crate)\` in \`config.rs\`
2. Add the two new tests to \`init.rs\`
3. Strengthen the gitignore test if the \`.apm/*.init\` assertion is missing
4. Run \`cargo test --workspace\`

### Open questions


### Amendment requests

- [ ] Add `SatisfiesDeps` enum to the list of types that need `pub(crate)` visibility in `config.rs` — the `default_workflow_toml_is_valid` test asserts against `SatisfiesDeps::Bool(false)` which won't compile if the enum is private

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
