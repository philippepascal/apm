+++
id = "992d816e"
title = "apm sync hint wrong epic to close"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/992d816e-apm-sync-hint-wrong-epic-to-close"
created_at = "2026-06-03T02:27:42.503993Z"
updated_at = "2026-06-03T06:32:17.623247Z"
+++

## Spec

### Problem

➜  syn git:(main) apm sync
sync: 22 ticket branches visible

Tickets ready to close:
  #288b434c  syn-server UI: invite link flow (server operator and client acceptance)  (branch merged into target)
  #3a2a9a09  syn-server UI: audit log screen  (branch merged into target)
  #4619bdbd  syn-server UI: tenant onboarding screen  (branch merged into target)
  #46b89e25  syn-server UI: Google sign-in for admin authentication  (branch merged into target)
  #4d9ff76e  syn-server UI: entitlement management screen  (branch merged into target)
  #6733fc52  syn-server UI: React and Vite scaffold  (branch merged into target)

Close all? [y/N] y
288b434c: implemented → closed
3a2a9a09: implemented → closed
4619bdbd: implemented → closed
46b89e25: implemented → closed
4d9ff76e: implemented → closed
6733fc52: implemented → closed

Epics ready to close (apm epic close <id>):
  25ae8e6c  Aws Transfer Family Adapter
  364a3bd0  Syn Client Bindings
  b8683407  Syn Test
  d9989d21  Release Pipeline
  f2b57ba1  Sftpgo Adapter
➜  syn git:(main) apm epic list
25ae8e6c [in_progress ] Aws Transfer Family Adapter              2 new                          ↓1968 clean
364a3bd0 [in_progress ] Syn Client Bindings                      2 new                          ↓1968 clean
5ca89700 [done        ] Syn Server Ui                            6 closed                       up to date
b8683407 [in_progress ] Syn Test                                 5 new                          ↓1968 clean
d9989d21 [in_progress ] Release Pipeline                         3 new                          ↓1968 clean
f2b57ba1 [in_progress ] Sftpgo Adapter                           1 new                          ↓1968 clean

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
| 2026-06-03T02:27Z | — | new | philippepascal |
| 2026-06-03T06:32Z | new | groomed | philippepascal |
