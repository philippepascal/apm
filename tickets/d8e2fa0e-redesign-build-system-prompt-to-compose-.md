+++
id = "d8e2fa0e"
title = "Redesign build_system_prompt to compose three layers"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d8e2fa0e-redesign-build-system-prompt-to-compose-"
created_at = "2026-05-22T23:23:06.850140Z"
updated_at = "2026-05-23T00:25:13.568768Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771", "edb0cf35"]
+++

## Spec

### Problem

`build_system_prompt` (apm-core/src/start.rs) currently works as: prepend the file at `config.agents.instructions` → then pick a single cascade winner from the role-file cascade (per-agent file | transition | profile | workers | built-in). The prefix is optional and always the same content regardless of role.

The new model replaces this with three explicitly named, ordered layers: (1) `apm_core::instructions::generate()` output (from T1/4bee5771, scoped to the role), (2) the project context file at `config.agents.project` (default path `.apm/agents/default/apm.project.md`), (3) the existing role-file cascade unchanged. All three are joined with a blank line between each present layer. The `[agents]` config key changes from `instructions` to `project`; the old key is deprecated — if present without `project`, use it as layer 2 and emit a deprecation warning.

`explain_system_prompt` and `format_provenance` must be updated so `apm prompt --explain` shows the source for all three layers rather than a separate "prefix" line plus a single "system prompt" line.

### Acceptance criteria

- [ ] `build_system_prompt` output, when all three layers are present, contains Layer 1 text, then a blank line, then Layer 2 text, then a blank line, then Layer 3 text — in that order
- [ ] When `agents.project` is not configured (None or empty string), Layer 2 is absent and the output is Layer 1 + blank line + Layer 3 with no extra blank line or gap
- [ ] When `agents.project` names a file that cannot be read, `build_system_prompt` returns an error whose message contains `"agents.project"` and the configured path
- [ ] `AgentsConfig` deserialises `project = "..."` from the `[agents]` section of `config.toml` and stores it as `project: Option<PathBuf>`
- [ ] When `[agents].instructions` is set and `[agents].project` is absent, `build_system_prompt` uses the `instructions` path as Layer 2 and emits a deprecation warning to stderr
- [ ] `apm prompt --explain` output labels all three layers: a `layer 1:` line for apm instructions (dynamic), a `layer 2:` line for the project file path (or "not configured"), and a `layer 3:` line for the cascade winner
- [ ] `apm prompt --agent A --role R` output begins with the content returned by `instructions::generate(root, Some(R), &[])`

### Out of scope

- Rewriting the role-file cascade logic (levels 0–4 within Layer 3) — unchanged by this ticket
- Updating `apm instructions` CLI help text or `apm prompt` help text — covered by bfa41899
- Rewriting the content of `apm.worker.md` or `apm.spec-writer.md` — covered by 78eeb755 and 34ad9126
- Implementing `apm_core::instructions::generate()` — covered by 4bee5771 (T1, a declared dependency)
- Creating `apm.project.md` or `apm.main-agent.md` built-in defaults — covered by edb0cf35 (T2, a declared dependency)
- Deleting `agents.md` or migrating `.apm/agents/` — covered by 1fce91bd and 7c5c491d
- Removing the `claude/apm.worker.md` built-in override — covered by 02bbcc2f
- Validating that the project file conforms to any schema
- Adding a `--project` CLI flag to `apm prompt`

### Approach

#### 1. `apm-core/src/config.rs` — extend `AgentsConfig`

Add `pub project: Option<PathBuf>` with `#[serde(default)]` immediately after `max_workers_on_default`. Keep the existing `pub instructions: Option<PathBuf>` field (no removal yet — needed for deprecation detection). Update the `Default` impl: `project: None`. Add a doc comment to `instructions` marking it deprecated.

Add a method to `AgentsConfig`:

```rust
pub fn effective_project(&self) -> Option<&Path> {
    self.project.as_deref().or_else(|| self.instructions.as_deref())
}
```

Deprecation warning emission is handled at call sites (see step 4).

#### 2. `apm-core/src/start.rs` — reshape `PromptProvenance`

