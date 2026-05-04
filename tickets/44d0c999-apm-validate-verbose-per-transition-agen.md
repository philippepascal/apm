+++
id = "44d0c999"
title = "apm validate --verbose: per-transition agent resolution audit"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/44d0c999-apm-validate-verbose-per-transition-agen"
created_at = "2026-05-04T17:40:24.657468Z"
updated_at = "2026-05-04T17:43:53.175056Z"
epic = "5acea599"
target_branch = "epic/5acea599-flexible-agent-configuration"
depends_on = ["6803b88b"]
+++

## Spec

### Problem

After ticket 6803b88b lands, `instructions` and `role_prefix` can be set directly on each `command:start` transition in `workflow.toml`. Combined with the existing profile → workers → project-agent-file → built-in fallback chain, a spawn transition now resolves its instructions through up to five levels and its role prefix through three. `apm validate` already checks that referenced files exist and that profile names are valid, but it does not show *which value wins* at each level for a given transition. A project author adding a new spawn transition—or debugging why the wrong instructions file is loading—has no way to confirm the effective agent, instructions file, role prefix, and wrapper without reading source code or running a live spawn.\n\n`apm validate --verbose` closes this gap by appending a per-transition agent resolution audit to the normal validate output.

### Acceptance criteria

- [ ] `apm validate --verbose` is accepted without error on a valid project\n- [ ] Without `--verbose`, validate output is byte-for-byte identical to current behavior (no extra lines, no changed exit code)\n- [ ] The audit section lists exactly the transitions whose `trigger` equals `"command:start"`\n- [ ] For each spawn transition the text output shows: from-state ID, to-state ID, profile name (or none), resolved agent + source label, resolved instructions path or description + source label, resolved role prefix + source label, resolved wrapper\n- [ ] Source label for instructions is one of: `transition`, `profile:<name>`, `workers`, `project-agent-file`, `built-in`\n- [ ] Source label for role prefix is one of: `transition`, `profile:<name>`, `default`\n- [ ] Source label for agent is one of: `profile:<name>`, `workers`, `default`\n- [ ] When no `command:start` transitions exist, the audit section states "0 spawn transitions"\n- [ ] When a transition references a missing profile, the audit row shows "profile not found" for profile-dependent fields without panicking\n- [ ] `apm validate --verbose --json` adds an `"agent_resolution"` array to the JSON output; each element has `from_state`, `to_state`, `profile`, `agent`, `instructions`, `role_prefix`, `wrapper` keys; `agent`, `instructions`, and `role_prefix` each carry `value` and `source` subkeys\n- [ ] The code compiles and does not panic when `worker_profiles` is empty

### Out of scope

- Auditing non-`command:start` transitions (review, manual, close, approve, etc.)\n- Per-ticket frontmatter overrides (`agent`, `agent_overrides`) — those are ticket-level, not config-level\n- Changing what `apm validate` currently validates — this ticket only adds display\n- Changes to `resolve_system_prompt` or `agent_role_prefix` in `start.rs` beyond making `resolve_builtin_instructions` pub(crate)\n- Extending the audit to cover a `TransitionConfig.agent` field introduced by sibling ticket ed16b686 — that ticket can update the audit when it lands

### Approach

#### 1. CLI flag - apm/src/main.rs\n\nAdd verbose: bool to the Validate variant after no_aggressive (annotated with #[arg(long)] and doc string Show per-transition agent resolution audit). In the Commands::Validate match arm, pass verbose to cmd::validate::run().\n\n#### 2. Propagate through cmd layer - apm/src/cmd/validate.rs\n\nAdd verbose: bool as the last parameter to run(). At the end of run(), after all existing validate phases, if verbose is true: call apm_core::validate::audit_agent_resolution(&config, root); if json is also true merge the result into the JSON envelope as an agent_resolution key (section 6); otherwise print the text audit block (section 5). Run the audit even when validation errors exist.\n\n#### 3. Core data structures - apm-core/src/validate.rs\n\nAdd after existing imports:\n\n    #[derive(Debug, Serialize)]\n    pub struct FieldAudit { pub value: String, pub source: String }\n\n    #[derive(Debug, Serialize)]\n    pub struct TransitionAudit {\n        pub from_state: String, pub to_state: String,\n        pub profile: Option<String>,\n        pub agent: FieldAudit, pub instructions: FieldAudit,\n        pub role_prefix: FieldAudit, pub wrapper: String,\n    }\n\n#### 4. audit_agent_resolution - apm-core/src/validate.rs\n\nNew public function: pub fn audit_agent_resolution(config: &Config, root: &Path) -> Vec<TransitionAudit>\n\nFor each state in config.workflow.states, for each transition where trigger == "command:start":\n\n- **Profile lookup**: config.worker_profiles.get(transition.profile). If named but absent, mark profile-dependent fields as FieldAudit { value: "(profile not found)", source: "none" } -- do not panic.\n- **Agent**: check profile.agent -> source "profile:<name>"; else workers.agent -> source "workers"; else default "claude" -> source "default".\n- **Role** (not shown, used for path lookup only): profile.role.as_deref().unwrap_or("worker").\n- **Instructions** (Level 0 from 6803b88b first, then 1-4): (1) transition.instructions Some -> source "transition"; (2) profile.instructions Some -> source "profile:<name>"; (3) workers.instructions Some -> source "workers"; (4) .apm/agents/<agent>/apm.<role>.md exists on disk -> source "project-agent-file"; (5) crate::start::resolve_builtin_instructions(agent, role) is Some -> source "built-in", value "built-in:<agent>:<role>"; (6) none -> source "none", value "(unresolved)".\n- **Role prefix** (Level 0 first): (1) transition.role_prefix Some -> source "transition"; (2) profile.role_prefix Some -> source "profile:<name>"; (3) default -> source "default", value "You are a Worker agent assigned to ticket #<id>.".\n- **Wrapper**: call wrapper::resolve_wrapper(root, &agent) -- verify exact signature in the wrapper module. Format as "builtin:<agent>" for built-ins or the wrapper file path for custom.\n\n#### 5. Text output format - apm/src/cmd/validate.rs\n\nPrint after existing output:\n\n    Agent resolution audit -- 3 spawn transitions:\n\n      groomed -> in_design  [profile: spec_agent]\n        agent:        claude                                    (default)\n        instructions: .apm/agents/default/apm.spec-writer.md  (transition)\n        role prefix:  You are a Spec-Writer agent...           (transition)\n        wrapper:      builtin:claude\n\nPad the value column to the longest value in the block; place source in parens at a fixed offset. Truncate role prefix strings longer than 60 chars with an ellipsis. When zero spawn transitions, print "Agent resolution audit -- 0 spawn transitions" with no block body.\n\n#### 6. JSON envelope - apm/src/cmd/validate.rs\n\nAdd to the existing JSON output struct:\n\n    #[serde(skip_serializing_if = "Option::is_none")]\n    pub agent_resolution: Option<Vec<TransitionAudit>>,\n\nPopulate when verbose is true; leave None otherwise to preserve backward compatibility for JSON consumers.\n\n#### 7. Make resolve_builtin_instructions accessible - apm-core/src/start.rs\n\nChange fn resolve_builtin_instructions to pub(crate) fn resolve_builtin_instructions so validate.rs can call it without duplicating the match table. No other changes to start.rs.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T17:40Z | — | new | philippepascal |
| 2026-05-04T17:40Z | new | groomed | philippepascal |
| 2026-05-04T17:43Z | groomed | in_design | philippepascal |