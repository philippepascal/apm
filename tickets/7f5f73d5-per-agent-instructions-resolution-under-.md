+++
id = "7f5f73d5"
title = "Per-agent instructions resolution under .apm/agents/<name>/"
state = "in_progress"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7f5f73d5-per-agent-instructions-resolution-under-"
created_at = "2026-04-30T20:03:33.687625Z"
updated_at = "2026-05-01T19:20:30.592124Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

The current `resolve_system_prompt` function in `apm-core/src/start.rs` uses a flat 4-level chain that ends in a silent hardcoded fallback string (`"You are an APM worker agent."`). It has no concept of which agent is being run and resolves the system prompt from a single flat path — `.apm/apm.worker.md` — shared across all agents. As custom wrappers are introduced (ticket 2c32a282), different agents may need different prompt conventions (Codex structured tags, Aider concise context, etc.); a single flat `.apm/apm.worker.md` cannot express per-agent defaults.

The desired behaviour is a 4-level resolution chain per spawn (agent A, role = worker|spec-writer, profile P):
1. `[worker_profiles.<P>].instructions` — project-level per-profile override
2. `[workers].instructions` — project-level global override, applies to all profiles
3. `.apm/agents/<A>/apm.<role>.md` — project-supplied per-agent file, if it exists
4. APM's bundled default for agent A (via `include_str!`, built-in agents only)
5. Hard error if none of the above resolve

Existing projects keep working without edits because their `[worker_profiles.spec_agent] instructions = ".apm/apm.spec-writer.md"` and `[worker_profiles.impl_agent] instructions = ".apm/apm.worker.md"` satisfy level 1. No migration is required.

The silent hardcoded fallback and the `StateConfig.instructions`-as-system-prompt path are both removed. `StateConfig.instructions` is a per-state annotation used for display and tooling (the field remains on the struct) but is no longer consumed by `resolve_system_prompt`.

### Acceptance criteria

- [x] When `[worker_profiles.<P>].instructions` is set and the referenced file exists, its content is used as the system prompt.
- [x] When `[worker_profiles.<P>].instructions` is absent (or the profile is not resolved) and `[workers].instructions` is set and the referenced file exists, its content is used as the system prompt.
- [x] When neither profile nor global `[workers].instructions` resolves and `.apm/agents/<A>/apm.<role>.md` exists in the project, its content is used as the system prompt.
- [x] When the first three levels all fail and agent A is the `claude` built-in, APM's bundled default for `apm.<role>.md` (compiled in via `include_str!`) is used as the system prompt.
- [x] When all four levels fail (custom agent, no project file, no config override), `apm start` exits with a descriptive error message that names the agent and role; no silent fallback occurs.
- [x] An existing project whose config has `[worker_profiles.spec_agent] instructions = ".apm/apm.spec-writer.md"` continues to work without any config edits.
- [ ] An existing project whose config has `[worker_profiles.impl_agent] instructions = ".apm/apm.worker.md"` continues to work without any config edits.
- [ ] `apm validate` reports a config error when `[workers].instructions` is set but the referenced file does not exist on disk.
- [ ] `apm validate` does not regress on the existing check for `[worker_profiles.<P>].instructions` pointing to a missing file.
- [ ] Both `apm.worker.md` and `apm.spec-writer.md` are compiled into the binary for the `claude` built-in (reachable at level 4 without any file on disk).
- [ ] The role (`worker` or `spec-writer`) is read from `WorkerProfileConfig.role` (defaults to `"worker"` when absent); the spec_agent profile in the `apm init` default config sets `role = "spec-writer"`.
- [ ] Unit tests cover all five levels of the chain independently.

### Out of scope

