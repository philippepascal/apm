# Merge strategies, dependencies, and epic concurrency

This spec defines how APM composes work across the default branch, epic
branches, and ticket branches — and the rules that make autonomous workers
safe under that composition.

## Two-tier model

Work runs in one of two modes, distinguished by whether a ticket has an epic.

**Main tier — supervised parallel.** A ticket without an epic targets the
default branch. On `implemented`, APM opens a PR against the default branch.
Multiple main-tier tickets run in parallel; conflicts surface to the
supervisor at PR review time, never to a worker.

**Epic tier — autonomous serial.** A ticket with an epic targets the epic
branch. On `implemented`, APM merges the ticket directly into the epic
branch. Within an epic, at most one worker is active at a time; tickets
pick up each other's work through the epic branch. The epic eventually
merges to default via `apm epic close`, which opens a PR for supervisor
review.

The two tiers are implemented by a single completion strategy
(`pr_or_epic_merge`): merge to `target_branch` when set, otherwise open a PR
to the default branch.

## Recommended default: `pr_or_epic_merge`

`pr_or_epic_merge` is the recommended and default completion strategy for
the `in_progress → implemented` transition. Other strategies remain
available but have caveats that make them unsafe for autonomous workers:

| Strategy | Composes dependencies? | Notes |
|---|---|---|
| `pr_or_epic_merge` | Yes, within an epic | Default. Same strategy yields PR-on-main and merge-to-epic depending on `target_branch`. |
| `merge` | Yes, when ticket and deps share `target_branch` | Lands directly on the target. Skips supervisor review on main. |
| `pr` | No | State→`implemented` fires when the PR is *opened*, not when it merges. Downstream tickets can start before upstream code lands. |
| `none` | No | Nothing lands automatically; downstream tickets cannot rely on upstream code being present. |

## Dependency rules per strategy

A ticket's `depends_on` is enforced wherever it can be written:
`apm new --depends-on` and `apm set <id> depends_on …` (and any future
write site). The rule is derived from the configured completion strategy.
Re-validating at `apm start` is unnecessary because the hash-trip /
`apm validate` mechanism (below) already catches the case where a
previously-valid setup becomes invalid after a config change:

| Strategy | `--depends-on` allowed when … |
|---|---|
| `pr_or_epic_merge` | The ticket and **all** its dependencies belong to the same epic. |
| `merge` | The ticket and **all** its dependencies share the same `target_branch` — same epic, or all on the default branch. |
| `pr` | Never. |
| `none` | Never. |

The reason: dependencies compose only when there is a shared integration
branch onto which upstream code lands before downstream work begins. Each
rule above describes a configuration where that invariant holds.

## Epic concurrency

`max_workers_per_epic` is a global setting in `apm.toml`, default `1`.
Within an epic, the dispatcher will not spawn a second worker while another
is active in the same epic.

The previous per-epic override (`apm epic set <id> max_workers …`) is
removed. Epics are the parallelism unit; if more parallelism is needed,
create another epic. Per-epic concurrency tweaks invite within-epic merge
races and undermine the autonomous-serial guarantee.

Main-tier tickets (no epic) run in parallel, gated by PR review. Concurrent
PR merges to the default branch are a supervisor concern, not a worker
concern.

## Hash-trip on config change

Configuration drift is the highest-impact source of silent breakage: a
strategy change or a workflow edit can invalidate dependencies that were
valid yesterday. APM stores a hash of `apm.toml` and `workflow.toml`
alongside its state; on every command, it compares the live hash to the
stored stamp.

When the hash changes, APM runs `apm validate` automatically. If validation
fails, the offending command is refused (mutating commands) or warned
loudly (read-only commands), with a clear pointer to what's wrong. The
stamp is refreshed only after validation passes.

`apm validate` itself is extended to enforce the dependency rules above:
every ticket's `depends_on` is checked against the current strategy and
target-branch topology, and violations are reported.

## Refresh and close: epic must be quiescent

Long-running epics drift from the default branch. To pull updates in,
`apm refresh-epic <id>` opens a PR from the default branch into the epic
branch. The supervisor reviews and merges; subsequent worker starts in the
epic see the updated tip.

Both `apm refresh-epic` and `apm epic close` require the epic to be
**quiescent**: no ticket in the epic is in an actively-worked state
(`in_design`, `in_progress`, or otherwise has a live worker). The check is
shared between the two commands.

The supervisor is responsible for pausing the dispatcher and waiting for
the active worker (if any) to complete before invoking these commands. APM
enforces the precondition; it does not stop running workers.

## Implementation rules

1. `pr_or_epic_merge` is the documented default for `in_progress →
   implemented`. Other strategies remain available with documented
   tradeoffs.
2. `max_workers_per_epic` is a global config option in `apm.toml`, default
   `1`. The per-epic override is removed.
3. `--depends-on` is gated by the strategy/target rules at every write
   site (`apm new --depends-on`, `apm set <id> depends_on …`, and any
   future write path).
4. `apm validate` enforces the dependency rules across all tickets.
5. A hash-trip on `apm.toml` / `workflow.toml` triggers automatic
   re-validation; failures block mutating commands.
6. `apm refresh-epic` is a new command that opens a PR from the default
   branch into the epic. Both `refresh-epic` and `epic close` require epic
   quiescence.
