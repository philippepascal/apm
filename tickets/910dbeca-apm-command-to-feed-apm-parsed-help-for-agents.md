+++
id = "910dbeca"
title = "apm command to feed apm parsed help for agents (piphi)"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/910dbeca-apm-command-to-feed-apm-parsed-help-for-"
created_at = "2026-05-13T00:52:51.102305Z"
updated_at = "2026-05-14T06:30:00.000000Z"
agent = "pi"
+++

## Spec

### Problem

currently agents load markdown files that are static to learn how to use apm. instead apm needs a special command similar to help but specialized for agents. it may be a subcommand of apm help. it may have subcommand for every other apm commands. it needs to be very precise to improve agent understanding of apm commands, and very compact as it will be used often by agents.

### Acceptance criteria

- [ ] `apm help --agent` outputs a concise description of the `--agent` subcommand option and available agent profiles
- [ ] `apm help --agent <agent-name>` outputs agent-specific instructions for `piphi` including wrapper requirements and permissions
- [ ] `apm help <command> --agent` outputs command-specific guidance for agents for all available apm commands
- [ ] Help output excludes internal implementation details such as file paths, config fields descriptions, and unrelated code comments
- [ ] Help output includes error codes, expected inputs, and agent-specific error handling
- [ ] Output is deterministic and identical across multiple runs for the same inputs
- [ ] Output size stays under ~5KB per command description to enable rapid agent consumption
- [ ] `apm help --agent list` shows all available agent profiles registered in the system

### Out of scope

- Implementing agent runtime execution or workflow logic
- Adding interactive agent chat or command suggestions
- Providing full markdown documentation for human users
- Supporting all apm commands - only essential commands for agent workflow

### Approach

#### Command structure

Create a new subcommand `apm help --agent [COMMAND | [LIST]]` with these options:

- `apm help --agent` - general agent help overview
- `apm help --agent list` - list all available agent profiles
- `apm help --agent <name>` - show profile-specific instructions
- `apm help <COMMAND> --agent` - command-specific agent guidance

#### Implementation steps

1. Parse frontmatter `agent` field from ticket metadata to determine agent profile
2. Load agent-specific markdown from `.apm/agents/<agent-name>/instruction.md`
3. Parse command-specific help from `apm/src/main.rs` or generated docs
4. Filter content to remove implementation details, config schema docs, and internal notes
5. Format output with: header, purpose, requirements, flags, outputs, errors
6. Write to stdout in compact plain text format
7. Cache output in `cache/agent-help` for repeated calls

#### Data flow

```
agentic workflow:
apm help --agent list --> .apm/agents/*/instruction.md
apm help --agent piphi --> piphi/procedures.md
apm help <cmd> --agent --> apm/src/cmd/<cmd>.rs + agent profile
```

#### File structure

- `.apm/agents/piphi/instruction.md` - piphi-specific agent info
- `.apm/agents/piphi/procedures.md` - piphi procedure documentation
- `cache/agent-help` - cached help output

### Open questions

- None

### Amendment requests

- [ ] 

### Code review

### History

| When | From | To | By |
|------|-----|----|-----|
| 2026-05-13T00:52Z | — | new | philippe |
| 2026-05-14T06:08Z | new | groomed | philippe |
| 2026-05-14T06:09Z | groomed | in_design | philippe |
| 2026-05-14T06:25Z | in_design | groomed | philippe |
| 2026-05-14T06:28Z | groomed | in_design | philippe |
| 2026-05-14T06:30Z | in_design | specd | agent |
