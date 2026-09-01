+++
id = "bc8919b4"
title = "when dealing with epic, apm sync should be more advance"
state = "closed"
priority = 5
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc8919b4-when-dealing-with-epic-apm-sync-should-b"
created_at = "2026-09-01T18:00:23.105706Z"
updated_at = "2026-09-01T20:10:17.107401Z"
+++

## Spec

### Problem

`apm sync` already detects epics whose tickets are all in a terminal state (`Candidates.epic_submit_hints` in `apm-core/src/sync.rs`) and epics whose branch is already merged into the default branch but not yet deleted (`Candidates.epic_close_hints`). Today `apm/src/cmd/sync.rs` only *prints* these as passive text ("Epics ready to submit (apm epic submit <id>):" / "...to close..."). The operator has to remember to run `apm epic submit <id>` and `apm epic close <id>` by hand afterwards, once per epic, choosing the right submit method (merge / PR / auto) themselves.

Two gaps compound this. First, the hints are computed once, before `sync::apply` closes this run's tickets, so an epic that only becomes fully done as a direct result of tickets closing during *this* `apm sync` invocation is not reported until the *next* `apm sync` run. Second, there is no way to act on a hint in the moment — the operator must re-run a separate command per epic, even though `apm epic submit`/`apm epic close` already implement the exact decision needed (see `run_refresh_epic`'s existing `[1] Merge / [2] PR / [3] Auto / [4] Skip` menu, which this ticket reuses in spirit).

This affects operators running `apm sync` interactively at the end of a work session, which is precisely when several tickets in an epic tend to finish in the same pass.

### Acceptance criteria

- [x] Running `apm sync` (not `--quiet`) that closes the last non-terminal ticket of an epic prints a submit prompt for that epic in the same invocation — a second `apm sync` run is not required to see it
- [x] When more than one epic becomes submit-ready in the same run, each is prompted separately with its own `[1] Merge locally / [2] Open or update PR / [3] Auto / [4] Skip` menu, not one combined yes/no for all epics
- [x] Choosing "Merge locally" merges the epic branch into the default branch the same way `apm epic submit --merge <id>` does today
- [x] Choosing "Open or update PR" pushes the epic branch and creates/updates a PR the same way `apm epic submit --pr <id>` does today
- [x] Choosing "Auto" merges when the merge would be clean and falls back to opening a PR on conflict, the same way `apm epic submit --auto <id>` does today
- [x] Choosing "Skip" (including the non-interactive/EOF default) leaves the epic branch untouched — not merged, no PR opened
- [x] After an epic is successfully submitted (its branch becomes content-merged into the default branch), `apm sync` asks in the same run whether to close it, without a second `apm sync` invocation
- [x] An epic that was already merged into the default branch from a previous run, but not yet closed, is still offered a close prompt
- [x] Accepting a close prompt deletes the epic branch locally and on origin the same way `apm epic close <id>` does today, including its live-worker and implemented-state safety checks; declining leaves the branch (and worktree, if any) in place
- [x] `apm sync --quiet` performs ticket closing without printing or prompting for any epic submit/close action, matching today's non-interactive behaviour
- [x] When an epic is merged locally (via Merge locally or Auto-fallback-to-merge) during the epic submit flow, apm sync offers to push the default branch to origin in the same run, even if it was not already ahead before the epic flow ran

### Out of scope

- A non-interactive flag to auto-submit or auto-close ready epics without prompting (e.g. a hypothetical `--auto-epic`); `--quiet` remains the only opt-out and it fully suppresses epic handling, same as today
- Changes to `apm epic submit` / `apm epic close` themselves — `apm sync` only calls into their existing logic
- Changes to how epic membership or "done" state is derived (`derive_epic_state`, the ticket `epic` frontmatter field) — untouched
- Surfacing this flow in `apm-server` (web UI) — CLI only
- Making `apm sync --offline` fully support the PR/auto submit choices — an offline PR attempt fails with the same network error `apm epic submit --pr` already produces today; not addressed here

### Approach

**1. Extract a reusable, freshness-capable epic-hint function (`apm-core/src/sync.rs`).**
Pull the existing "Epic detection pass" block out of `detect()` (the loop over `crate::epic::epic_branches` that builds `epic_submit_hints`/`epic_close_hints`) into a standalone function:

```rust
pub struct EpicHints {
    pub submit: Vec<(String, String)>, // (epic_id, title)
    pub close: Vec<(String, String)>,
}
pub fn epic_hints(root: &Path, config: &Config) -> Result<EpicHints>
```

`detect()` calls `epic_hints(root, config)?` and assigns the two fields into `Candidates` as before, so `Candidates`'s shape and the existing `epic_submit_hints`/`epic_close_hints` integration tests in `apm/tests/integration.rs` (search "epic_submit_hints") keep passing unchanged. The point of the extraction is that the CLI can now call `epic_hints` again, cheaply, *after* tickets have been closed, instead of relying on the stale snapshot taken before `sync::apply`.

