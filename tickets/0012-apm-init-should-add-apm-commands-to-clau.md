+++
id = 12
title = "apm init should add apm commands to Claude allow list with user approval"
state = "specd"
priority = 5
effort = 4
risk = 3
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

Agents running in Claude Code need `apm` subcommands in the Claude allow list
to operate without prompting the user on every command. Today this is done
manually. `apm init` should offer to add the necessary entries to
`.claude/settings.json` so the setup is automatic and reproducible across clones.
User approval is required because this modifies Claude's permission model.

### Acceptance criteria

- [ ] `apm init` detects if `.claude/settings.json` exists in the repo root
- [ ] If it exists and is missing apm allow entries, `apm init` prompts: "Add apm commands to Claude allow list? [y/N]"
- [ ] On confirmation, the following patterns are added to `permissions.allow`:
  `apm sync*`, `apm next*`, `apm list*`, `apm show*`, `apm set *`, `apm state *`,
  `apm start *`, `apm take *`, `apm spec *`, `apm agents*`, `apm _hook *`,
  `apm verify*`, `apm new *`
- [ ] Each entry is wrapped as `"Bash(apm <pattern>)"`
- [ ] If entries are already present, the step is skipped silently
- [ ] If `.claude/settings.json` does not exist, the step is skipped silently (not everyone uses Claude Code)
- [ ] `apm init --no-claude` skips this step entirely without prompting

### Out of scope

- Global (`~/.claude/settings.json`) modifications — repo-local only
- Adding entries to `settings.local.json`
- Managing `ask` or `deny` lists

### Approach

In `cmd/init.rs`, add `update_claude_settings(root: &Path, skip: bool) -> Result<()>`:
1. If `skip` or no `.claude/settings.json`, return early
2. Parse JSON; find or create `permissions.allow` array
3. Check which `Bash(apm ...)` entries are missing
4. If none missing, return
5. Print the list of entries to be added; prompt `[y/N]`
6. On `y`: insert missing entries, write file back with preserved formatting
   (use `serde_json` for parse/write; indentation will normalize to 2-space)

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
