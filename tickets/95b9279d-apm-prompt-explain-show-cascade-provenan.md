+++
id = "95b9279d"
title = "apm prompt --explain: show cascade provenance instead of prompt text"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/95b9279d-apm-prompt-explain-show-cascade-provenan"
created_at = "2026-05-22T10:22:16.387302Z"
updated_at = "2026-05-22T10:29:08.770132Z"
+++

## Spec

### Problem

`build_system_prompt()` in `apm-core/src/start.rs` resolves the agent system prompt through a 5-level cascade (level 0: per-agent file, 1: transition.instructions, 2: profile.instructions, 3: workers.instructions, 4: built-in default) preceded by an `agents.instructions` prefix layer. When a spawned worker behaves unexpectedly — wrong persona, wrong instructions — there is no way to know which level won or which file was read without manually grepping config files and checking the filesystem. `apm prompt <id>` currently prints the full assembled prompt, which does not tell the user *why* that content was chosen.

`apm prompt <id> --explain` should print a compact provenance table that names the source of each layer — which file or config path supplied the prefix, which cascade level won and what its source is, and which levels were checked or configured but not used. The flag makes the debugging loop fast: a supervisor can confirm at a glance that the right file is winning without reading a full prompt dump.

### Acceptance criteria

- [ ] `apm prompt <id> --explain` prints a provenance table to stdout instead of the prompt text
- [ ] The `prefix:` line names the `agents.instructions` file path when configured, or `none` when not configured
- [ ] The `system prompt:` line names the cascade level number (0–4), its fixed label, and its source (file path or `built-in default`)
- [ ] All cascade levels that did not win appear under `skipped:` with their fixed label and their reason (`none set`, `file absent: <path>`, or `not reached`)
- [ ] `--agent` and `--role` override flags work together with `--explain` and are reflected in the provenance output
- [ ] `apm prompt --explain` (no ticket ID) exits non-zero with a message indicating that `--explain` requires a ticket ID
- [ ] Unit tests cover: level 0 wins (per-agent file present), level 4 wins (built-in default), and prefix layer configured

### Out of scope

- Modifying `build_system_prompt()` itself or any spawn path behaviour
- Changing the default output of `apm prompt <id>` (without `--explain`)
- Machine-readable output formats (JSON, TOML)
- Explaining the `state.instructions` injection (that field is not part of the `build_system_prompt` cascade)
- Colour or ANSI formatting

### Approach

#### Data structures — `apm-core/src/start.rs`

Add three public-crate structs alongside `build_system_prompt`:

```rust
pub(crate) struct PromptProvenance {
    pub prefix_path: Option<String>,   // None = agents.instructions not configured
    pub winner: ProvenanceEntry,
    pub skipped: Vec<ProvenanceEntry>,
}
pub(crate) struct ProvenanceEntry {
    pub level: u8,            // 0–4
    pub label: &'static str,  // fixed per-level label (see below)
    pub source: String,       // file path, "built-in default (<agent>/<role>)", or reason
}
```

Fixed labels (used in both winner and skipped output):
- 0 → `"per-agent file"`
- 1 → `"transition.instructions"`
- 2 → `"profile.instructions"`
- 3 → `"workers.instructions"`
- 4 → `"built-in default"`

#### New function — `apm-core/src/start.rs`

Add `pub(crate) fn explain_system_prompt(root, transition_instructions, profile, workers, agents_instructions, agent, role) -> Result<PromptProvenance>`.

Walk the same cascade as `build_system_prompt_body()`, but instead of returning `String`:
- Collect each non-winning level as a `ProvenanceEntry` in `skipped` with its source set to the reason string:
  - Level 0: `"file absent: .apm/agents/<agent>/apm.<role>.md"` when the file does not exist
  - Level 1: `"none set"` when `transition_instructions` is `None`
  - Level 2: `"none set"` when `profile` is `None` or `profile.instructions` is `None`
  - Level 3: `"none set"` when `workers.instructions` is `None`
  - Levels below the winner: `"not reached"` (still populate entries for completeness)
- Set `winner` to the `ProvenanceEntry` for the level that would have been returned by `build_system_prompt_body()`.
- For the prefix layer: check `agents_instructions`; set `prefix_path` to `Some(path.display().to_string())` if configured and non-empty, else `None`.
- Level 5 (no instructions found) should still propagate as an error, not a provenance entry.

#### Output formatting — `apm-core/src/prompt.rs`

Add `pub fn explain(root, id, agent_override, role_override, out) -> Result<()>`.

- Resolve params identically to `run()` (same config load, ticket lookup, transition resolution, profile/agent/role cascade).
- Call `explain_system_prompt(...)` to get `PromptProvenance`.
- Format and write the provenance table:

```
prefix:         .apm/agents/default/agents.md  (agents.instructions)
system prompt:  .apm/agents/claude/apm.worker.md  (level 0 — per-agent file)
skipped:        level 1 (transition.instructions — none set)
                level 2 (profile.instructions — none set)
                level 3 (workers.instructions — none set)
```

  - Column widths: `"prefix:"` and `"system prompt:"` labels left-padded to 16 chars; `"skipped:"` label left-padded to 16 chars for the first skipped entry, blank 16-char indent for subsequent entries.
  - When `prefix_path` is `None`, emit `prefix:         none`.
  - Omit a `skipped:` block entirely when there are no skipped entries (level 0 wins with prefix configured — no skipped levels 1–3 shown only when they haven't been reached; but always show all four entries 0–3 excluding the winner to give a complete picture).

  Concrete rule: always emit exactly one entry for each level 0–3 that is not the winner, plus level 4 only if it is the winner. This keeps the table length predictable.

#### CLI plumbing

- `apm/src/main.rs` `Prompt` variant: add `#[arg(long)] explain: bool`. Update `long_about` to document `--explain` with the example output shown in the problem statement. Update the dispatch call on line 1202 to pass `explain`.
- `apm/src/cmd/prompt.rs`: update `run` signature to `run(root, id, agent, role, explain)`. When `explain` is `true` and `id` is `Some`, call `apm_core::prompt::explain(...)`. When `explain` is `true` and `id` is `None`, return `Err(anyhow!("--explain requires a ticket ID"))`.

#### Tests — `apm-core/src/prompt.rs`

Add tests in the existing `#[cfg(test)]` block:
- `explain_level0_wins`: fixture with `.apm/agents/mock-happy/apm.worker.md` present; assert output contains `"level 0"`, the file path, and three `"none set"` skipped entries.
- `explain_level4_wins`: fixture with no per-agent file, no transition/profile/workers instructions, agent = `"claude"`; assert output contains `"level 4"` and `"built-in default"`.
- `explain_prefix_shown`: fixture with `agents.instructions` pointing to a readable file; assert prefix line names the file path.
- `explain_no_id_errors`: call `cmd::prompt::run` with `explain=true` and `id=None`; assert non-zero exit / error result.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T10:22Z | — | new | philippepascal |
| 2026-05-22T10:23Z | new | groomed | philippepascal |
| 2026-05-22T10:23Z | groomed | in_design | philippepascal |
| 2026-05-22T10:29Z | in_design | specd | claude-0522-1023-2ed8 |