Replace `prefix_path: Option<String>` with two fields:
- `pub layer1_role: Option<String>` — the role string passed to `instructions::generate`; `None` means instructions were not generated (should not occur in normal flow)
- `pub layer2_path: Option<String>` — the project file path, or `None` if layer 2 was not configured

Keep `winner: ProvenanceEntry` and `skipped: Vec<ProvenanceEntry>` unchanged (they describe the cascade resolution within layer 3).

#### 3. `apm-core/src/start.rs` — update `build_system_prompt`

Rename parameter `agents_instructions: Option<&Path>` → `project_file: Option<&Path>`. New body:

1. **Layer 1**: call `crate::instructions::generate(root, Some(role), &[])` — propagate errors.
2. **Layer 2**: if `project_file` is `Some(path)` and non-empty, read `root.join(path)` — return a hard error naming `"agents.project"` and the path on failure. If `None` or empty, `layer2 = None`.
3. **Layer 3**: call `build_system_prompt_body(...)` unchanged.
4. Compose: `[layer1.trim_end(), layer2.as_deref().map(str::trim_end), layer3.trim_end()]` — collect present parts and join with `"\n\n"`.

#### 4. `apm-core/src/start.rs` — update the `build_system_prompt` call at line 363

Replace `config.agents.instructions.as_deref()` with the resolved project path. Emit a one-time deprecation warning when `config.agents.project.is_none() && config.agents.instructions.is_some()` — reuse the existing `DEPRECATION_WARNED` + `emit_deprecation_warning_to` pattern with a new message: `"apm: deprecated: [agents].instructions renamed to [agents].project — update config.toml"`.

Pass `config.agents.effective_project()` as `project_file`.

#### 5. `apm-core/src/start.rs` — update `explain_system_prompt`

Rename parameter `agents_instructions` → `project_file`. Populate the updated `PromptProvenance`:
- `layer1_role`: `Some(role.to_string())`
- `layer2_path`: `project_file.filter(|p| !p.as_os_str().is_empty()).map(|p| p.display().to_string())`
- `winner`, `skipped`: unchanged from current logic

#### 6. `apm-core/src/prompt.rs` — update all four callsites

`run`, `explain`, `run_without_ticket`, `explain_without_ticket` each call `build_system_prompt` or `explain_system_prompt` with `config.agents.instructions.as_deref()`. Replace with `config.agents.effective_project()`. Apply the same one-time deprecation warning as step 4 (check `project.is_none() && instructions.is_some()` before each call).

#### 7. `apm-core/src/prompt.rs` — update `format_provenance`

Replace the current `prefix:` + `system prompt:` output with three labeled lines:

```
layer 1:         apm instructions (dynamic, role: <role>)
layer 2:         <path>                   (or "not configured")
layer 3:         <source>  (level N — <label>)
skipped:         level N (<label> — <source>)
```

`layer1_role` drives the `role:` annotation. `layer2_path` drives the layer 2 line. The existing `winner`/`skipped` render under `layer 3:` and `skipped:` unchanged.

#### 8. Tests in `apm-core/src/start.rs`

Rename and update the four `agents_instructions_*` tests:
- `agents_instructions_prepended_with_blank_line` → assert layer 2 content appears between layer 1 and layer 3 (not at the start)
- `agents_instructions_none_is_no_op` → assert output is `layer1 + "\n\n" + layer3` (layer 2 absent)
- `agents_instructions_empty_path_is_no_op` → same shape as above
- `agents_instructions_missing_file_is_hard_error` → assert error message contains `"agents.project"` (not `"agents.instructions"`)
- `agents_instructions_trailing_whitespace_trimmed` → assert exactly one blank line between each present layer

Add: `project_file_in_layer2` — configures project file, asserts `layer1_content + "\n\n" + project_content + "\n\n" + layer3_content`.

#### 9. Tests in `apm-core/src/prompt.rs`

Update `explain_prefix_shown`: change config to use `project = "..."`, change assertion from checking `prefix:` line to checking that output contains `"layer 2:"` and the configured file path. Update `make_explain_project` helper to write `project = "..."` instead of `instructions = "..."` in the config.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:20Z | groomed | in_design | philippepascal |