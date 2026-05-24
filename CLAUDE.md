@.apm/agents/default/apm.main-agent.md

# Claude Code — APM Project

@.apm/project.md
@.apm/agents/claude/apm.main-agent.md
@.apm/agents/claude/style.md

## Style rules

@.apm/agents/claude/style.md contains opt-in output-style rules for this session. On startup:
- Read `.apm/agents/claude/style.md` and identify every rule marked `[x]` in `## Conversation`
- Apply those rules to your own replies for the entire session
- Rules marked `[ ]` are inactive — do not apply or reference them
- When spawning subagents via the Agent tool, prepend the text of each active `## Conversation` rule to the subagent prompt

## Commits

- Imperative mood, present tense: "Add X", "Fix Y", "Refactor Z"
- First line ≤ 72 chars
- Don't add: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
- Do not amend published commits — create new ones

## Merging

`apm state <id> implemented` handles push, PR creation, and merging based on the workflow config. Do not push branches or open PRs manually.

## Code style

- No unnecessary abstractions — solve the specific problem, not the general one
- No docstrings, comments, or type annotations on code you didn't change
- No backwards-compat shims — if something is unused, delete it
- Prefer editing existing files over creating new ones
- Do not add error handling for cases that can't happen

## Tests

- Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
- Integration tests in `apm/tests/integration.rs` — use temp git repos, no fixture files needed
- Run `cargo test --workspace` — all tests must pass before calling `apm state <id> implemented`

## Performance

- No premature optimisation — correctness and simplicity first
- Do not add caching, indexes, or in-memory structures unless a measured bottleneck exists
- Reading from git branch blobs is fast enough at ticket scale; filesystem caches are not needed

## State transitions require explicit instruction

Never transition a ticket's state unless the user explicitly asks you to.
Reviewing a ticket and recommending a verdict ("ready", "ammend", "close")
is not authorization to act on that verdict. Wait for the user to say
"move X to ready" or "close X" before running `apm state`.

This applies even when the user says "review and ammend" — only the
amendment is authorized, not any other transitions mentioned in the review.

This also applies to corrective actions: if a previous transition was a
mistake, do not run `apm state` to revert it. Report the problem and let
the user decide how to fix it.

## Things to avoid

- Do not run `rm -rf` or destructive git commands without confirmation
- Do not push `--force` to `main`
- Do not commit `.env` files or credentials
- Do not open issues, post comments, or send messages without being asked
