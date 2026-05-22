+++
id = "3c4fb21a"
title = "apm prompt: make ID optional; show agent/role discovery when called with no ID"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3c4fb21a-apm-prompt-make-id-optional-show-agent-r"
created_at = "2026-05-22T08:01:03.768635Z"
updated_at = "2026-05-22T08:08:08.900599Z"
+++

## Spec

### Problem

`apm prompt` declares its ID argument as `id: String` in the clap struct, making it a required positional. Running the command bare — or with only `--agent`/`--role` flags but no ID — causes clap to abort with a generic missing-argument error before any application code runs. This is unhelpful when a user wants to know what agents and roles are configured in the project before assembling a full prompt invocation.

The desired behaviour is a discovery mode: when no ID is supplied (regardless of whether `--agent` or `--role` are present), the command scans `.apm/agents/` for agent subdirectory names and extracts role names from `apm.<role>.md` filenames within those directories, then prints a two-line summary and exits 0. When an ID is supplied, behaviour is entirely unchanged.

### Acceptance criteria

- [ ] `apm prompt` with no arguments exits 0 and prints an `Agents:` line whose value is the sorted, comma-space-separated list of subdirectory names under `.apm/agents/`
- [ ] `apm prompt` with no arguments exits 0 and prints a `Roles:` line whose value is the sorted, comma-space-separated list of unique role names extracted from `apm.<role>.md` filenames across all agent directories
- [ ] The two output lines align their values at the same column (labels padded to equal width)
- [ ] `apm prompt --agent <name>` with no ID triggers discovery mode and produces the same output as bare `apm prompt`
- [ ] `apm prompt --role <name>` with no ID triggers discovery mode and produces the same output as bare `apm prompt`
- [ ] When `.apm/agents/` does not exist, discovery exits 0 and prints `Agents:` and `Roles:` lines with empty values rather than erroring
- [ ] `apm prompt <id>` with a valid ticket ID behaves identically to the pre-change implementation
- [ ] `apm prompt <id> --agent <a> --role <r>` continues to work as before

### Out of scope

- Changing how `apm start` or any other command resolves agents or roles
- Filtering the discovery output by the supplied `--agent` or `--role` flag value
- Discovery for commands other than `apm prompt`
- Validating that discovered agent/role pairs have usable instructions (that is a concern for the prompt-building path, not discovery)

### Approach

#### 1. `apm/src/main.rs` — make ID optional

Change the `Prompt` struct field at line 866:
```rust
// before
id: String,
// after
id: Option<String>,
```

Update the `long_about` string to document discovery mode and add an example line:
```
  apm prompt                                 # list available agents and roles
```

The dispatch at line 1198 changes from `cmd::prompt::run(&root, &id, agent, role)` to:
```rust
cmd::prompt::run(&root, id.as_deref(), agent, role)
```

#### 2. `apm/src/cmd/prompt.rs` — branch on None

Change the `run` signature to `id: Option<&str>`. When `id.is_none()`, call `apm_core::prompt::discover(root, &mut stdout)` and return early. Otherwise delegate to `apm_core::prompt::run` exactly as today.

```rust
pub fn run(root: &Path, id: Option<&str>, agent: Option<String>, role: Option<String>) -> Result<()> {
    let mut stdout = std::io::stdout();
    match id {
        None => apm_core::prompt::discover(root, &mut stdout),
        Some(id) => apm_core::prompt::run(root, id, agent.as_deref(), role.as_deref(), &mut stdout),
    }
}
```

#### 3. `apm-core/src/prompt.rs` — add `discover()`

Add a new public function. Algorithm:

1. Build the path `root/.apm/agents/`. If it does not exist or is not a directory, treat the agent list as empty.
2. Read directory entries; collect names of entries that are themselves directories. Sort lexicographically.
3. For each agent directory, read its entries; collect filenames matching `apm.*.md`; extract the `*` portion as the role name.
4. Collect all role names across all agent dirs, deduplicate, sort.
5. Format and write two lines. Pad label to 8 chars so values align:
   ```
   Agents:  claude, default, pi
   Roles:   spec-writer, worker
   ```
   If either list is empty, write the label with no trailing values (just a newline).

Use `std::fs::read_dir`; ignore non-UTF-8 names silently (skip). Propagate I/O errors on the output writer.

#### 4. Tests in `apm-core/src/prompt.rs`

- **`discover_lists_agents_and_roles`**: create a tempdir with `.apm/agents/mock-happy/apm.worker.md` and `.apm/agents/pi/apm.spec-writer.md`; call `discover()`; assert output contains `Agents:  mock-happy, pi` and `Roles:   spec-writer, worker`.
- **`discover_no_agents_dir`**: tempdir with no `.apm/agents/` directory; call `discover()`; assert exits Ok(()) and output contains `Agents:` and `Roles:` lines (empty values).
- **`discover_deduplicates_roles`**: two agent dirs each containing `apm.worker.md`; assert role appears once.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T08:01Z | — | new | philippepascal |
| 2026-05-22T08:05Z | new | groomed | philippepascal |
| 2026-05-22T08:05Z | groomed | in_design | philippepascal |