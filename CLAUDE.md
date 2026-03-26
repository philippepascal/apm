# Claude Code — APM Project

## Repo structure

Rust workspace — one crate to start, structured to grow:

- `apm-core/` — library: data model, config parsing, ticket storage, state machine
- `apm/` — CLI binary (thin wrapper over `apm-core`)
- `testdata/` — ticket fixtures for integration tests
- `initial_specs/` — design docs (SPEC.md, STATE-MACHINE.md, TICKET-SPEC.md, USECASES.md, etc.)

## Managing tasks

State labels: `new` → `question` → `specd` → `ready` → `in_progress` → `implemented` → `accepted` → `closed`
(`ammend` flags a spec needing revision before `ready`)

See `initial_specs/STATE-MACHINE.md` for the full workflow and transition schema.
See `initial_specs/TICKET-SPEC.md` for the ticket document format.

## Development workflow

1. Read the relevant spec files before implementing anything
2. Make the minimal change that satisfies the acceptance criteria
3. Add or update tests — all acceptance criteria should be covered
4. Run `cargo test --workspace` before opening a PR
5. All tests must pass before opening a PR

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

- Unit tests inline in each crate or in `crate/tests/`
- Integration tests use real ticket files in `testdata/`

## Things to avoid

- Do not run `rm -rf` or destructive git commands without confirmation
- Do not push `--force` to `main`
- Do not commit `.env` files or credentials
- Do not open issues, post comments, or send messages without being asked
