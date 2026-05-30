+++
id = "48d3932b"
title = "Simplify apm prompt --explain output: hide cascade detail when no fallback fired"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/48d3932b-simplify-apm-prompt-explain-output-hide-"
created_at = "2026-05-30T07:40:46.558546Z"
updated_at = "2026-05-30T17:17:00.283744Z"
+++

## Spec

### Problem

`apm prompt --explain` currently produces confusing output in three ways. First, the layer-3 line includes parenthetical `(level N — label)` text that conflates cascade level numbers with the layer concept, forcing users to decode what "level" means vs. "layer". Second, a `skipped:` block appears at the same indent as the layer lines, making it look like a fourth layer rather than a sub-detail of layer 3. Third, even in the common case where the agent's own role file resolves immediately, two `not reached` lines are printed — noise that adds no information.

The desired output collapses to the minimum needed: show what was used, and when the cascade fell back, explain why. When the per-agent file exists, print its path on layer 3 with no cascade block. When one or both on-disk candidates were missing, show a single indented sub-line naming the path(s) that triggered the fallback.

### Acceptance criteria

- [ ] When the per-agent file `.apm/agents/{agent}/apm.{role}.md` exists, `apm prompt --explain` prints exactly three numbered layer lines and no cascade sub-line or skipped block.
- [ ] When one fallback fired (per-agent file absent, claude default file present), layer 3 shows the claude default path followed by a single `(fallback — <agent-specific path> not found)` sub-line indented to align with the layer-3 content.
- [ ] When both on-disk candidates are absent and the built-in default is used, layer 3 shows `built-in {agent}/{role} default` followed by a `(fallback — <path1> not found,` line and a continuation line `<path2> not found)` aligned under `— `.
- [ ] The output begins with a header `System prompt for {agent}/{role} — 3 layers composed:` followed by a blank line.
- [ ] Layer 1 reads `  1  apm instructions (dynamic)` with no role parenthetical.
- [ ] Layer 2 reads `  2  {path}` when a project file is configured, or `  2  (not configured)` when it is not.
- [ ] When the per-agent file exists and `agent=claude`, the output contains neither the word `skipped` nor the word `cascade`.
- [ ] `apm prompt --explain` without a ticket (using `--agent`/`--role` flags) produces the same new format.
- [ ] All existing `cargo test --workspace` tests pass after the changes.

### Out of scope

- Layer 1 and layer 2 output changes beyond removing the role parenthetical from layer 1 and renaming the `layer N:` prefix to the numbered format.
- Removing the layer 2 line when `.apm/project.md` is unset.
- Changing the cascade resolution order or adding new cascade levels.
- JSON output of `apm prompt` (if any) — this ticket is purely the human-readable text path.
- `apm-server` / `apm-ui` surfacing of prompt provenance.
- TTY colorization or width-detection beyond the existing baseline.

### Approach

#### Data structure (`apm-core/src/start.rs`)

Replace the existing `PromptProvenance` / `ProvenanceEntry` pair with a leaner shape:

```rust
pub(crate) struct PromptProvenance {
    pub layer2_path: Option<String>,
    pub layer3_source: String,     // resolved path or "built-in {agent}/{role} default"
    pub missed_paths: Vec<String>, // paths tried and absent, in traversal order
}
```

Remove `layer1_role` (the header line now carries agent/role), `ProvenanceEntry`, and `LEVEL_LABELS`. `missed_paths` contains only paths that were actually tried and found absent — "not reached" entries are never recorded.

#### `explain_system_prompt` (`apm-core/src/start.rs`)

Rewrite to populate the new struct. The cascade check order is unchanged; only the reporting changes:

- Level 0 found: `layer3_source = per_agent_rel`, `missed_paths = []`
- Level 0 absent: push `per_agent_rel` to `missed_paths`; check level 1
- Level 1 found (only when `agent != "claude"`): `layer3_source = claude_rel`, `missed_paths = [per_agent_rel]`
- Level 1 absent: push `claude_rel` to `missed_paths`; check level 2
- Level 2 (built-in): `layer3_source = "built-in {agent}/{role} default"`, `missed_paths` holds what was tried

The `agent != "claude"` guard that skips the claude-fallback check is unchanged; when agent is claude the fallback level is structurally the same path as level 0, so no entry is added for it.

#### `format_provenance` (`apm-core/src/prompt.rs`)

Change signature to accept `agent` and `role`:

```rust
fn format_provenance(prov: &PromptProvenance, agent: &str, role: &str, out: &mut dyn Write) -> Result<()>
```

Update the two call sites (`explain` and `explain_without_ticket`) to pass the already-resolved agent and role strings.

Rewrite the body to emit:

```
System prompt for {agent}/{role} — 3 layers composed:

  1  apm instructions (dynamic)
  2  {layer2_path or "(not configured)"}
  3  {layer3_source}
     (fallback — {missed[0]} not found)           // only when missed_paths non-empty
     (fallback — {missed[0]} not found,            // two-path form for Case 3
                 {missed[1]} not found)
```

Layout constants: layer lines use `  N  ` (2-space + digit + 2-space = 5-char) prefix. The fallback sub-line uses 5 spaces then `(fallback — ` (12 chars = 17 chars total) before the first path. When a second missed path exists, it continues on the same opening line ending in `,` and a newline, then a continuation line indented by 17 spaces to align under `— `.

#### Tests (`apm-core/src/prompt.rs`)

Update four existing explain tests to match the new output shape:

- `explain_level0_wins` — remove `level 0` and `not reached` assertions; add assertion that the per-agent path appears in the output and no fallback sub-line is present.
- `explain_level2_wins` — remove `level 2` assertion; change `built-in default` check to `built-in claude/coder default`.
- `explain_prefix_shown` — update to find the `  2  ` line (not `layer 2:`) for the project path assertion.
- `explain_agent_role_override` — remove `level 2` assertion; check `built-in claude/coder default` and that the agent-specific path appears in the fallback sub-line.

Add four new tests using a helper that constructs a `PromptProvenance` directly and calls `format_provenance` without a git repo:

1. **Case 1 (no fallback)** — `missed_paths = []`: assert output has no `(fallback` line and no `skipped` or `cascade` words.
2. **Case 2 (one fallback)** — `missed_paths = [".apm/agents/phi4/apm.coder.md"]`: assert the fallback sub-line names that path with `not found`.
3. **Case 3 (two fallbacks)** — `missed_paths = [".apm/agents/my-bot/apm.coder.md", ".apm/agents/claude/apm.coder.md"]`: assert both paths appear and the second is indented by 17 spaces.
4. **Regression guard (agent=claude)** — same as Case 1 but `agent=claude`: assert neither `skipped` nor `cascade` appears in the output.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T07:40Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:13Z | groomed | in_design | philippepascal |