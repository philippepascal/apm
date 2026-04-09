+++
id = "7d7d9c35"
title = "UI: clean option window"
state = "in_progress"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7d7d9c35-ui-clean-option-window"
created_at = "2026-04-09T05:15:30.189617Z"
updated_at = "2026-04-09T05:41:50.383427Z"
+++

## Spec

### Problem

The clean command exposes seven flags (--dry-run, --force, --branches, --remote, --older-than, --untracked, --yes) and produces detailed log output (per-worktree, per-branch, per-remote-branch messages plus warnings). The existing Clean button in the SupervisorView header fires a hard-coded POST /api/clean with no options and discards all log output — the server handler ignores every flag and returns only a removed count.

Users who need to preview what clean will do (dry-run), delete local branches alongside worktrees (--branches), clean remote branches (--remote --older-than), force-remove unmerged worktrees (--force), or remove untracked files first (--untracked) must fall back to the CLI. Log output produced during dry-run is completely invisible in the UI.

The fix is to replace the one-click Clean button with a modal window that exposes all options and displays the command log output, making dry-run a first-class UI workflow.

### Acceptance criteria

- [x] Clicking the Clean button in the SupervisorView header opens the Clean modal instead of immediately running clean
- [x] The Clean modal displays a Dry run checkbox
- [x] The Clean modal displays a Branches checkbox (also remove local ticket/* branches)
- [x] The Clean modal displays a Force checkbox (bypass merge checks)
- [x] The Clean modal displays an Untracked checkbox (remove untracked files from worktrees before removal)
- [x] The Clean modal displays a Remote checkbox
- [x] When Remote is checked, an Older than text field appears and is required before Run is enabled
- [x] When Remote is unchecked, the Older than field is hidden
- [x] The Run button is disabled when Remote is checked and Older than is empty
- [x] The modal has a scrollable log output area that is empty on open
- [x] Clicking Run calls POST /api/clean with the selected options and displays the returned log lines in the output area
- [x] While the request is in-flight the Run button shows a spinner and is disabled
- [x] When Dry run is checked the Run button label reads "Dry run"; otherwise it reads "Run"
- [x] After a successful non-dry-run execution the tickets query is invalidated (board refreshes)
- [x] After a successful dry-run execution the tickets query is NOT invalidated
- [x] Pressing Escape closes the modal
- [x] Clicking outside the modal (backdrop) closes the modal
- [x] The modal can be reopened cleanly after being closed (state resets: checkboxes unchecked, older-than cleared, log output cleared)
- [x] POST /api/clean accepts an optional JSON body with fields: dry_run, force, branches, remote, older_than, untracked
- [x] POST /api/clean returns a JSON object with a log field (string) containing all output lines and a removed field (number)
- [x] When --remote is used with --older-than, the server passes both to the clean logic and remote candidates appear in the log

### Out of scope

- Per-branch confirmation prompts for --force (the modal acts as the single confirmation; --yes is implicitly always true when called from the UI)
- Streaming log output line-by-line as the command runs (full response returned when complete is sufficient)
- Keyboard shortcut to open the Clean modal (the button is enough for now)
- Persisting option selections between modal opens
- Any changes to apm-core clean logic or apm CLI clean command

### Approach

Five files change: the server handler, the Zustand store, a new CleanModal component, WorkScreen, and SupervisorView.

#### Server — apm-server/src/main.rs

1. Add a `CleanRequest` deserializable struct above `clean_handler`:
   ```rust
   #[derive(serde::Deserialize, Default)]
   struct CleanRequest {
       dry_run:    Option<bool>,
       force:      Option<bool>,
       branches:   Option<bool>,
       remote:     Option<bool>,
       older_than: Option<String>,
       untracked:  Option<bool>,
   }
   ```

2. Change the `clean_handler` signature to accept an optional JSON body:
   ```rust
   async fn clean_handler(
       State(state): State<Arc<AppState>>,
       body: Option<Json<CleanRequest>>,
   ) -> Result<Response, AppError>
   ```

3. Replace the current handler body with a port of the full `apm/src/cmd/clean.rs` logic, collecting all output into a `Vec<String>` log buffer instead of calling `println!`/`eprintln!`. Steps:
   - Unpack `body` with defaults (all false / None)
   - Validate: if `remote && older_than.is_none()` return 400 with error message
   - Call `clean::candidates(root, config, force, untracked, dry_run)` — push `candidate_warnings` into log
   - For each dirty worktree push the same warning strings the CLI prints
   - For each candidate in dry-run mode push "would remove worktree ..." / "would remove branch ..." / "would keep branch ..." lines
   - For each candidate in live mode call `clean::remove(root, candidate, force, branches)` and push removal lines plus `remove_out.warnings`
   - For remote path call `clean::remote_candidates(root, config, threshold)`: in dry_run push "would delete remote branch ..." lines; in live mode (always `yes=true` for UI) call `git::delete_remote_branch` and push "deleted remote branch ..." lines
   - Return `Json({ "log": log.join("\n"), "removed": count })`
   - The existing `/api/clean` route registration does not change — only the handler body.

#### UI — Zustand store (apm-ui/src/store/useLayoutStore.ts)

Add two fields alongside the existing `newTicketOpen`/`newEpicOpen` pattern:
- `cleanOpen: boolean` (default `false`)
- `setCleanOpen: (v: boolean) => void`

#### UI — CleanModal component (apm-ui/src/components/CleanModal.tsx)

New file, modelled on `NewTicketModal.tsx`. Local state:
- `dryRun`, `force`, `branches`, `remote`, `untracked` — boolean (all default `false`)
- `olderThan` — string (default `""`)
- `log` — string (output area, default `""`)

Behaviour:
- `useEffect` on open: reset all state when `open` goes false (same pattern as `NewTicketModal`)
- Escape key handler (same pattern); backdrop click closes
- `useMutation` calling `POST /api/clean` with JSON body `{ dry_run, force, branches, remote, older_than: remote ? olderThan : undefined, untracked }`
- `onSuccess`: set `log` to `data.log`; if `!dryRun` invalidate `["tickets"]` query
- `onError`: set `log` to error message string
- Run button disabled when `isPending` or `(remote && !olderThan.trim())`
- Run button label: `dryRun ? "Dry run" : "Run"`

Layout (same backdrop/card pattern as `NewTicketModal`):
- Header: "Clean worktrees"
- Body: checkbox rows for Dry run, Branches, Force, Untracked, Remote; when Remote is checked show an "Older than" text input (placeholder "30d or YYYY-MM-DD") — hidden when Remote is unchecked; a `<pre>` (or `<textarea readOnly>`) for log output with min-height ~120 px, `overflow-y-auto`, monospace `text-xs`, dark/muted background
- Footer: Cancel + Run buttons (Run shows `<Loader2>` spinner while pending)

#### UI — WorkScreen.tsx

1. Import `CleanModal`; destructure `cleanOpen` / `setCleanOpen` from `useLayoutStore`
2. Render `<CleanModal open={cleanOpen} onOpenChange={setCleanOpen} />` alongside `NewTicketModal` and `NewEpicModal` — in both the `reviewMode` branch and the normal branch

#### UI — SupervisorView (apm-ui/src/components/supervisor/SupervisorView.tsx)

1. Destructure `setCleanOpen` from `useLayoutStore`
2. Remove the local `postClean` function, `cleanMutation`, and `cleanError` state
3. Change the Clean button `onClick` to `() => setCleanOpen(true)`; remove `disabled={cleanMutation.isPending}` and the spinner — the button simply opens the modal

### Server — apm-server/src/main.rs

1. Add a `CleanRequest` deserializable struct above `clean_handler`:
   ```rust
   #[derive(serde::Deserialize, Default)]
   struct CleanRequest {
       dry_run:    Option<bool>,
       force:      Option<bool>,
       branches:   Option<bool>,
       remote:     Option<bool>,
       older_than: Option<String>,
       untracked:  Option<bool>,
   }
   ```

2. Change the `clean_handler` signature to accept an optional JSON body:
   ```rust
   async fn clean_handler(
       State(state): State<Arc<AppState>>,
       body: Option<Json<CleanRequest>>,
   ) -> Result<Response, AppError>
   ```

3. Replace the current inline logic with a port of the full `apm/src/cmd/clean.rs` logic, collecting output into a `Vec<String>` log buffer instead of calling `println!`/`eprintln!`. Key points:
   - Validate: if `remote && older_than.is_none()` then return 400 with error message
   - Call `clean::candidates(root, config, force, untracked, dry_run)` and push `candidate_warnings` into log
   - For dirty worktrees push the same warning strings as the CLI
   - For each candidate in dry-run mode push "would remove worktree ..." / "would remove branch ..." / "would keep branch ..." lines
   - For each candidate in normal mode call `clean::remove` and push removal lines plus `remove_out.warnings`
   - For remote path call `clean::remote_candidates`; in dry_run push "would delete remote branch ..." lines; in live mode (always yes=true for UI calls) call `git::delete_remote_branch` and push "deleted remote branch ..." lines
   - Return `Json({ "log": log.join("\n"), "removed": count })`

   The existing `/api/clean` route registration does not change — only the handler body.

---

### UI — Zustand store (apm-ui/src/store/useLayoutStore.ts)

Add two fields alongside the existing `newTicketOpen`/`newEpicOpen` pattern:
- `cleanOpen: boolean` (default `false`)
- `setCleanOpen: (v: boolean) => void`

---

### UI — CleanModal component (apm-ui/src/components/CleanModal.tsx)

New file, modelled on `NewTicketModal.tsx`. Local state:
- `dryRun`, `force`, `branches`, `remote`, `untracked` — boolean checkboxes (all default `false`)
- `olderThan` — string (default `""`)
- `log` — string (output area content, default `""`)

Behaviour:
- `useEffect` on open: reset all state when modal closes (same pattern as `NewTicketModal`)
- Escape key handler (same pattern)
- Backdrop click closes modal
- `useMutation` calling `POST /api/clean` with JSON body `{ dry_run, force, branches, remote, older_than: remote ? olderThan : undefined, untracked }`
- `onSuccess`: set `log` to `data.log`; if `!dryRun` invalidate `["tickets"]` query
- `onError`: set `log` to error message string
- Run button disabled when `isPending` or `(remote && !olderThan.trim())`
- Run button label: `dryRun ? "Dry run" : "Run"`

Layout (same backdrop/card pattern as `NewTicketModal`):
- Header: "Clean worktrees"
- Body:
  - Checkbox rows: Dry run, Branches, Force, Untracked, Remote
  - When Remote is checked, show an "Older than" text input inline (e.g. "30d" or "2026-01-01") with a small hint label; hide it when Remote is unchecked
  - A `<pre>` (or `<textarea readOnly>`) for log output — min-height ~120 px, `overflow-y-auto`, monospace `text-xs`, dark/muted background — empty until Run is clicked
- Footer: Cancel + Run buttons (Run shows `<Loader2>` spinner while pending)

---

### UI — WorkScreen.tsx

1. Import `CleanModal` and destructure `cleanOpen` / `setCleanOpen` from `useLayoutStore`
2. Render `<CleanModal open={cleanOpen} onOpenChange={setCleanOpen} />` alongside the existing `NewTicketModal` and `NewEpicModal` (add it in both the `reviewMode` branch and the normal render branch)

---

### UI — SupervisorView (apm-ui/src/components/supervisor/SupervisorView.tsx)

1. Import `setCleanOpen` from `useLayoutStore`
2. Remove the local `postClean` function, `cleanMutation`, and `cleanError` state
3. Change the Clean button `onClick` to `() => setCleanOpen(true)`; remove `disabled={cleanMutation.isPending}` and spinner — the button now simply opens the modal

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T05:15Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:23Z | groomed | in_design | philippepascal |
| 2026-04-09T05:29Z | in_design | specd | claude-0409-0523-7528 |
| 2026-04-09T05:30Z | specd | ready | apm |
| 2026-04-09T05:41Z | ready | in_progress | philippepascal |
