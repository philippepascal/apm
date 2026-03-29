# APM Implementation Plan

Working document. Derived from TICKET-LIFECYCLE.md.

---

## Immediate actions (no ticket — do before any workers start)

- [ ] Update `apm.agents.md`: remove `in_progress → ready` (gone from state machine);
  replace with `in_progress → blocked` for abandonment. Agents hitting the old
  pattern will get a hard error.

---

## Ordering and priorities

Priority scale: higher number = picked first by `apm next`.

### Priority 5 — Foundation (blocks everything)

| # | Ticket | Why first |
|---|--------|-----------|
| 53 | Config schema: `ticket.sections`, state `instructions`, transition `completion`/`focus_section`/`context_section` | All Phase C commands read these config fields. Nothing else can be implemented correctly without them. |

### Priority 4 — Core commands (depend on #53; block agent infrastructure)

| # | Ticket | Notes |
|---|--------|-------|
| 41 | Configurable merge strategy → `completion` on transitions | Spec needs amendment first; rewrite around `completion` on `[[workflow.states.transitions]]` |
| 55 | `apm spec`: write and check spec sections | Must land before #61 and #62 (instruction files describe using it) |
| 56 | `apm start --next`: delegator primitive | Must land before #46 (apm work is a thin loop around it) |
| 47 | Bug: apm start does not fetch main before merging | Independent bug fix; no deps; low risk; do early |

### Priority 3 — Secondary commands and amendments (depend on #53/#55/#56)

| # | Ticket | Notes |
|---|--------|-------|
| 52 | `apm init`: create `.apm/` folder and migrate config | **Breaking change** — must implement with backward compat (fall back to `apm.toml` at root if `.apm/` absent). Never a flag day. |
| 54 | `apm validate` | Depends on #53 for schema knowledge |
| 58 | `apm new --context` | Depends on #53 for `context_section` on transitions |
| 59 | `apm clean` | Independent; simple addition |
| 38 | Docker sandbox | Needs amendment addressed first; worker credential model aligns with `completion` design (#41) |
| 46 | `apm work` delegator loop | Needs amendment addressed first; thin wrapper around #56 |

### Priority 2 — Agent infrastructure (depend on #55/#56 being live)

| # | Ticket | Notes |
|---|--------|-------|
| 61 | Create `apm.spec-writer.md` | Instructions reference `apm spec` — must exist first |
| 62 | Create `apm.worker.md` | Instructions reference `apm spec` and `apm start --next` |
| 48 | Extend aggressive mode to `apm new`, `review`, `take` | Needs amendment addressed first; `apm review` approach section changes with #57 |

### Priority 1 — Breaking changes (implement last in their phase)

| # | Ticket | Risk notes |
|---|--------|------------|
| 57 | `apm review` redesign | Breaks current `apm review --to <state>` workflow. Implement after all other Phase C tickets are merged and stable. Consider keeping `--to` as a deprecated alias during transition. |
| 60 | `apm sync` interactive prompts | Breaks silent agent use. **Hard requirement in spec**: default non-interactive when stdout is not a tty; `--non-interactive` flag for explicit suppression. |

### Priority 0 — Deferred

| # | Ticket | Why last |
|---|--------|----------|
| 35 | Hex IDs migration | Renames every ticket file and branch. Must go after all other tickets are merged into main — never mid-flight with open `in_progress` tickets on numeric branches. |
| 51 | `apm review` adds checkboxes to amendment requests | Fold into #57 when that spec is written. |

---

## Dependency graph (short form)

```
apm.agents.md update (now)
  └─ unblocks: all workers

#53 (config schema)
  ├─ #41 (completion on transitions)
  ├─ #55 (apm spec)
  │    ├─ #61 (apm.spec-writer.md)
  │    └─ #62 (apm.worker.md)
  ├─ #56 (apm start --next)
  │    └─ #46 (apm work)
  ├─ #52 (apm init / .apm/)
  ├─ #54 (apm validate)
  ├─ #57 (apm review redesign)   ← implement last
  └─ #58 (apm new --context)

#47 (fetch bug) — independent, any time
#59 (apm clean) — independent, any time
#60 (apm sync interactive) — after #53; tty-detection required
#38 (Docker) — after #41 lands
#48 (aggressive mode) — after #57 lands

#35 (hex IDs) — absolute last, all other PRs merged
```

---

## Known risks during implementation

| Risk | Mitigation |
|------|------------|
| `in_progress → ready` removed | Update `apm.agents.md` now; workers get hard error otherwise |
| `.apm/` migration (#52) is a flag day | Backward compat: read `.apm/config.toml` first, fall back to `apm.toml` |
| `apm sync` prompts hang agents (#60) | Require tty detection + `--non-interactive` in spec before implementing |
| `apm review` redesign (#57) breaks current workflow | Keep `--to` as deprecated alias; implement last |
| `apm spec` (#55) referenced in instruction files before it exists | #61/#62 must be written after #55 ships; until then agents use worktree direct editing |
| Hex ID migration (#35) mid-flight chaos | Do not start until every other ticket is `closed` or `accepted` |
