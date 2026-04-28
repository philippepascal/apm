# APM MCP Server — Design Spec

## Problem

Agents interacting with `apm` today go through Bash. Claude Code's permission system matches Bash command strings by prefix, which forces an extensive set of "shell discipline" rules in `.apm/agents.md`:

- No `&&`, `&`, `$()` (compound forms defeat allow-list matching or trip security checks)
- No heredocs (bypass allow list)
- No `cd && git` (trips bare-repository-attack check) — must use `git -C`
- `apm spec --set "$(cat ...)"` is forbidden; agents must `Write` to a temp file then call `apm spec --set-file <path>`

These rules exist solely because of the matching mechanism, not because they reflect anything intrinsic about how `apm` should be used. They cost agent attention every session and produce friction that scales with how much specs and state get touched.

Two structural problems compound this:

1. **Spec writing is a forced two-step.** Every section requires a `Write` (to a temp file) plus a Bash call. With four required sections per spec, plus amendments and open questions, this is the dominant flow.
2. **Worktree-isolated subagents can't use apm at all.** Subagents launched with `isolation:"worktree"` don't inherit `.claude/settings.json`, so Bash on `apm--worktrees/` paths is denied and Write/Edit is blocked. The isolation feature is unusable for ticket work today.

A native MCP server for `apm` removes all of this in one move: typed tool calls instead of string-prefix matching, structured JSON returns instead of stdout parsing, and a configuration mechanism (project-scoped MCP discovery) that survives subagent isolation.

## Goals

- Eliminate the "Shell discipline" section of `agents.md` for apm operations
- Make spec writing a single typed call per section (no temp file)
- Enable apm use from worktree-isolated subagents
- Return structured JSON for every operation; remove stdout parsing
- Single allow-list entry (`mcp__apm__*`) replaces the ~20 `Bash(apm * ...)` patterns

## Non-goals

- Replace the `apm` CLI. Engineers use it directly; spawn workers shell out to `claude`; both must keep working.
- Migrate existing apm projects with in-flight ticket branches. `apm init` is run on a clean repo before any ticket branches exist, so no fallback / synthesize-on-worktree logic is needed.
- Multi-session shared state. Each Claude session spawns its own `apm mcp serve` over stdio; no HTTP transport, no shared daemon.

## Approach

### 1. Ship the MCP server as a subcommand

New subcommand `apm mcp serve` that speaks MCP over stdio. Same Rust binary, same dependency surface, no separate runtime. If `apm` is on PATH (precondition for everything else), the server is reachable.

Each invocation opens its own git/sqlite handles, like the CLI does today. No global state, no daemon.

### 2. `apm init` writes `.mcp.json`

`apm init` creates or merges into `.mcp.json` at the repo root:

```json
{
  "mcpServers": {
    "apm": { "command": "apm", "args": ["mcp", "serve"] }
  }
}
```

Claude Code auto-discovers `.mcp.json` from the project root. Because the file is checked in on `main`, every ticket branch cut from `main` inherits it, and every worktree has it. Workers spawned via `claude` in their worktree pick it up on startup; engineers running `claude` interactively get the same.

### 3. `apm init` also updates `.claude/settings.json`

Adds `mcp__apm__*` to the allow list. Replaces the existing per-command `Bash(apm * ...)` patterns. Checked in, shared across the team.

`.claude/settings.local.json` is **not** touched — that file is gitignored and contributor-specific.

### 4. Init behavior is idempotent and merge-safe

- Re-running `apm init` on an initialized project is a no-op for entries that already match
- `.mcp.json` may already contain other MCP servers (filesystem, github, etc.) — merge under `mcpServers.apm`, never clobber
- `.claude/settings.json` may already have a permissions block — merge into the allow list, never clobber

### 5. `apm start --spawn` passes `--mcp-config` explicitly

Belt-and-braces. The spawner already constructs the `claude` invocation; adding `--mcp-config <path-to-checked-in-.mcp.json>` makes the wiring explicit and removes any dependency on Claude Code's auto-discovery resolving correctly inside a worktree.

