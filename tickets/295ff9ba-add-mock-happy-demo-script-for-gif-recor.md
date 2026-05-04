+++
id = "295ff9ba"
title = "Add mock_happy demo script for GIF recording"
state = "specd"
priority = 0
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/295ff9ba-add-mock-happy-demo-script-for-gif-recor"
created_at = "2026-05-04T16:48:42.740876Z"
updated_at = "2026-05-04T17:30:37.136818Z"
epic = "65af2998"
target_branch = "epic/65af2998-apm-demo-enhancements"
depends_on = ["e04e1b3f"]
+++

## Spec

### Problem

The APM project has no demo GIF showing `apm work` orchestrating mock workers across tickets. Producing one manually requires a live Claude session and a GitHub repository, making it slow, expensive, and non-reproducible. `mock-happy` is a built-in APM worker that completes tickets deterministically without any API credentials, making it ideal for scripted demos — but no script exists that drives a representative `apm work` session suitable for recording.

Without such a script, every attempt to produce the README GIF is a manual process: set up a repo, create tickets, wire up mock-happy, record, discard. The script that should encapsulate this setup does not exist.

### Acceptance criteria

- [ ] `scripts/record-demo.sh` is a new executable shell script committed to the APM repo
- [ ] Running the script without arguments creates a complete demo environment in a temp directory, requiring no GitHub account, no Claude CLI, and no API credentials
- [ ] The demo sequence runs `apm list` before processing, then `apm work` (which blocks until all workers complete in non-daemon mode), then `apm list` again
- [ ] At least 3 tickets are visible in the final `apm list` output having transitioned from `ready` to `implemented`
- [ ] The demo environment uses `mock-happy` as the configured worker (`command = "mock-happy"` in `config.toml`)
- [ ] Each key `apm` command is preceded by a printed `$ <command>` line so the recording looks like a realistic shell session
- [ ] The script accepts `--keep-dir` to suppress temp-directory cleanup on exit
- [ ] The script exits 0 on successful completion

### Out of scope

- VHS `.tape` files, asciinema configuration, or any recording-tool setup
- Actual GIF creation or upload — the script produces a reproducible session; the recording tool is the caller's concern
- Creating a GitHub repository (the demo uses a local bare-repo remote)
- Reusing or calling `create-demo.sh` — this script creates its own minimal project
- Changes to `create-demo.sh` (covered by ticket e04e1b3f)

### Approach

New file: `scripts/record-demo.sh` (chmod +x). No other files change.

#### Flag parsing and cleanup trap

After `set -euo pipefail`, parse flags and set up a cleanup trap:

- `KEEP_DIR=false` by default; `--keep-dir` sets it true
- Unknown flags print an error and exit 1
- `WORKDIR=$(mktemp -d)` for the entire session
- Unless `--keep-dir`, register an EXIT trap that removes `WORKDIR`

#### Local bare-repo remote

`apm work` calls `apm start --next --spawn` internally, which fetches and pushes branches. A local bare repository acts as a network-free remote:

- `git init --bare "$WORKDIR/jot.git"`
- `git clone "$WORKDIR/jot.git" "$WORKDIR/jot"`
- `cd "$WORKDIR/jot"` and set `user.email` / `user.name` in the local config

#### Minimal project setup

Write three files into the cloned repo before running `apm init`:

1. `Cargo.toml` — minimal `[package]` stanza, `name = "jot"`
2. `src/main.rs` — single `fn main` that prints `"jot"`
3. `.apm/config.toml` — full APM config; `[workers]` stanza uses `command = "mock-happy"`; `[worktrees] dir = "../jot--worktrees"` (sits inside `WORKDIR`)

Run `apm init --no-claude`, commit the initial files, and push to `origin main`.

#### Ticket creation

Create 4 tickets with `--no-edit --no-aggressive`. For the 3 tickets that will be processed:

- Fill all four required spec sections (`Problem`, `Acceptance criteria`, `Out of scope`, `Approach`) using `apm spec --section ... --set`
- Set `effort` and `risk` via `apm set`
- Force-advance to `ready` with `apm state --no-aggressive --force ready`

Leave the fourth ticket in `groomed` so `apm list` shows a mixed board.

Push all branches before the demo sequence: `git push origin --all`.

#### Demo sequence

Define a helper that prints the command with a `$ ` prefix before running it, with a brief sleep so a recorder can capture the prompt line:

```
run() — printf the command string, sleep 0.5s, then execute it
```

Steps:
1. `run apm list` — shows 4 tickets in their starting states
2. `sleep 1`
3. `run apm work` — dispatches `mock-happy` workers on the 3 `ready` tickets and blocks until all workers have completed (non-daemon mode exits when `no_more && workers.is_empty()`)
4. `sleep 1`
5. `run apm list` — shows the 3 processed tickets in `implemented` state

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:55Z | groomed | in_design | philippepascal |
| 2026-05-04T17:00Z | in_design | specd | claude-0504-1655-4030 |
| 2026-05-04T17:26Z | specd | ammend | philippepascal |
| 2026-05-04T17:29Z | ammend | in_design | philippepascal |
| 2026-05-04T17:30Z | in_design | specd | claude-0504-1729-b330 |
