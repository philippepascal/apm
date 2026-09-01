+++
id = "fb30010b"
title = "wrong message when epic submit doesn't need to merge anything"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fb30010b-wrong-message-when-epic-submit-doesn-t-n"
created_at = "2026-09-01T00:59:43.049321Z"
updated_at = "2026-09-01T01:04:15.158538Z"
+++

## Spec

### Problem

syn git:(main) apm epic submit 565f --pr
Error: gh pr create failed: pull request create failed: GraphQL: No commits between main and epic/565fe172-manual-test-observations-consumer-folder (createPullRequest)
➜  syn git:(main) git merge epic/565fe172-manual-test-observations-consumer-folder
Already up to date.
➜  syn git:(main) apm epic submit 565f --merge
Error: merge conflict — resolve manually after checking out main, or use --pr to open a PR instead

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-09-01T00:59Z | — | new | philippepascal |
| 2026-09-01T01:04Z | new | groomed | philippepascal |
| 2026-09-01T01:04Z | groomed | in_design | philippepascal |