## Tool surface

Initial set, named `apm_<verb>`:

| Tool | Replaces |
|---|---|
| `apm_sync` | `apm sync` |
| `apm_next` | `apm next --json` |
| `apm_list` | `apm list --state <s>` |
| `apm_show` | `apm show <id>` |
| `apm_state` | `apm state <id> <target>` (parameterized, not one tool per transition) |
| `apm_spec_set` | `apm spec <id> --section X --set <content>` |
| `apm_spec_mark` | `apm spec <id> --section X --mark <item>` |
| `apm_spec_get` | `apm spec <id> --section X --get` |
| `apm_set` | `apm set <id> <field> <value>` (effort, risk, etc.) |
| `apm_assign` | `apm assign <id> <user>` |
| `apm_new` | `apm new --no-edit --context <ctx> <title>` (and `--side-note`) |
| `apm_start` | `apm start <id>` and `apm start --next` (with `spawn: bool`, `permissionless: bool`) |

`apm_state` is parameterized (id + target state) rather than one tool per transition. Schema validation rejects illegal targets; fewer tools to maintain and allow-list.

The `apm_spec_*` tools take section content as a string parameter directly — no temp file dance. This is the largest concrete ergonomics win.

## Tool naming is a stable surface

Once `mcp__apm__state` is allow-listed across users' machines, renaming forces every contributor to re-grant. Treat the tool names as more stable than CLI flags: pick carefully on the first cut, version with care after.

## Configuration

### `.mcp.json` (checked in)

```json
{
  "mcpServers": {
    "apm": { "command": "apm", "args": ["mcp", "serve"] }
  }
}
```

### `.claude/settings.json` (checked in)

Add to the allow list:

```json
{
  "permissions": {
    "allow": ["mcp__apm__*"]
  }
}
```

The first `apm mcp serve` invocation in a fresh checkout still triggers Claude Code's per-machine MCP server-trust prompt. One-time per contributor, not per session — flag this in the `apm init` output so users aren't surprised.

## Out of scope

- Migration for existing projects with pre-existing ticket branches. `apm init` is run on a clean repo.
- Removing or renaming any existing `apm` CLI command.
- HTTP / shared-daemon MCP transport. Stdio per session is sufficient.
- Exposing `apm start --spawn` through MCP. The CLI invocation stays — it's the harness's job to launch workers, not a tool's.

## Open questions

- Does Claude Code's `--mcp-config <path>` flag accept a path argument that overrides project discovery? Verify before relying on the belt-and-braces step in §5.
- For `apm_state`, should the response include side-effect metadata (worktree path on `in_design`/`start`, PR URL on `implemented`) as structured fields rather than free text? Yes, but the exact schema needs to be pinned.
- Should `apm_new` accept `context` as a string parameter only, or also `context_file` for very large contexts? String-only is simpler; revisit if a real case hits the parameter-size limit.
- Per-machine MCP server-trust prompt UX — is there a way to pre-approve via project config, or does it always require interactive consent on first use?

## Acceptance criteria

- [ ] `apm mcp serve` subcommand implements the full tool set above
- [ ] `apm init` creates `.mcp.json` if missing; merges if present
- [ ] `apm init` adds `mcp__apm__*` to `.claude/settings.json` allow list; merges if present
- [ ] `apm init` is idempotent — re-running on an initialized repo is a no-op
- [ ] `apm init` does not touch `.claude/settings.local.json`
- [ ] `apm start --spawn` passes `--mcp-config` to the spawned `claude` invocation
- [ ] Worktree-isolated subagents can call `apm_*` tools successfully
- [ ] All `apm_*` tools return structured JSON (typed schemas, not stdout text)
- [ ] `apm_spec_set` accepts content as a string parameter; no temp file required
- [ ] `agents.md` "Shell discipline" section is removed; replaced with a one-line note that apm is invoked via MCP tools
- [ ] `agents.md` examples updated to use tool calls instead of Bash invocations
