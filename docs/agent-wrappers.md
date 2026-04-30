# Agent wrappers

This is the design spec for APM's agent-wrapper architecture: how APM
invokes coding agents (Claude, Codex, Aider, mocks, etc.) via a uniform
contract, so the binary doesn't hardcode any specific agent's CLI shape.

## Why

Today APM hardcodes Claude-specific flags (`--print`,
`--output-format=stream-json`, `--verbose`, `--system-prompt`,
`--dangerously-skip-permissions`) and assumes Claude's positional-arg
input shape and JSONL `stream-json` output schema. Moving the flag *names*
into config (the previous proposal at ticket `163e0ee3`) would only paper
over the surface — argument order, input mechanism, and output format are
all still Claude-shaped. Agents like Aider (uses `--message`), OpenAI
Codex (different invocation), Gemini CLI (different output) cannot be
slotted in without code changes.

The wrapper architecture defines APM's contract once and pushes
agent-specific translation into a small adapter (the wrapper). APM stays
agnostic; wrappers do the work.

## Naming

- **Wrapper / agent wrapper** — a small adapter (script or built-in) that
  knows how to invoke one specific coding agent and translate its output.
- **`.apm/agents/<name>/`** — the directory for project-defined wrappers.
  Replaces the older "workers" terminology, which conflated process
  identity (a running worker) with adapter identity (the agent it wraps).
- **`.apm/agents.md`** — the project-wide agent conventions file
  (unchanged; coexists with the new directory at the same path level —
  `agents.md` is a file, `agents/` is a directory).
- **`[workers]` config section** — kept (changing the section name would
  break every existing repo). Its contents change: instead of `command`,
  `args`, `model` it now holds `agent = "<name>"` and an `options` table.

## Overall design

```
                                APM
                                 │
                  config: agent = "claude"
                                 │
                                 ▼
              ┌──────────────────────────────┐
              │     wrapper resolution       │
              │ 1. ticket.frontmatter.agent  │
              │ 2. worker_profile.agent      │
              │ 3. [workers].agent           │
              └──────────────────────────────┘
                                 │
                                 ▼
              ┌──────────────────────────────┐
              │ Built-in (Rust) or script at │
              │ .apm/agents/<name>/wrapper.* │
              │ (script overrides built-in)  │
              └──────────────────────────────┘
                                 │
                       APM env vars + cwd
                                 │
                                 ▼
                         <wrapper invocation>
                                 │
                       agent-specific CLI call
                                 │
                                 ▼
                JSONL stream-json on stdout
                              (or piped through
                               a parser strategy)
```

APM picks a wrapper, sets a fixed contract via env vars + working
directory, exec's it, and captures stdout/stderr to `.apm-worker.log`.
Everything Claude-specific lives inside the Claude wrapper.

## The wrapper contract

A wrapper is anything (script or built-in) that:

- Reads its inputs from a fixed set of environment variables and working
  directory.
- Writes structured events to stdout in APM's canonical format.
- Returns 0 on success, non-zero on failure.

### Inputs

**Working directory**: APM `chdir`s to the ticket worktree before exec.
The wrapper can assume `pwd` is the worktree root.

**Environment variables** (APM sets these; wrapper reads them):

| Variable | Purpose | Always set? |
|---|---|---|
| `APM_AGENT_NAME` | Worker session name (e.g. `claude-0429-1430-a3f9`) | yes |
| `APM_TICKET_ID` | Ticket ID being worked | yes |
| `APM_TICKET_BRANCH` | Branch name (e.g. `ticket/abc123-foo`) | yes |
| `APM_TICKET_WORKTREE` | Absolute path to the ticket worktree | yes (== cwd) |
| `APM_SYSTEM_PROMPT_FILE` | Path to a temp file containing the role prompt | yes |
| `APM_USER_MESSAGE_FILE` | Path to a temp file containing the ticket content | yes |
| `APM_SKIP_PERMISSIONS` | `1` or `0` — pass through to underlying agent if applicable | yes |
| `APM_PROFILE` | Worker profile name (e.g. `spec_agent`, `impl_agent`) | yes |
| `APM_ROLE_PREFIX` | Profile's role prefix string (currently used by Claude) | when configured |
| `APM_OPT_<KEY>` | Each entry from `[workers.options]` (or profile override), uppercased | optional |
| `APM_WRAPPER_VERSION` | The wrapper-contract version APM was built against (currently `1`) | yes |

