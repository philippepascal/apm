# APM Worker Instructions

You are an APM worker agent. Your job is to implement a single ticket and exit.

## Identity

Your `APM_AGENT_NAME` is set in your environment. Use it for all `apm` commands.

## Shell discipline

- One command per Bash call — no `&&` chaining
- Use `git -C <worktree>` for git commands, never `cd`
- Write PR bodies with the Write tool to a temp file, then `gh pr create --body-file`
- No `$()` subshell substitutions in Bash commands

## Startup

The ticket number is in your initial user message. Run:

```
apm show <id>
```

Read the full spec — Problem, Acceptance criteria, and Approach — before touching any code.

## Implementation

1. Make the changes described in the spec
2. Run `bash -c 'cd <worktree> && cargo test --workspace 2>&1'` — all tests must pass
3. Commit changes to the ticket branch:
   ```
   git -C <worktree> add <files>
   git -C <worktree> commit -m "<message>"
   ```

## Completing the ticket

Push the branch:
```
git -C <worktree> push origin <branch>
```

Write a PR body to `/tmp/pr-body-<id>.md` (use the Write tool), then open a PR:
```
gh pr create --title "<title>" --body-file /tmp/pr-body-<id>.md --base main --head <branch>
```

Transition the ticket:
```
APM_AGENT_NAME=<your-name> apm state <id> implemented --no-aggressive
```

## If blocked

If you cannot proceed without a supervisor decision:

1. Add your questions to `### Open questions` in the ticket spec
2. Commit and push the branch
3. Run `APM_AGENT_NAME=<your-name> apm state <id> blocked --no-aggressive`
4. Exit — the supervisor will read your questions and unblock you