**2. Replace the static hint prints in `apm/src/cmd/sync.rs` with an interactive per-epic flow**, inserted where the current `!quiet && !candidates.epic_submit_hints.is_empty()` / `epic_close_hints` print blocks are (lines ~110-121, between Block 2's ticket-close handling and Block 3's push handling), guarded the same way (`!quiet` only — no new flag, no `is_tty` check, matching the existing `prompt_close` convention elsewhere in this file):

- Before the submit loop, capture the default branch's current local commit (`git rev-parse <default_branch>`) as `default_before`.
- Call `apm_core::sync::epic_hints(root, &config)` fresh (not the `candidates` computed before `sync::apply`) to get submit-ready epics that reflect this run's ticket closures.
- For each `(epic_id, title)`, print a short banner ("Epic <id> — <title>: no tickets remain open.") followed by the same menu text `run_refresh_epic` already uses (`[1] Merge locally`, `[2] Open / update PR`, `[3] Auto (merge if clean, fall back to PR)`, `[4] Skip`), read one line via `std::io::stdin().lock().read_line` (mirror the existing pattern in `apm/src/cmd/epic.rs::run_refresh_epic`), and map the choice to `crate::cmd::epic::run_submit(root, &epic_id, merge, pr, auto_mode)` with the corresponding flags (`4`/anything unmatched → skip, do not call). Print the banner/menu *before* reading stdin so it is always visible in captured output even when stdin is closed/EOF.
- Catch `Err` from `run_submit` and print `warning: could not submit epic <id>: {e:#}` without aborting the rest of `apm sync`.
- After the submit loop finishes (all epics processed), re-read the default branch's local commit (`default_after`). If `default_after != default_before`, some choice (an explicit "Merge locally", or an "Auto" that resolved to a clean local merge rather than falling back to PR) moved the default branch tip forward. `run_submit` itself does not report which path it took, so comparing the tip before/after the whole loop is the simplest reliable signal — set `default_is_ahead = true` in that case. This makes the existing "push {default_branch} to origin now?" prompt in Block 3 (sync.rs:127-132) fire in the same run even when the default branch was not already ahead before the epic flow ran (e.g. it was already in sync with origin, or `--offline` skipped Block 1's `sync_default_branch` check).
- After the submit loop, call `apm_core::sync::epic_hints` a second time to get the current close-ready set — this naturally includes epics just merged locally in this run plus any pre-existing merged-but-undeleted epics.
- For each `(epic_id, title)` in that set, prompt `crate::util::prompt_yes_no("Close epic <id> — <title>? [y/N] ")` and, on yes, call `crate::cmd::epic::run_close(root, &epic_id, false)`, again catching and warning on error rather than aborting.
- Delete the old static "Epics ready to submit (apm epic submit <id>):" / "Epics ready to close (apm epic close <id>):" print blocks — this flow supersedes them.

**3. No new CLI flags.** `--quiet` continues to be the single opt-out and, as today, skips epic handling entirely. `--offline` does not gate epic prompting — ticket closing itself already runs offline (see the existing "Block 2 ... unconditional" comment); a PR-based choice while offline surfaces the same `gh`/network error `apm epic submit --pr` already produces. A "Merge locally" or "Auto" choice that merges cleanly works fine offline since it is a local git operation; the resulting `default_is_ahead = true` still causes Block 3's push prompt to fire, but pushing is itself gated by `!offline` (Block 3's outer `if !offline` in sync.rs:125), so an offline run merges locally and simply does not offer to push — consistent with how `default_is_ahead` from Block 1 already behaves under `--offline` today.

**4. Tests.** Add an `apm-core` integration test (alongside the existing epic hint tests in `apm/tests/integration.rs`) asserting `epic_hints` reflects a ticket-close commit made moments earlier, without a fresh process. Add `apm/tests/e2e.rs` coverage asserting the submit banner/menu text appears in `apm sync`'s stdout when an epic becomes ready mid-run, and that `--quiet` suppresses it entirely. The existing e2e helper (`Command::output()`) inherits the cargo-test process's already-closed/EOF stdin, so prompts safely default to "skip"/"no" without needing new stdin-piping test infrastructure; assert on the printed banner and on the epic branch remaining un-merged/un-deleted after the run, not on the acted-upon outcome.

