+++
id = "e04e1b3f"
title = "Revise apm-demo creation script for mock worker support"
state = "in_design"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e04e1b3f-revise-apm-demo-creation-script-for-mock"
created_at = "2026-05-04T16:48:32.146018Z"
updated_at = "2026-05-04T17:28:58.129371Z"
epic = "65af2998"
target_branch = "epic/65af2998-apm-demo-enhancements"
+++

## Spec

### Problem

The `create-demo.sh` script always writes `command = "claude"` and `args = ["--print"]` into the `[workers]` block of the generated `.apm/config.toml`. Anyone who runs `apm work` against the resulting demo must have a live Claude CLI session, which is a barrier for documentation, CI, and onboarding.

The sibling ticket 295ff9ba ("Add mock_happy demo script for GIF recording") depends on this ticket because it needs a way to create a demo repo that uses the `mock-happy` built-in wrapper instead. `mock-happy` processes tickets deterministically and instantly without Claude — ideal for recording a repeatable GIF of the APM workflow. The creation script must be extended to support this use case while leaving the existing Claude-based default intact.

### Acceptance criteria

- [ ] `create-demo.sh --mock` produces a demo repo whose `.apm/config.toml` `[workers]` block contains `command = "mock-happy"` with no `args` field
- [ ] `create-demo.sh` with no flags produces a demo repo whose `[workers]` block contains `command = "claude"` and `args = ["--print"]` (existing behaviour unchanged)
- [ ] `create-demo.sh --mock` runs to completion without error on a clean GitHub account that has `gh`, `apm`, and internet access
- [ ] Passing an unrecognised flag to `create-demo.sh` prints an error message and exits non-zero

### Out of scope

- Changing the tickets, epics, or specs written by the script
- Changes to `src/main.rs` or `Cargo.toml`
- Changes to the README written by the script
- A `--mock` variant of the GitHub repo name or description
- Validating that `mock-happy` is available at create time (it is a built-in — if `apm` is present, `mock-happy` is present)

### Approach

All changes are in `scripts/create-demo.sh`.

#### Flag parsing

After `set -euo pipefail` (line 21), add:

```bash
MOCK_MODE=false
for arg in "$@"; do
  case "$arg" in
    --mock) MOCK_MODE=true ;;
    *) echo "ERROR: unknown flag: $arg"; exit 1 ;;
  esac
done
```

#### config.toml generation (step 4)

The current step 4 writes `.apm/config.toml` as one single-quoted heredoc (`<< 'APM_CONFIG'`). Split it into three parts so the `[workers]` stanza can be written conditionally.

**Part 1** — replace the existing `cat > .apm/config.toml << 'APM_CONFIG'` block with a heredoc that stops before `[workers]`:

```bash
cat > .apm/config.toml << 'APM_CONFIG_TOP'
[project]
name = "jot"
description = "A minimal CLI notes tool"
default_branch = "main"
collaborators = []

[tickets]
dir = "tickets"

[worktrees]
dir = "../jot--worktrees"
agent_dirs = [".claude", ".cursor", ".windsurf"]

[agents]
max_concurrent = 3
instructions = ".apm/agents.md"

APM_CONFIG_TOP
```

**Part 2** — append the workers stanza conditionally:

```bash
if "$MOCK_MODE"; then
  cat >> .apm/config.toml << 'APM_WORKERS'
[workers]
command = "mock-happy"

APM_WORKERS
else
  cat >> .apm/config.toml << 'APM_WORKERS'
[workers]
command = "claude"
args = ["--print"]

APM_WORKERS
fi
```

**Part 3** — append the logging section (was the tail of the original heredoc):

```bash
cat >> .apm/config.toml << 'APM_CONFIG_TAIL'
[logging]
enabled = false
file = "~/.local/state/apm/jot.log"
APM_CONFIG_TAIL
```

No other parts of the script change. The rest of step 4 (`apm init --no-claude`) and all subsequent steps are unaffected.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:50Z | groomed | in_design | philippepascal |
| 2026-05-04T16:55Z | in_design | specd | claude-0504-1650-e758 |
| 2026-05-04T17:26Z | specd | ammend | philippepascal |
| 2026-05-04T17:27Z | ammend | in_design | philippepascal |