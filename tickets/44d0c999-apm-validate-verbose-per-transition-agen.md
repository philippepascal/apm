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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T17:40Z | — | new | philippepascal |
| 2026-05-04T17:40Z | new | groomed | philippepascal |
| 2026-05-04T17:43Z | groomed | in_design | philippepascal |