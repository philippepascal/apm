# APM Spec vs. Linear: Missing Features Analysis

> What Linear does that the current APM spec does not cover.

---

## 1. Multiple PRs per ticket (and multiple tickets per PR)

**What Linear does**

Linear treats PR-to-issue as a many-to-many relationship. You can link multiple PRs to one issue, and multiple issues to one PR.

The behavior when multiple PRs close one issue: Linear watches all of them. The issue does not move to "Done" until the *last* closing PR is merged. If you have PR #88 and PR #89 both tagged `Fixes ENG-42`, the issue stays "In Progress" when #88 merges. It only moves to Done when #89 merges too.

The reverse is also true: one PR can close multiple issues at once. `Fixes ENG-42, ENG-43, DES-5` in a PR description links three issues to that PR. Each one moves independently based on its own set of PRs.

**What the APM spec does**

The `tickets` table has:
```sql
pr_number  INTEGER,
pr_url     TEXT,
```

One slot. This means:
- If Claude opens a frontend PR and a backend PR for the same ticket, APM only tracks whichever one it saw last (or first, depending on how the webhook handler is written)
- The second PR silently overwrites the first
- The `implemented → accepted` transition fires when *any* PR closes, even if it's just one of two required PRs
- No way to link a PR to two tickets simultaneously

**Why it matters in practice**

The ticker workflow generates this exact case. A ticket for "Add auth" might have:
- A backend PR (`feature/42-auth-backend`)
- A follow-up PR (`feature/42-auth-frontend`)

Both reference `Closes #42`. With the current schema, APM moves the ticket to `accepted` when the first one merges, losing track of the second entirely. Or if the webhook fires for the second one after the first, it overwrites `pr_number` with the second PR number and the first is gone.

**The fix**

Replace `pr_number` and `pr_url` columns on `tickets` with a join table:

```sql
CREATE TABLE ticket_prs (
  id           INTEGER PRIMARY KEY,
  ticket_id    INTEGER REFERENCES tickets(id),
  pr_number    INTEGER NOT NULL,
  pr_url       TEXT NOT NULL,
  link_type    TEXT NOT NULL DEFAULT 'closes',  -- 'closes' | 'refs'
  state        TEXT NOT NULL DEFAULT 'open',    -- 'open' | 'merged' | 'closed'
  review_state TEXT,   -- null | 'approved' | 'changes_requested' | 'review_requested'
  opened_at    TIMESTAMP,
  merged_at    TIMESTAMP,
  closed_at    TIMESTAMP
);
```

And the `implemented → accepted` transition logic becomes:

```
all closing PRs for this ticket are merged
AND at least one closing PR exists
```

Not just "a PR event fired."

---

## 2. Commit-level linking (magic words in commit messages)

**What Linear does**

Linear listens to GitHub push events and parses individual commit messages for magic words. A commit message like:

```
Fixes ENG-42: implement CSV export
```

Does two things:
- When the commit is pushed to any branch: moves the issue to "In Progress"
- When the commit reaches the default branch (merged): moves the issue to "Done"

This is separate from PR linking. You don't need a PR at all. A commit alone can drive the full state transition. It's set up via a GitHub push-events webhook pointing at Linear.

**What the APM spec does**

The spec handles the `push` webhook event, but only for the activity indicator:

```
push (to feature branch) → Mark ticket as "active"; record activity
```

It extracts the branch name (`feature/42-dark-mode`) to identify the ticket and update `last_active_at`. It does not parse commit messages. It does not trigger state transitions.

**Why it matters in practice**

In the ticker workflow, Claude creates a branch and immediately starts committing. The sequence is:

```
1. git checkout -b feature/42-export-csv   (local only, no webhook)
2. git push -u origin feature/42-export-csv  (push webhook fires)
3. Claude commits work over hours/days      (push webhooks fire)
4. Claude opens PR                          (pull_request.opened fires)
```

Between steps 2 and 4, the ticket sits at `ready` in APM. There's no signal that work has started. The swimlane shows it in the READY column even though Claude is actively implementing it. Linear would move it to "started" at step 2 because the branch push itself is the signal.

**The fix**

Two parts:

First, on any push to a branch matching `feature/<n>-*`, immediately set `tickets.branch` and show the ticket as "active" in the READY column. This is almost already specced — it just needs to be explicit that it happens on first push.

