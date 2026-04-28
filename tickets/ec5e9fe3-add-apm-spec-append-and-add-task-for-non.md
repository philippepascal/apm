+++
id = "ec5e9fe3"
title = "Add apm spec --append and --add-task for non-destructive section updates"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ec5e9fe3-add-apm-spec-append-and-add-task-for-non"
created_at = "2026-04-27T22:17:27.580621Z"
updated_at = "2026-04-28T01:15:15.172904Z"
+++

## Spec

### Problem

`apm spec <id> --section <name> --set` and `--set-file` replace the entire section content. This is destructive for sections that accumulate over time as a decision record — specifically `Amendment requests` and `Open questions` (`.apm/agents.md`: "Do not delete answered questions or checked amendment items — they are the decision record"). The only non-destructive writer today is `--mark`, which is read-only on items it doesn't match. There is no constructive complement.

Real incident: during ticket 941e57fa amendment, calling `apm spec --set-file` to write new amendment requests erased a previously-checked amendment item from a prior round. Recovery required reading the prior commit on the ticket branch and re-stitching the content manually.

Proposed additions to `apm spec`:

1. **`--append <text>`** and **`--append-file <path>`** — generic appenders. Append the given content to the existing section with a newline separator. Works for any section type (`free`, `qa`, `tasks`). Auto-commits to the ticket branch like `--set`.

2. **`--add-task <text>`** — typed sugar for sections with `type = "tasks"` (Acceptance criteria, Amendment requests). Appends `- [ ] <text>` to the list. Errors out cleanly if invoked on a non-tasks section, catching writer mistakes early.

Behavior on missing section: `--set` today creates the section if absent; the appenders should match that behavior so they're drop-in safe.

Implementation lives in `apm-core/src/spec.rs` (where `set_section` already lives) and the CLI handler in `apm/src/cmd/spec.rs`.

### Acceptance criteria

- [x] **`--append` / `--append-file`**

- [x] `apm spec <id> --append <text>` without `--section` exits non-zero with an error containing `"--append requires --section"`
- [x] `apm spec <id> --section <name> --append <text>` appends the trimmed text after the existing section content, separated by a single newline
- [x] When the target section is empty or absent, `--append` creates it with the new text (no leading newline)
- [x] `apm spec <id> --section <name> --append-file <path>` reads the file at `<path>` and appends its contents to the section identically to `--append`
- [x] `--append-file` without `--section` exits non-zero with an error containing `"--append-file requires --section"`
- [x] Supplying both `--append` and `--set` (or `--set-file`) exits with a clap conflict error
- [x] Supplying both `--append` and `--append-file` exits with a clap conflict error
- [x] When config is active and the section has a defined type, `--append` applies `apply_section_type` formatting to the appended text before committing (consistent with `--set`)
- [x] `--append` commits to the ticket branch with message `ticket(<id>): append to section <name>`
- [x] When aggressive sync is enabled, `--append` pushes to origin after the commit; a push failure prints a warning but does not fail the command

- [x] **`--add-task`**

- [x] `apm spec <id> --add-task <text>` without `--section` exits non-zero with an error containing `"--add-task requires --section"`
- [x] `apm spec <id> --section <name> --add-task <text>` appends `- [ ] <text>` to the named section
- [ ] When the target section is empty or absent, `--add-task` creates it with `- [ ] <text>` as its sole item
- [ ] When config is active and the named section has `type != "tasks"`, `--add-task` exits non-zero with an error that names the actual section type
- [ ] `--add-task` commits to the ticket branch with message `ticket(<id>): add task to <name>`
- [ ] When aggressive sync is enabled, `--add-task` pushes to origin after the commit; a push failure prints a warning but does not fail the command
- [ ] Supplying `--add-task` together with `--set`, `--set-file`, `--append`, or `--append-file` exits with a clap conflict error

### Out of scope

- Stdin input via `-` for `--append` (only `--set` supports stdin today; no regression, just not extended)\n- Type validation for `--add-task` when no `[ticket.sections]` config is present — no config means no type to validate; the item is appended unconditionally\n- Changes to existing flag behaviour: `--set`, `--set-file`, `--mark`, `--check` are untouched\n- Bulk append (appending to multiple sections in one invocation)\n- A blank-line separator variant for `--append` (single newline is the separator)

### Approach

**`apm-core/src/spec.rs` — add `append_section`**

Add one new public function below `set_section`:

```rust
pub fn append_section(doc: &mut TicketDocument, name: &str, value: String) {
    let existing = get_section(doc, name).unwrap_or_default();
    let new_value = if existing.trim().is_empty() {
        value
    } else {
        format!("{}\n{}", existing.trim_end(), value)
    };
    set_section(doc, name, new_value);
}
```

