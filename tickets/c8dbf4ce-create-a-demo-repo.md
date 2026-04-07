+++
id = "c8dbf4ce"
title = "create a demo repo"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8dbf4ce-create-a-demo-repo"
created_at = "2026-04-07T17:01:04.559759Z"
updated_at = "2026-04-07T17:43:43.554278Z"
+++

## Spec

### Problem

APM has no standalone public demo that a new user can clone and explore without first building a project from scratch. The only way to currently "kick the tires" is to run `apm init` on a blank repo (no pre-existing tickets, no context) or wade through the actual APM source tickets (complex, hundreds of entries, opaque to outsiders).

A purpose-built demo repo solves this by giving new users a realistic, self-contained project they can clone and immediately explore. It provides a believable software project with a representative ticket backlog, so every APM command has something meaningful to act on.

The demo must cover the full feature surface: multiple ticket states, epics, cross-ticket dependencies, the `apm-server` web UI, and the README-driven onboarding flow. Without it, the "getting started" story for APM is fragile and requires significant upfront investment from the user.

### Acceptance criteria

- [ ] A public GitHub repository named `apm-demo` exists and is cloneable without authentication
- [ ] The repo contains a Rust CLI project that compiles with `cargo build` without errors
- [ ] Running the compiled binary (e.g. `./jot list`) produces output without panicking
- [ ] The repo contains a `.apm/config.toml` with project name, default branch, and merge strategy configured
- [ ] `apm list` run from the cloned repo shows tickets across at least 8 distinct states
- [ ] At least one epic exists and `apm epic list` shows it
- [ ] At least two tickets have `depends_on` set referencing other tickets in the repo
- [ ] At least one ticket is assigned to the epic (has `epic` field set)
- [ ] `apm show <id>` on a `closed` ticket shows a fully-populated spec (all four sections filled)
- [ ] `apm show <id>` on a ticket in `ammend` state shows a `### Amendment requests` section with at least one unchecked checkbox
- [ ] `apm show <id>` on a ticket in `question` state shows a `### Open questions` section with a pending question
- [ ] `apm next` returns a ticket (the highest-priority actionable one)
- [ ] The README contains a "Getting started" section that covers: cloning, verifying binaries, `apm list`, `apm show`, `apm next`, `apm-server`
- [ ] The README explains the fictional project context so the ticket backlog makes narrative sense
- [ ] All ticket states from the default workflow appear at least once across the ticket set: `new`, `groomed`, `in_design`, `specd`, `question`, `ammend`, `ready`, `in_progress`, `blocked`, `implemented`, `closed`

### Out of scope

- Building or publishing a binary release for the demo CLI
- CI/CD configuration (GitHub Actions, etc.) for the demo repo
- Automated testing of the demo repo itself
- apm-server deployment or hosting of the web UI
- Using `apm register` / `apm sessions` / `apm revoke` (server auth commands) — those require a running server instance and are mentioned in the README as a next step, not demonstrated
- Creating tickets for every single possible state combination — a representative subset is sufficient
- Keeping the demo repo in sync with future APM feature changes (out-of-scope for this ticket; a separate maintenance process is needed)
- The Rust CLI being a genuinely useful piece of software — it only needs to be plausible and compilable

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:01Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |