# Claude Code — APM Project

@apm.agents.md

## Commits

- Imperative mood, present tense: "Add X", "Fix Y", "Refactor Z"
- First line ≤ 72 chars
- Don't add: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
- Do not amend published commits — create new ones

## Pull requests

- Title: mirrors the ticket title (kept short)
- Body must include:
  - Link: `Closes #<n>` (ticket number)
  - Brief summary of the approach (1–3 bullets)
  - Test plan (what was run, what to verify manually if anything)
- Do not push to `main` directly — always use a PR
- Do not merge without user approval

## Code style

- No unnecessary abstractions — solve the specific problem, not the general one
- No docstrings, comments, or type annotations on code you didn't change
- No backwards-compat shims — if something is unused, delete it
- Prefer editing existing files over creating new ones
- Do not add error handling for cases that can't happen

## Tests

- Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
- Integration tests in `apm/tests/integration.rs` — use temp git repos, no fixture files needed
- Run `cargo test --workspace` before opening a PR

## Things to avoid

- Do not run `rm -rf` or destructive git commands without confirmation
- Do not push `--force` to `main`
- Do not commit `.env` files or credentials
- Do not open issues, post comments, or send messages without being asked
