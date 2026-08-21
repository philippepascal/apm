+++
id = "5297b67e"
title = "apm help and version should work even outside a repo"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5297b67e-apm-help-and-version-should-work-even-ou"
created_at = "2026-08-21T19:59:20.649883Z"
updated_at = "2026-08-21T20:06:15.470880Z"
+++

## Spec

### Problem

The top-level `--version`/`--help` flags already work outside a git repository, because clap intercepts and handles them inside `Cli::parse()` before any of apm's own code runs. But the explicit `apm version` and `apm help [topic]` subcommands do not: `apm/src/main.rs` unconditionally calls `repo_root()` (which shells out to `git rev-parse --show-toplevel`) before dispatching to any subcommand handler, with a single carve-out for the hidden `path-guard` command. Outside a git repository, `repo_root()` bails with "not inside a git repository", so `apm version` and `apm help` never get a chance to run — even though neither needs a repository at all. `cmd::version::run()` only prints compile-time constants, and `cmd::help::run()` only introspects the statically-built `clap::Command` tree returned by `Cli::command()`; neither touches `root`, `.apm/config.toml`, or any ticket data.

This is a rough edge for anyone evaluating apm for the first time (checking `apm version` or reading `apm help` before ever running `git init` or `apm init`), and for any script or CI step that wants to sanity-check the installed apm version from an arbitrary working directory.

### Acceptance criteria

- [ ] `apm version` run from a directory outside any git repository prints the version line and exits 0
- [ ] `apm help` (no topic) run from a directory outside any git repository prints the topic overview and exits 0
- [ ] `apm help commands` run from a directory outside any git repository prints the full command reference and exits 0
- [ ] `apm help config`, `apm help workflow`, and `apm help ticket` each run successfully (exit 0, non-empty output) from outside any git repository
- [ ] `apm help badtopic` run from outside any git repository still fails with the "unknown help topic" error and a non-zero exit code — it fails for that reason, not because a repository is missing
- [ ] `apm version` and `apm help [topic]` behave exactly as before when run from inside a git repository, whether or not apm is initialized there
- [ ] Every other subcommand (e.g. `apm list`, `apm show`, `apm init`) still fails with the existing "not inside a git repository" error when run outside a repository — this ticket does not widen that exemption

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
| 2026-08-21T19:59Z | — | new | philippepascal |
| 2026-08-21T19:59Z | new | groomed | philippepascal |
| 2026-08-21T20:06Z | groomed | in_design | philippepascal |