`trim_end()` on existing prevents a double blank line when the stored value has a trailing newline. Add three unit tests in the `#[cfg(test)]` block:
- `append_section_adds_to_existing` — verifies existing + "\n" + new
- `append_section_creates_when_absent` — section not in doc; value becomes the content
- `append_section_treats_empty_section_as_absent` — section exists but is `""` or whitespace-only; result is just the new value

No other changes to `spec.rs`.

---

**`apm/src/main.rs` — extend `Spec` variant in `Command` enum**

Add three new fields inside the `Spec { … }` struct variant, after `mark`:

```rust
/// Append content to the section without replacing existing content; use "-" to read from stdin
#[arg(long, allow_hyphen_values = true, conflicts_with_all = ["set", "set_file", "append_file", "add_task"])]
append: Option<String>,

/// Read content to append from this file
#[arg(long, value_name = "PATH", conflicts_with_all = ["set", "set_file", "append", "add_task"])]
append_file: Option<String>,

/// Append a new unchecked task item (`- [ ] <text>`) to a tasks-typed section
#[arg(long, conflicts_with_all = ["set", "set_file", "append", "append_file"])]
add_task: Option<String>,
```

Update the dispatch arm for `Command::Spec` to pass the three new arguments to `cmd::spec::run`.

---

**`apm/src/cmd/spec.rs` — extend `run` signature and add handling**

1. Add `append: Option<String>`, `append_file: Option<String>`, `add_task: Option<String>` to the `run` function signature.

2. Add guard clauses immediately after the existing `--mark` / `--set` guards:
   ```rust
   if append.is_some() && section.is_none()    { bail!("--append requires --section"); }
   if append_file.is_some() && section.is_none() { bail!("--append-file requires --section"); }
   if add_task.is_some() && section.is_none()  { bail!("--add-task requires --section"); }
   ```

3. After the `--mark` early-return block, before the `--set` block, insert two new blocks. These run after `doc` is parsed (move them after `let mut doc = ...`).

   **`--add-task` block** (insert before `--append` block):
   ```rust
   if let Some(ref task_text) = add_task {
       let name = section.as_ref().unwrap();
       if config_active {
           match config.find_section(name) {
               Some(sc) if sc.type_ != SectionType::Tasks =>
                   bail!("--add-task requires a tasks section; {:?} has type {:?}", name, sc.type_),
               None => bail!("unknown section {:?}; not defined in [ticket.sections]", name),
               _ => {}
           }
       }
       let item = format!("- [ ] {}", task_text.trim());
       spec::append_section(&mut doc, name, item);
       t.body = doc.serialize();
       git::commit_to_branch(root, &branch, &rel_path, &t.serialize()?,
           &format!("ticket({id}): add task to {name}"))?;
       if aggressive {
           if let Err(e) = git::push_branch(root, &branch) {
               eprintln!("warning: push failed: {e:#}");
           }
       }
       println!("ticket #{id}: task added to {name:?}");
       return Ok(());
   }
   ```

   **`--append` / `--append-file` block** (insert after `--add-task` block):
   ```rust
   let append_resolved = match (append, append_file) {
       (Some(v), _) => Some(v),
       (None, Some(path)) => Some(std::fs::read_to_string(&path)
           .map_err(|e| anyhow::anyhow!("--append-file: {}: {e}", path))?),
       (None, None) => None,
   };
   if let Some(value) = append_resolved {
       let name = section.as_ref().unwrap();
       let trimmed = value.trim().to_string();
       let formatted = if config_active {
           let sc = config.find_section(name).unwrap();
           spec::apply_section_type(&sc.type_, trimmed)
       } else {
           trimmed
       };
       spec::append_section(&mut doc, name, formatted);
       t.body = doc.serialize();
       git::commit_to_branch(root, &branch, &rel_path, &t.serialize()?,
           &format!("ticket({id}): append to section {name}"))?;
       if aggressive {
           if let Err(e) = git::push_branch(root, &branch) {
               eprintln!("warning: push failed: {e:#}");
           }
       }
       println!("ticket #{id}: section {name:?} updated");
       return Ok(());
   }
   ```

4. The existing `--set` / `--set-file` block and read-only path remain unchanged below.

---

**Order matters:** place `--add-task` check before `--append` check, and both before the existing `--set` check. All three follow the `--mark` early-return and the `let mut doc` / `let config_active` lines.

**No other files change.** The `SectionType` enum already has `Tasks`; no new variants needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T22:17Z | — | new | philippepascal |
| 2026-04-27T22:55Z | new | groomed | philippepascal |
| 2026-04-27T23:05Z | groomed | in_design | philippepascal |
| 2026-04-27T23:10Z | in_design | specd | claude-0427-2305-aaf0 |
| 2026-04-28T00:25Z | specd | ready | philippepascal |
| 2026-04-28T00:26Z | ready | in_progress | philippepascal |
| 2026-04-28T00:50Z | in_progress | ready | philippepascal |
| 2026-04-28T01:15Z | ready | in_progress | philippepascal |