- Changing the content of `apm.worker.md` or `apm.spec-writer.md` — this ticket only changes where the file comes from, not what it says.
- Per-agent `agents.md` — the project-wide agent conventions file stays at `.apm/agents.md`, not per-agent.
- Instruction defaults for mock built-ins (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — deferred to ticket 25c92daa; those wrappers may not need per-role instruction files at all.
- Per-ticket frontmatter `agent_overrides` changing which instruction file is loaded — ticket 0ca3e019.
- Updating the `apm init` template to remove profile-level `instructions` fields — the existing template keeps its overrides; the per-agent fallback is an addition, not a replacement.
- Config field `[workers].agent` for config-driven agent selection — ticket 6cac8518; after that ticket the hardcoded `"claude"` string at call sites becomes `config.workers.agent`, but the shape of `resolve_system_prompt` does not change.
- Removing the `StateConfig.instructions` field from the config struct — the field is kept for display / tooling use; only its role as a `resolve_system_prompt` input is removed.
- Windows execute-bit or platform-specific path differences.

### Approach

**New default files**

Create two files (content copied verbatim from existing siblings):
- `apm-core/src/default/agents/claude/apm.worker.md` — copy of `apm-core/src/default/apm.worker.md`
- `apm-core/src/default/agents/claude/apm.spec-writer.md` — copy of `apm-core/src/default/apm.spec-writer.md`

Add two module-level constants in `start.rs` (or a new private `instructions.rs`):
```rust
const CLAUDE_WORKER_DEFAULT: &str = include_str!("default/agents/claude/apm.worker.md");
const CLAUDE_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/claude/apm.spec-writer.md");
```

---

**`apm-core/src/config.rs` changes**

1. Add `pub instructions: Option<String>` to `WorkersConfig` and its `Default` impl (default = None). Docstring: "Global instructions file used as the system prompt for all profiles; overridden by per-profile `instructions`.".

2. Add `pub role: Option<String>` to `WorkerProfileConfig` (default = None). Docstring: "Role name used to select the per-agent instruction file (e.g. \"worker\", \"spec-writer\"). Defaults to \"worker\" when absent.".

---

**`apm-core/src/start.rs` changes**

*New private helper:*
```rust
fn resolve_builtin_instructions(agent: &str, role: &str) -> Option<&'static str> {
    match (agent, role) {
        ("claude", "worker")       => Some(CLAUDE_WORKER_DEFAULT),
        ("claude", "spec-writer")  => Some(CLAUDE_SPEC_WRITER_DEFAULT),
        _                          => None,
    }
}
```

*Updated `resolve_system_prompt` signature and implementation:*
```rust
fn resolve_system_prompt(
    root: &Path,
    profile: Option<&WorkerProfileConfig>,
    workers: &WorkersConfig,
    agent: &str,
    role: &str,
) -> Result<String>
```

Implementation (levels in order — return on first match, bail on exhaustion):
1. If `profile.instructions` is Some(path): read `root.join(path)`; return Ok(content) on success, or bail with "[worker_profiles.*].instructions: file not found: {path}".
2. If `workers.instructions` is Some(path): read `root.join(path)`; return Ok(content) on success, or bail with "[workers].instructions: file not found: {path}".
3. Try `root.join(".apm/agents/{agent}/apm.{role}.md")`; if the file exists and is readable, return Ok(content). Missing file is not an error — fall through.
4. Call `resolve_builtin_instructions(agent, role)`; if Some(s), return Ok(s.to_string()).
5. `bail!("no instructions found for agent '{agent}' role '{role}': set [workers].instructions in .apm/config.toml or add .apm/agents/{agent}/apm.{role}.md")`

*Remove the `state_instructions` parameter* from `resolve_system_prompt`. The `StateConfig.instructions` value is no longer passed to this function at any call site.

*Update call sites* — three locations (`run()`, `run_next()`, `spawn_next_worker()`):
- Remove the local `state_instructions` variable that was fed into `resolve_system_prompt`.
- Add `let role = profile.and_then(|p| p.role.as_deref()).unwrap_or("worker");`
- Pass `&config.workers`, `"claude"` (hardcoded; replaced by `config.workers.agent` in ticket 6cac8518), and `role`.
- Propagate the `Result` with `?`.

---

**`apm-core/src/init.rs` changes**

In the `apm init` default config template string (the `[worker_profiles.spec_agent]` section), add `role = "spec-writer"`:
```toml
[worker_profiles.spec_agent]
command = "claude"
args = ["--print"]
instructions = ".apm/apm.spec-writer.md"
role = "spec-writer"
role_prefix = "You are a Spec-Writer agent assigned to ticket #<id>."
```
(`impl_agent` is left without a `role` field — its default of `"worker"` is correct.)

---

**`apm-core/src/validate.rs` changes**

In the existing config validation pass, add a check for `config.workers.instructions`:
```rust
if let Some(ref path) = config.workers.instructions {
    if !root.join(path).exists() {
        errors.push(format!(
            "config: [workers].instructions — file not found: {path}"
        ));
    }
}
```

---

**Tests in `apm-core/src/start.rs` `#[cfg(test)]`**

Update the three existing `resolve_system_prompt_*` tests to match the new signature (add `workers`, `agent`, `role` args; unwrap the Result). Then add:

- `resolve_system_prompt_uses_workers_instructions_when_no_profile` — `WorkersConfig` with `instructions = Some(path)`; file exists; no profile → returns that content.
- `resolve_system_prompt_uses_per_agent_file` — no profile, no workers.instructions, `.apm/agents/claude/apm.worker.md` exists → returns its content.
- `resolve_system_prompt_falls_back_to_builtin_default` — no overrides, no per-agent project file, agent="claude", role="worker" → returns `CLAUDE_WORKER_DEFAULT`.
- `resolve_system_prompt_falls_back_to_builtin_spec_writer` — same but role="spec-writer" → returns `CLAUDE_SPEC_WRITER_DEFAULT`.
- `resolve_system_prompt_errors_for_unknown_agent` — no overrides, no per-agent project file, agent="custom-bot" → returns `Err`.
- `resolve_system_prompt_profile_instructions_missing_file_is_error` — profile.instructions set but file absent → returns `Err` (not silently falls through).
- `resolve_system_prompt_backward_compat` — profile.instructions = ".apm/apm.worker.md", file exists → works, confirming no regression for existing projects.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:42Z | groomed | in_design | philippepascal |
| 2026-04-30T21:50Z | in_design | specd | claude-0430-2142-eea0 |
| 2026-05-01T17:38Z | specd | ready | philippepascal |
| 2026-05-01T19:20Z | ready | in_progress | philippepascal |