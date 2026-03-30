+++
id = 63
title = "apm sync does not detect squash-merged branches"
state = "in_progress"
priority = 1
effort = 0
risk = 0
author = "philippepascal"
agent = "claude-0329-1430-main"
branch = "ticket/0063-apm-sync-does-not-detect-squash-merged-b"
created_at = "2026-03-29T22:50:59.530523Z"
updated_at = "2026-03-30T01:08:31.191681Z"
+++

## Spec

### Problem

git branch --merged only detects regular merges. Squash-merged PRs leave the branch tip as a non-ancestor of main, so merged_into_main() in git.rs misses them. apm sync therefore does not transition squash-merged tickets to accepted.

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T22:50Z | — | new | philippepascal |
| 2026-03-30T01:08Z | new | in_progress | claude-0329-1430-main |