Second, optionally parse commit messages for magic words. For V1 it's reasonable to rely on PR events. But the spec should note this as a deliberate omission, not an oversight.

---

## 3. Non-closing link type (ref / related-to)

**What Linear does**

Linear has two categories of magic words:

**Closing words** (`fix`, `fixes`, `fixed`, `close`, `closes`, `closed`, `resolve`, `resolves`, `resolved`, `complete`, `completes`, `completing`):
- Attaches the PR to the issue
- Moves issue to "In Progress" when branch is pushed
- Moves issue to "Done" when PR merges to default branch

**Non-closing words** (`ref`, `refs`, `references`, `part of`, `related to`, `contributes to`, `toward`, `towards`):
- Attaches the PR to the issue — visible in the issue's attachment list
- May move issue through intermediate states per workflow config
- Does **not** move issue to "Done" when the PR merges
- The issue continues to whatever state is configured prior to the "on merge" event, but stops there

**What the APM spec does**

The spec has no concept of link type. Every PR linked to a ticket is treated as closing. The `pull_request.merged` webhook fires and APM moves the ticket to `accepted` unconditionally.

The GitHub API reference in `ddd/github-api-reference.md` includes this regex:
```go
re := regexp.MustCompile(`(?i)(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#(\d+)`)
```
This only matches closing words. A PR saying `Part of #42` isn't linked at all — not even for display.

**Why it matters in practice**

Several real scenarios in the ticker workflow:

- **Prerequisite PR**: Claude opens a refactoring PR (`refs #42`) to clean up the code before implementing the feature. With the current spec this PR is invisible to APM.
- **Partial implementation**: A large ticket gets implemented in two PRs. The first (`Part of #42`) does the backend; the second (`Closes #42`) does the frontend. Without link types, APM either marks the ticket done too early or doesn't track the first PR at all.
- **Debugging PR**: A hotfix (`related to #42`) addresses a bug discovered while implementing #42. Should appear as related, not as the implementing PR.

**The fix**

Expand the webhook parser to capture non-closing words and store the type:

```go
closingRe := regexp.MustCompile(`(?i)(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?|complet\w*)\s+#(\d+)`)
refsRe    := regexp.MustCompile(`(?i)(?:refs?|references?|part of|related to|contributes? to|toward[s]?)\s+#(\d+)`)
```

Store in `ticket_prs.link_type = 'closes' | 'refs'`. Only `closes` rows participate in the `implemented → accepted` state transition. Both types show up in the PR list in the ticket detail panel.

---

## 4. PR review state on the ticket

**What Linear does**

When a PR is linked to a Linear issue, Linear tracks the review state and displays it directly on the issue — both on the kanban card and in the issue detail:

- **No review yet**: shows PR title + open state
- **Review requested**: shows "review requested" or specific reviewer avatars with a pending indicator
- **Changes requested**: shows reviewer avatar with a red/warning indicator
- **Approved**: shows reviewer avatar with a green checkmark
- **Team review**: shows "in review" if a team was requested instead of an individual

This means someone can look at the `implemented` swimlane column and see at a glance: two tickets have approved PRs (ready to merge), one has changes requested (needs rework), one is waiting for review — without opening GitHub.

**What the APM spec does**

The spec shows on the implemented card:
- PR link (#88)
- Time in state
- Active indicator

No review state. To find out if a PR is approved or has changes requested, you have to click through to GitHub.

**Why it matters in practice**

The `implemented` column is the most action-sensitive column in the swimlane. Everything in it is waiting for something:
- Tickets waiting for review → you need to ping the reviewer
- Tickets with changes requested → Claude needs to fix things
- Tickets with approved PRs → just needs a merge (your action)

Without review state, all three look identical in the swimlane. You can't prioritize at a glance. The whole point of the single-page view is to eliminate this context-switching — and this is exactly the signal that makes it work.

**The fix**

Listen to the `pull_request_review` webhook event (not currently in the spec's webhook table):

```json
{
  "action": "submitted",
  "review": {
    "state": "approved",
    "user": { "login": "reviewer-name" },
    "submitted_at": "..."
  },
  "pull_request": { "number": 88 }
}
```

Store on `ticket_prs.review_state`. Display on the card:
- Grey: no review yet
- Yellow: review requested
- Red: changes requested
- Green: approved

Add `pull_request_review` to the webhook subscription event list alongside `issues`, `pull_request`, `push`.

---

## 5. Skip/ignore unlinking

**What Linear does**

If a branch is named `feature/eng-42-something`, Linear automatically links every PR from that branch to issue ENG-42 because the issue identifier is in the branch name. But sometimes you don't want that link. Linear supports escaping it with magic words in the PR description:

```
skip ENG-42
ignore ENG-42
```

These words in the PR body explicitly break the auto-link. The PR is detached from the issue even though the branch name would otherwise create the connection. State automation is also suppressed.

**What the APM spec does**

The spec's webhook handler parses the branch name to find the issue number:
```
feature/42-dark-mode → issue #42
```
And creates the link unconditionally. There's no escape hatch.

**Why it matters in practice**

- Claude opens a draft WIP PR on `feature/42-*` to get CI feedback, intends to close it and open a proper one later — both get linked as closing PRs
- A branch is cut from the feature branch for a related cleanup, also named `feature/42-*` — the cleanup PR gets linked and triggers `accepted` when merged
- A cross-cutting PR touches code that happens to live on the feature branch — gets linked to issue 42 even though it has nothing to do with it

Without `skip #42` support, spurious links cause incorrect state transitions.

**The fix**

Before creating a `ticket_prs` row from a branch-name match, check the PR body for skip/ignore:

```go
skipRe := regexp.MustCompile(`(?i)(?:skip|ignore)\s+#(\d+)`)
skippedIssues := extractIssueNumbers(prBody, skipRe)

if contains(skippedIssues, issueNumber) {
    return  // do not create row, do not trigger state change
}
```

Also apply this check on PR body edits (`pull_request.edited` event), not just on open.

---

## 6. Branch push → ticket auto-start

**What Linear does**

Linear has a setting "Auto-assign and move issue to start when copying branch name." When you copy the branch name from the Linear UI, it immediately moves the issue to "In Progress" and assigns it to you — before you even type `git checkout -b`.

Beyond that preference, even without it: the moment Linear receives a push event to a branch containing the issue identifier, it moves the issue to "In Progress." The first commit push is the guaranteed start signal.

**What the APM spec does**

Push events are used only for activity indicators — updating `last_active_at` and showing the active dot (●). The ticket stays in `ready` until `pull_request.opened` fires, which moves it to `implemented`. There is no intermediate signal.

In the ticker workflow, the gap between branch push and PR open can be hours or days. During all that time the `ready` column shows the ticket as untouched (just with a dot). The swimlane can't answer "what's being actively worked on right now" — only "what has an open PR."

**The options**

Three approaches, in order of invasiveness:

**A — Visual only (least invasive):** Keep the state machine as-is. On first push to `feature/<n>-*`, set `tickets.branch` and `last_active_at`. Render cards in `ready` with a branch set in a visually distinct subsection ("in flight") vs cards without a branch ("waiting"). No new state, no workflow change.

**B — Add `in_progress` state:** Add a state between `ready` and `implemented`. First push → `ready → in_progress`. PR open → `in_progress → implemented`. This gives the swimlane a 7th column and makes the signal explicit, but changes the state machine and any tooling that drives transitions.

**C — Implicit subgrouping in READY column:** Don't add a new state. In the swimlane, split the READY column into two subsections: "In flight" (branch set, recent commits) and "Queued" (no branch). Same state machine, no new transitions, better visual signal than just a dot.

Option A or C is right for V1. Option B is more accurate but adds friction to the workflow model.

---

## Summary of Required Schema Changes

| Change | Priority | Effort |
|--------|----------|--------|
| Replace `pr_number/pr_url` with `ticket_prs` table | High — correctness | Low |
| Add `link_type` to `ticket_prs` | High — correctness | Low |
| Add `review_state` to `ticket_prs` | Medium — UX | Low |
| Add `pull_request_review` to webhook subscriptions | Medium — UX | Low |
| Parse commit messages for magic words | Low — V2 | Medium |
| Add skip/ignore detection in PR body parser | Medium — correctness | Low |
| On first branch push: set `tickets.branch` immediately | High — UX | Low |
| Clarify `implemented → accepted` requires all closing PRs merged | High — correctness | Low |