Add one additional regression test for the `default_is_ahead` fix, in `apm/tests/integration.rs` rather than `e2e.rs` — that file already has a bare-origin-plus-local-clone harness for exactly this shape (see `setup_sync_repo` in the sync-push test block), whereas `e2e.rs`'s `Env::setup` has no origin at all. This test cannot assert on the interactive "push {default_branch} to origin now?" prompt text: that prompt is gated on `is_tty && !quiet` (`apm/src/cmd/sync.rs:128`, `is_tty` from `stdin().is_terminal()` at line 17), and stdin piped from a test process is never a terminal, so the prompt would not appear under the test harness even with a fully correct implementation. Instead: build a bare origin + local clone with the default branch already pushed and in sync (`default_is_ahead` false going in), create an epic whose last open ticket closes during this `apm sync` invocation, run `apm sync --push-default --auto-close` with stdin piped `"1\n"` (selecting "Merge locally" at the epic submit menu — the menu print itself is not tty-gated, only the push prompt is). `--auto-close` is required here: without it, the ticket-close confirmation ("Close all? [y/N]", sync.rs:89-90, `crate::util::prompt_yes_no`) is gated on `!quiet` only (not `is_tty`), so it fires before the epic submit menu and would consume the piped `"1\n"` as its own answer (`"1" != "y"` → close declined) — the epic's last ticket would never close, the epic would never become submit-ready, and the menu would never see the `"1"`. `--auto-close` confirms ticket closing without reading stdin, leaving the piped `"1\n"` free for the epic submit menu; the subsequent close-epic prompt then reads EOF and safely declines. Assert that the *origin* bare repo's default-branch tip (read directly, e.g. via the existing `rev_parse` helper) advances to include the epic-merge commit. `--push-default` bypasses the tty-gated confirmation prompt, but the push it triggers still sits inside the outer `if default_is_ahead` block (sync.rs:127) — so the remote tip can only advance if the epic submit loop actually set `default_is_ahead = true`, which is what this test verifies.

### Open questions


### Amendment requests

- [x] Approach: default_is_ahead is computed in Block 1 (apm/src/cmd/sync.rs:30) before the epic flow runs, so after a '[1] Merge locally' (or '[3] Auto' that merged) submit, Block 3 never offers to push main; an immediate 'yes' to the close prompt then runs git push origin --delete on the epic branch while the merged commits exist only on unpushed local main. After the epic submit loop, set default_is_ahead = true whenever any submit merged locally (i.e. main's tip changed), so the existing 'push main to origin now?' prompt at sync.rs:127-132 fires in the same run. Add an AC for this.
- [x] Tests: the default_is_ahead regression e2e cannot assert on the 'push {default_branch} to origin now?' prompt — it is gated on is_tty && !quiet (apm/src/cmd/sync.rs:128, is_tty from stdin is_terminal at sync.rs:17), and piped stdin is not a terminal, so the prompt never appears under the test harness even with a correct implementation. Rework: give the temp repo a file:// origin with the default branch starting in sync, run apm sync --push-default with stdin piped '1\n', and assert origin's default-branch tip advances to include the epic merge — --push-default bypasses the tty-gated prompt but the push still sits inside the outer 'if default_is_ahead', so the advancing remote tip proves the epic flow set it.
- [x] Tests: the default_is_ahead regression test must invoke apm sync --push-default --auto-close. Without --auto-close, the ticket-close prompt ('Close all? [y/N]', apm/src/cmd/sync.rs:89) — which is gated on !quiet only, not on is_tty — fires before the epic submit menu and consumes the piped '1\n' as its answer ('1' != 'y' → close declined), so the epic's last ticket never closes, the epic never becomes submit-ready, and the menu never sees the '1'. --auto-close (apm/src/main.rs:556) confirms ticket closing without reading stdin, leaving the piped '1\n' for the epic submit menu; the subsequent close-epic prompt then reads EOF and safely declines.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-09-01T18:00Z | — | new | philippepascal |
| 2026-09-01T18:03Z | new | groomed | philippepascal |
| 2026-09-01T18:03Z | groomed | in_design | philippepascal |
| 2026-09-01T18:09Z | in_design | specd | claude |
| 2026-09-01T18:13Z | specd | amend | philippepascal |
| 2026-09-01T18:31Z | amend | in_design | philippepascal |
| 2026-09-01T18:32Z | in_design | specd | claude |
| 2026-09-01T18:34Z | specd | amend | philippepascal |
| 2026-09-01T18:35Z | amend | in_design | philippepascal |
| 2026-09-01T18:38Z | in_design | specd | claude |
| 2026-09-01T19:03Z | specd | amend | philippepascal |
| 2026-09-01T19:03Z | amend | in_design | philippepascal |
| 2026-09-01T19:04Z | in_design | specd | claude |
| 2026-09-01T19:17Z | specd | ready | philippepascal |
| 2026-09-01T19:17Z | ready | in_progress | philippepascal |
| 2026-09-01T19:52Z | in_progress | implemented | claude |
| 2026-09-01T20:10Z | implemented | closed | philippepascal(apm-sync) |
