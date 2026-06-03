+++
id = "612ca2cb"
title = "apm epic close doesn't detect merged (locally) epic"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/612ca2cb-apm-epic-close-doesn-t-detect-merged-loc"
created_at = "2026-06-03T02:29:52.020160Z"
updated_at = "2026-06-03T06:32:21.601762Z"
+++

## Spec

### Problem

syn git:(main) apm epic submit --merge 5ca89700
Merged epic/5ca89700-syn-server-ui into branch.
➜  syn git:(main) git status
On branch main
Your branch is ahead of 'origin/main' by 251 commits.
  (use "git push" to publish your local commits)

nothing to commit, working tree clean
➜  syn git:(main) apm epic list
25ae8e6c [in_progress ] Aws Transfer Family Adapter              2 new                          ↓2219 clean
364a3bd0 [in_progress ] Syn Client Bindings                      2 new                          ↓2219 clean
5ca89700 [done        ] Syn Server Ui                            6 closed                       up to date
b8683407 [in_progress ] Syn Test                                 5 new                          ↓2219 clean
d9989d21 [in_progress ] Release Pipeline                         3 new                          ↓2219 clean
f2b57ba1 [in_progress ] Sftpgo Adapter                           1 new                          ↓2219 clean
➜  syn git:(main) apm epic close 5ca89700
Error: epic has 251 commit(s) not yet in main. Use --force to delete unconditionally.
➜  syn git:(main)

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
| 2026-06-03T02:29Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