The wrapper inherits the spawning shell's environment too, so secrets
(`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.) flow through naturally.
APM does not store secrets in config.

**Why files for the prompts** (instead of env-var values): system prompts
and ticket content can be large and contain newlines, quotes, and other
shell-hostile content. Files avoid escaping issues and let wrappers
choose how to inject them (read into a flag, redirect to stdin, mount in
a container, etc.).

### Outputs

**Stdout**: APM's canonical event format — JSONL stream-json, the same
schema Claude emits with `--output-format=stream-json --verbose`. Each
line is a JSON object with at least:

```json
{"type": "...", "timestamp": "...", "..." }
```

Wrappers for agents that don't emit this format natively must translate
their agent's output before writing to stdout. The Claude built-in
passes through; mock wrappers synthesize events; a hypothetical Codex
wrapper would translate its agent's output via `jq`, a small helper, or
a parser strategy (see "Output parser strategy" below).

**Stderr**: also captured to `.apm-worker.log`. Wrappers may emit free-
form diagnostics, warnings, etc. APM doesn't structure or parse stderr.

**Exit code**: 0 = success (transition to the worker's terminal state
proceeds), non-zero = failure (worker is recorded as crashed; ticket
stays where it was). Mock wrappers can exit zero while still producing
non-success transitions, since the transition is driven by the wrapper's
`apm state` calls, not by exit code.

### What the wrapper is responsible for

- Invoking the underlying agent with the right CLI shape, input
  mechanism, and any wrapper-specific options from `APM_OPT_*`.
- Translating output into APM's canonical JSONL stream-json (or piping
  through a parser, see below).
- Propagating signals (SIGTERM/SIGINT) to its child agent so
  `apm workers --kill` works. Standard process-group handling.
- Cleaning up any temp files it creates beyond the ones APM provides.

### What APM is responsible for

- Setting cwd to the ticket worktree.
- Writing the system prompt and user message to temp files and pointing
  the wrapper at them via env vars.
- Capturing stdout/stderr to `.apm-worker.log`.
- Sending SIGTERM on cancellation.
- Removing the temp prompt and message files when the worker exits.

## Configuration

```toml
# Default agent for all workers (formerly: command, args, model)
[workers]
agent = "claude"
instructions = ".apm/apm.worker.md"   # optional override; defaults to .apm/agents/<name>/apm.worker.md

[workers.options]
model = "sonnet"
# any wrapper-specific options as string KV pairs

# Per-profile overrides (existing pattern, now with agent selection)
[worker_profiles.spec_agent]
agent = "claude"                       # optional; falls back to [workers].agent
instructions = ".apm/apm.spec-writer.md"
role_prefix = "You are a Spec-Writer agent assigned to ticket #<id>."

[worker_profiles.spec_agent.options]
model = "haiku"                        # cheaper for spec-writing

[worker_profiles.impl_agent]
agent = "claude"
instructions = ".apm/apm.worker.md"
role_prefix = "You are a Worker agent assigned to ticket #<id>."

[worker_profiles.impl_agent.options]
model = "opus"                         # more capable for implementation
```

### Options table

`[workers.options]` (and per-profile overrides) is a free-form
`String -> String` map exposed to the wrapper as `APM_OPT_<KEY>`
environment variables (key uppercased, dots/dashes converted to
underscores). APM does not interpret these — wrappers read whatever they
need.

This avoids APM having to know what options each wrapper supports.
Adding a new wrapper-specific knob (temperature, max-tokens, custom env
var name) does not require an APM change.

### Frontmatter override

A ticket can specify its own agent in frontmatter, overriding both the
profile and the global default. Two granularities:

```toml
+++
id = "abc12345"
title = "..."
state = "ready"

# Single override applied to every worker spawn for this ticket.
agent = "mock-happy"

# OR per-profile, so different phases of the ticket use different agents.
[agent_overrides]
spec_agent = "claude"        # use real Claude for spec writing
impl_agent = "mock-random"   # but mock the implementation phase
+++
```

Both fields are optional; either, both, or neither may be set.

**Resolution order** (per worker spawn, where `P` is the worker profile
that the workflow is dispatching — e.g. `spec_agent`, `impl_agent`):

1. `frontmatter.agent_overrides[P]` if present
2. `frontmatter.agent` if present
3. `[worker_profiles.<P>].agent` if present
4. `[workers].agent`

**Why per-profile and not per-transition.** Profiles are already the
natural unit: the workflow's `command:start` transitions declare a
`profile` (e.g. `profile = "spec_agent"` for the transition into
`in_design`, `profile = "impl_agent"` for the transition into
`in_progress`). The transition→profile mapping lives in `workflow.toml`;
the agent→profile mapping lives in config or frontmatter. Two clean
axes, no per-transition combinatorial explosion.

A finer-grained `{transition: agent}` map would handle edge cases the
two-tier model can't (e.g. "Aider for the final
`in_progress → implemented` attempt but Claude for an earlier
`in_progress → ammend` revision"). That's edge-case territory; if it
materializes, add it later as a contract v2 extension.

Use cases:
- Debug a stuck ticket with `mock-happy` to see if the harness is OK
- Use a specialized agent for one weird ticket (e.g. a frontend ticket
  where you want Claude with a different prompt)
- Force a specific agent for a regression test
- Mix agents per phase (spec with Claude, impl with Codex) for one ticket

## Transition outcomes

For mock wrappers (and other tooling) to know which transition is the
"happy path" from the worker's perspective, transitions carry an
explicit `outcome` field:

```toml
[[workflow.states.transitions]]
to         = "implemented"
trigger    = "manual"
completion = "merge"
on_failure = "merge_failed"
outcome    = "success"

[[workflow.states.transitions]]
to       = "ammend"
trigger  = "manual"
outcome  = "rejected"

[[workflow.states.transitions]]
to       = "blocked"
trigger  = "manual"
outcome  = "needs_input"

[[workflow.states.transitions]]
to       = "question"
trigger  = "manual"
outcome  = "needs_input"
```

**Vocabulary**: `success`, `needs_input`, `blocked`, `rejected`,
`cancelled`. Custom values are accepted (treated as non-success by
mocks and tooling); the five above are the recognized standard.

### Implicit defaults when `outcome` is omitted

The field is optional. APM infers a default per transition using these
rules, in order:

1. If the transition has a `completion` strategy set
   (`merge`, `pr_or_epic_merge`, etc.) → **`success`**. The transition
   actively attempts to land the work; success is its goal.
2. Else if the transition's target state has `terminal = true` →
   **`cancelled`**. End-of-life without landing work — could be by
   choice (`apm close`) or by force (`--force`). Distinct from
   `success` because no work is committed by this transition itself.
3. Else → **`needs_input`**. The worker can't proceed; some external
   action (supervisor decision, more questions, amendment) is needed.

These defaults match every transition in the shipped default workflow
without anyone having to write `outcome` explicitly. The field exists
so projects with custom states can override the inference, and so
tooling (mocks, validate, UI) has a stable, declared contract instead
of a rule-of-thumb.

### Consumers of `outcome`

- **`mock-happy`** picks the `success` transition from the current
  state.
- **`mock-sad`** picks any `outcome ≠ "success"` transition the
  current state can reach.
- **`mock-random`** picks any valid transition for the current state
  (success or otherwise).
- **`apm validate`** can warn if a profile would never reach a
  `success` outcome (dead-end workflow).
- **The supervisor UI** could colour transitions by outcome
  (success = green, blocked = yellow, rejected = red, etc.).

## Built-in wrappers

APM ships several wrappers compiled into the binary. Why built-in
instead of shell scripts shipped to `.apm/agents/<name>/wrapper.sh`?

- **Speed**: no extra fork into bash/python before reaching the agent.
- **No template drift**: when Claude changes a flag, APM updates one
  Rust function. Existing repos pick up the fix on the next APM upgrade.
  Shell-script templates copied at `apm init` time would be stale forever.
- **Mocks need real logic**: producing realistic fake transitions,
  random behavior, and validate-passing fake specs is awkward in bash.
- **Customization is still trivial**: a script at
  `.apm/agents/<name>/wrapper.sh` shadows the built-in entirely. Users
  who want to customize use the eject pattern (see "Custom wrappers").

The shipped built-ins:

### `claude` (default)

Wraps Anthropic's `claude` CLI. Invokes:
```
claude --print --output-format=stream-json --verbose \
       --system-prompt "$(<APM_SYSTEM_PROMPT_FILE)" \
       [--model APM_OPT_MODEL] \
       [--dangerously-skip-permissions when APM_SKIP_PERMISSIONS=1] \
       "$(<APM_USER_MESSAGE_FILE)"
```

Pass-through on stdout (already canonical format).

### `mock-happy`

Synthesizes a successful run with no real model call:
1. Writes placeholder content to all required spec sections via
   `apm spec --set` (Problem, Acceptance criteria, Out of scope,
   Approach) so the spec passes `--check`.
2. Sets `effort` and `risk` to default values (e.g. 1, 1).
3. Picks the transition with `outcome = "success"` from the ticket's
   current state and runs it. (Resolved via the implicit defaults
   described in "Transition outcomes" — typically the transition with
   a `completion` strategy.)
4. Emits canonical JSONL events for each action so logs look real.
5. Exits 0.

Use cases: testing the harness end-to-end without burning credits,
demos, CI smoke tests, spec-quality bar verification (does the
state machine accept a spec with all four sections?).

### `mock-sad`

Synthesizes a failure run:
1. Writes placeholder content to *some but not all* required sections
   (or writes content that fails `apm validate`).
2. Optionally writes a question to `### Open questions`.
3. Picks any transition where `outcome ≠ "success"` (typically
   `needs_input`, `blocked`, or `rejected`) and runs it. Selection
   from the eligible set is random and seedable via `APM_OPT_SEED`.
4. Exits 0 (the wrapper succeeded; the *ticket outcome* is unsuccessful).

Use cases: testing supervisor flows that handle stuck tickets, exercising
the `merge_failed` and `blocked` paths.

### `mock-random`

Like `mock-sad` but picks from *all* valid transitions for the ticket's
current state — any outcome including `success`. Same `APM_OPT_SEED`
support.

Use cases: chaos testing, fuzz-style verification that any state machine
path is recoverable.

### Candidates for built-in (not shipped initially)

- **`codex`** (OpenAI Codex CLI) — viable if its CLI surface is stable
  enough to wrap. Needs investigation. Could ship as built-in once
  validated against a real session.
- **`aider`** — popular, well-documented CLI. Wrapper would translate
  APM's contract into `aider --message` form and post-process its output
  into canonical JSONL.
- **`gemini-cli`** — Google's CLI; wrapper feasibility depends on
  output structure stability.

These should land as built-ins only after someone has used them against
real tickets and confirmed the translation works. Until then they live
as community-contributed scripts in `.apm/agents/<name>/wrapper.sh` form.

## Custom wrappers

Users can add wrappers per project at `.apm/agents/<name>/`. Layout:

```
.apm/agents/
  my-custom-agent/
    wrapper.sh           # invocation (or wrapper.py, wrapper.ts, …)
    apm.worker.md        # role-specific instructions for impl_agent
    apm.spec-writer.md   # role-specific instructions for spec_agent
    manifest.toml        # optional metadata (parser, version, …)
```

A custom wrapper at `.apm/agents/<name>/wrapper.<ext>` shadows any
built-in of the same name. APM's resolution order:

1. Project script: `.apm/agents/<name>/wrapper.{sh,py,js,ts,…}` (any
   executable file named `wrapper.*`)
2. Built-in registered in APM's binary

If neither is found for the configured agent, validate fails with a
clear error pointing at the missing wrapper.

### `manifest.toml` (optional)

```toml
[wrapper]
name = "my-custom-agent"
# Wrapper contract version this wrapper targets. APM checks compat.
contract_version = 1

# Output parser strategy. "canonical" = wrapper produces APM's JSONL.
# "external" = pipe stdout through the named binary first.
parser = "canonical"
# parser_command = "apm-output-parser-mything"  # only when parser = "external"
```

If `manifest.toml` is absent, APM assumes `contract_version = 1` and
`parser = "canonical"`.

### Skeleton command

```
apm agents new <name>
```

Creates `.apm/agents/<name>/` with:
- `wrapper.sh` — a runnable template that prints all `APM_*` env vars
  to stderr (so the user can see what's available) and produces a
  minimal valid JSONL event on stdout
- `apm.worker.md` — a copy of the project's current `apm.worker.md` so
  the user can adapt
- `apm.spec-writer.md` — same
- `manifest.toml` — defaults written explicitly so the user can edit

The skeleton documents the contract inline as comments in `wrapper.sh`.

### Other wrapper-related commands

- `apm agents list` — list available wrappers (built-in + project),
  marking which is currently configured. Useful for discovery.
- `apm agents test <name>` — run the wrapper against a synthetic ticket
  in a temp worktree, capture output, report on canonical-format
  compliance and exit code. Useful smoke test before assigning it to
  real work.
- `apm agents eject <name>` — write the built-in's source to
  `.apm/agents/<name>/wrapper.sh` so the user can customize. Sets the
  manifest's `parser` and `contract_version` to match the built-in's.

## Output parser strategy

APM's canonical format is JSONL stream-json (Claude's output schema, by
historical accident). Wrappers must produce this on stdout. Three
strategies for getting there:

1. **Native canonical (default)**: the underlying agent already emits
   the right format. Claude built-in is in this category.
2. **In-wrapper translation**: the wrapper translates its agent's
   output before writing to stdout. Typically a small `jq` or
   inline-script step. Suitable for agents whose output is structured
   but in a different shape.
3. **External parser**: a separate binary (`apm-output-parser-<X>`)
   takes the agent's raw output on stdin and emits canonical JSONL on
   stdout. The wrapper pipes through it. Declared in `manifest.toml` as
   `parser = "external"` with a `parser_command`. Suitable for agents
   whose output is wildly different (free-form prose, custom binary
   protocols, multi-stream).

The external-parser path keeps APM's binary lean: parsers can be
distributed as separate cargo crates (`apm-output-parser-aider`,
`apm-output-parser-codex`) and installed independently.

## Per-agent instructions

Each agent may need slightly different prompt conventions (Codex prefers
structured tags, Aider expects concise context, etc.). The instruction
files (`apm.worker.md`, `apm.spec-writer.md`) live per-agent:

```
.apm/agents/<name>/apm.worker.md
.apm/agents/<name>/apm.spec-writer.md
```

Resolution order (highest priority first):
1. `[worker_profiles.<profile>].instructions` (project-level override)
2. `[workers].instructions` (project-level override, all profiles)
3. `.apm/agents/<configured-agent>/apm.<role>.md` (agent default)
4. APM's built-in default for that agent

This lets a project keep its existing project-wide prompts while
benefiting from per-agent defaults when a new agent is selected.

## Detailed considerations and edge cases

### Wrapper not found

If `agent = "foo"` and no built-in or `.apm/agents/foo/wrapper.*` exists,
`apm validate` fails with a config error: "agent 'foo' not found:
checked built-ins {claude, mock-happy, …} and `.apm/agents/foo/`."
Spawning fails the same way at runtime.

`apm validate --fix` cannot port a missing wrapper (no template to
copy from), so the user must fix the agent name or write a wrapper.

### Built-in shadowed by a non-functional script

If the user creates an empty or buggy `.apm/agents/claude/wrapper.sh`,
the project file wins and the built-in is hidden. `apm agents test
<name>` is the supported way to verify before relying on a custom
wrapper.

To revert to the built-in: delete the project script (or rename
`wrapper.sh.bak`).

### Concurrent workers

Each wrapper invocation is a separate process with its own temp prompt
and message files. No shared state. Safe to run multiple wrappers
simultaneously (one per ticket worktree). Wrappers must not write to
shared global paths.

### Cancellation

`apm workers --kill <id>` sends SIGTERM to the wrapper process group.
The wrapper is responsible for forwarding signals to the underlying
agent (default in most shells; built-ins handle this in Rust). On a
clean exit the temp files are removed; on SIGKILL or panic, APM cleans
them up at the next worker reaping.

### API keys / secrets

Wrappers inherit the spawning shell's environment, so users export their
keys (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.) per their normal
shell setup. APM does not read or store secrets. Document this in the
init flow.

### Logging and debug

`.apm-worker.log` captures stdout (canonical events) interleaved with
stderr (wrapper diagnostics). When debugging a misbehaving wrapper, the
log shows both the events the wrapper produced and any error messages
it emitted. Wrappers should be liberal with stderr in dev mode.

A `debug` built-in wrapper could be added: it prints all env vars to
stderr, the prompt and message file contents, then exits 0 with a
synthetic success event. Useful for verifying the wrapper-contract
plumbing without invoking any real agent. Effectively a no-op
mock-happy with extra introspection.

### Wrapper-contract versioning

The `APM_WRAPPER_VERSION` env var declares which contract version APM
is using (currently 1). The wrapper's `manifest.toml` declares which
versions it targets. APM checks compatibility at spawn time:

- Match → proceed.
- Wrapper's contract is older than APM's: APM either downgrades to the
  older contract (best effort) or warns and proceeds with the newer
  contract anyway.
- Wrapper's contract is newer than APM's: refuse to spawn with a clear
  upgrade-APM message.

For v1, all wrappers are at version 1 and this check is informational
only. The mechanism exists so future contract changes don't silently
break wrappers.

### `apm validate` checks

After this lands:
- The configured agent must exist (built-in or project script).
- For each profile, if `agent` is set, it must exist.
- For each ticket whose `frontmatter.agent` is set, the named agent
  must exist.
- The instruction-file resolution must succeed (the file actually
  exists at the resolved path).
- If a project-level wrapper is present, its `manifest.toml` (if any)
  parses cleanly and the declared `contract_version` is supported.

### Migration from current config

Existing repos have `[workers] command = "claude" args = […] model = "sonnet"`.

Migration strategy:
1. `apm init --migrate` (or a dedicated `apm agents migrate`) detects
   the legacy shape and rewrites it to `[workers] agent = "claude"
   [workers.options] model = "sonnet"`.
2. The legacy fields are accepted as a fallback for one APM major
   version: if `agent` is absent but `command` is present, APM
   synthesizes `agent = "claude"` (when `command = "claude"`) or fails
   loudly otherwise.
3. After the deprecation window, only the new shape is accepted.

`check_output_format_supported()` is removed — wrappers own their own
agent compat checks (or punt to runtime). APM's role narrows to
spawning + capturing.

### Mock-happy details

- Spec content: minimal but valid markdown for each required section.
  E.g. Problem = "Mock spec — no real problem analyzed.", Acceptance
  criteria = "- [ ] Mock criterion 1\n- [ ] Mock criterion 2", etc.
- Effort/risk: deterministic defaults (1, 1) so tests are reproducible.
- Target transition: the unique transition from the current state with
  `outcome = "success"` (resolved via the implicit-default rules in
  "Transition outcomes" — typically the transition with a `completion`
  strategy). If the state has zero or multiple `success` transitions,
  the wrapper exits non-zero with a clear diagnostic (a workflow with
  ambiguous success paths is a config error worth surfacing).
- `apm state` calls happen via shelling out to the same `apm` binary
  the wrapper was invoked from (no special internal API).
- Emits 1-2 fake JSONL `tool_use` events for visual realism in logs.

### Mock-sad / mock-random determinism

- `mock-sad` chooses uniformly from transitions where
  `outcome ≠ "success"`, restricted to transitions valid from the
  current state. Deterministic by default; can be seeded via
  `APM_OPT_SEED` for reproducible test runs.
- `mock-random` chooses uniformly from *all* valid transitions for the
  current state, regardless of outcome. Same seed env var.
- If the eligible transition set is empty (no non-success transitions
  available, in mock-sad's case), the wrapper exits non-zero with a
  diagnostic — the workflow doesn't allow the requested simulation.

### REPL / multi-turn agents

Some agents (Aider in interactive mode, future REPL-style tools) want
to maintain state across "turns". The wrapper contract is one-shot per
spawn — the wrapper invocation begins, does its work, ends. Users who
want REPL behavior must write a wrapper that internally spans multiple
turns within a single spawn (using temp files, sockets, or whatever the
underlying agent supports).

APM doesn't model multi-turn at the contract level. Adding it would be
a contract version 2 change.

## Summary of changes from today

| Today | After |
|---|---|
| `[workers] command/args/model` triplet hardcoded for Claude | `[workers] agent = "<name>"` plus `[workers.options]` |
| `start.rs` hardcodes Claude flags | `start.rs` picks a wrapper, sets env vars, exec's it |
| `.apm/apm.worker.md` and `.apm/apm.spec-writer.md` shared across whatever agent runs | Per-agent files at `.apm/agents/<name>/apm.*.md`, project-level overrides honored first |
| `check_output_format_supported()` smoke-tests Claude flag presence | Removed; wrappers own their own compat |
| Adding a new agent requires editing `start.rs` and re-shipping APM | Adding a new agent is one wrapper script in `.apm/agents/<name>/` |
| Mocks for testing don't exist | Built-in `mock-happy`, `mock-sad`, `mock-random`, optional `debug` |
| No way to override which agent handles a specific ticket | `frontmatter.agent` overrides workflow default |
| Output format is hardcoded JSONL stream-json | Same default, but `manifest.toml` can declare an external parser |

## Open questions

- Should APM ship a `claude.sh` reference script in addition to the Rust
  built-in, so users have a real example to copy when authoring their
  own wrappers? (Built-in is faster; reference script is more
  pedagogical. Could ship both: built-in is the runtime, the script is
  in `docs/examples/claude.sh` as documentation.)
- What's the minimum set of canonical event types a wrapper *must*
  emit? Today APM's parser is forgiving — accepts any JSONL line — but
  formal documentation of the canonical event vocabulary would help
  third-party wrapper authors. Worth a follow-up doc.
- Should `apm agents test` actually invoke the wrapper, or just
  validate its config and contract metadata? Both are useful at
  different phases of authoring.
- How does the wrapper architecture interact with the existing
  `apm work` daemon's spawn-cwd discipline (the worker must be cwd'd
  to the worktree, which `start.rs` enforces today)? Confirm the
  contract still encodes this clearly enough that wrapper authors
  don't accidentally break it.
