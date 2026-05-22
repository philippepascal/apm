+++
id = "177b68b3"
title = "UI shows assembled worker prompt for a ticket"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/177b68b3-ui-shows-assembled-worker-prompt-for-a-t"
created_at = "2026-05-14T21:14:45.432859Z"
updated_at = "2026-05-22T02:23:57.769513Z"
depends_on = ["ba121f45", "de2588b4"]
+++

## Spec

### Problem

The apm UI's ticket-detail view has no way to inspect the system prompt a worker would receive before dispatch. The only path is to launch a live worker, which is slow and gives no chance to catch misconfigured agents or instructions before they consume compute. After ba121f45 and de2588b4 land, `build_system_prompt()` is deterministic and accessible via `apm prompt <id>` — but only from the CLI.\n\nThis ticket wires that capability into the UI. The goal is twofold: supervisors can verify "is this really the prompt my worker will see?" before clicking a transition button, and they can experiment with different agent-name overrides without committing to them, which is the primary debugging path for small-model agents (pi, phi4, etc.) that misbehave unexpectedly.

### Acceptance criteria

- [x] `GET /api/tickets/:id/prompt` returns `{"prompt": "<text>"}` for a ticket in any state.
- [x] `GET /api/tickets/:id/prompt?agent=<name>` returns the prompt computed as if the ticket's `agent` frontmatter field were set to `<name>`.
- [x] `GET /api/tickets/:id/prompt` returns 404 when the ticket ID matches no ticket.
- [x] The ticket-detail header shows a "Prompt" button adjacent to the existing "Review" button; it is only rendered when a ticket is loaded.
- [x] Clicking "Prompt" opens a modal that fetches and displays the assembled system prompt in a scrollable monospace block.
- [x] The modal contains an agent-override text input pre-filled with the ticket's current `agent` frontmatter value (empty when the field is absent).
- [x] Submitting a new agent name in the input (blur or Enter) refetches the prompt and re-renders the content without closing the modal.
- [x] Closing the modal (× button or Escape key) does not write any change to the ticket.

### Out of scope

- Editing the prompt text from the UI (the modal is read-only)
- Persisting the agent override selected in the modal to the ticket frontmatter (`apm set <id> agent <name>` covers that workflow)
- A role-override input in the modal (role defaults to "worker"; the CLI `apm prompt` handles edge cases)
- A dropdown of known agent names (a text input avoids needing a new `/api/agents/list` endpoint)
- Pre-flight prompt confirmation integrated into the transition buttons (a future ticket could add that)

### Approach

#### Step 1 — Server endpoint (apm-server)

`apm-server/src/models.rs` — add `PromptResponse { prompt: String }` (Serialize) and `PromptQuery { agent: Option<String>, role: Option<String> }` (Deserialize).

`apm-server/src/handlers/tickets.rs` — add handler `get_ticket_prompt`:
- Accept `Path(id)` and `Query(params): Query<PromptQuery>`.
- Return 501 if `state.git_root()` is None.
- Load tickets, resolve full ID with `resolve_id_in_slice`; return 404 if missing.
- In `spawn_blocking`, call `apm_core::prompt::run(root, &full_id, params.agent.as_deref(), params.role.as_deref())` — the `Result<String>` function from ba121f45.
- Return `Json(PromptResponse { prompt })` on success, or a 500 JSON error body on failure.

`apm-server/src/main.rs` — import `get_ticket_prompt` and add `.route("/api/tickets/:id/prompt", get(get_ticket_prompt))` to the protected router after `/api/tickets/:id/body`.

Dependency note: if ba121f45 places `run` in `start.rs` rather than `prompt.rs`, update the call site accordingly. Expected interface: `fn(root: &Path, id: &str, agent: Option<&str>, role: Option<&str>) -> Result<String>`.

#### Step 1 integration tests

Follow the `build_app_with_tickets()` + `.oneshot()` pattern used by the `put_body` tests in `apm-server/src/main.rs` (lines 862–911). The test app is constructed without the `require_auth` middleware, so no session token or cookie is needed. Add three tests in the same file:

- **In-memory returns 501**: `build_app_with_tickets(test_tickets())`, `GET /api/tickets/aaaabbbb/prompt`, assert `501 NOT_IMPLEMENTED`. (In-memory sources have no git root, the same early-return path as `put_body_in_memory_returns_501`.)
- **Unknown ID returns 404**: same test app, `GET /api/tickets/zzzzzzzz/prompt`, assert `404`.
- **Known ID happy path** (git-backed): construct a temp dir with a minimal ticket file, build a git-backed app state, call the endpoint, assert `200` and a non-empty `prompt` field in the JSON body. Follow whatever git-backed test helper exists alongside `build_app_with_tickets`; if none exists, defer this test to a follow-up.

#### Step 2 — UI modal component (apm-ui)

`apm-ui/src/components/PromptModal.tsx` (new file). Props: `ticketId: string`, `initialAgent: string | undefined`, `onClose: () => void`. State: `agentInput` (controlled input) and `committedAgent` (updated on blur or Enter), both initialized from `initialAgent`. `useQuery` on key `['ticket-prompt', ticketId, committedAgent]` fetches `GET /api/tickets/{id}/prompt` with `?agent=<name>` appended when `committedAgent` is non-empty (`refetchOnWindowFocus: false`, no polling interval). Renders a fixed full-screen overlay with a centred card:
- Header: title ("Worker Prompt") and x close button.
- Agent text input with placeholder `'default'` that commits on blur or Enter key.
- Scrollable `<pre>` block for the prompt text (monospace, `whitespace-pre-wrap`).
- Loading skeleton while the query is in-flight.
- Inline error message on failure.

`useEffect` registers Escape to call `onClose`. Backdrop click calls `onClose`.

`apm-ui/src/components/TicketDetail.tsx` — three changes:
1. Add `agent?: string` to the local `TicketDetail` interface.
2. Add `const [showPrompt, setShowPrompt] = useState(false)`.
3. Add a "Prompt" button immediately before the existing "Review" button in the header (guarded by `data`); conditionally mount `<PromptModal>` after `<TransitionButtons>` when `showPrompt && data`, passing `ticketId`, `initialAgent={data.agent}`, and `onClose={() => setShowPrompt(false)}`.

### Open questions


### Amendment requests

- [x] The Approach has literal \\n\\n strings instead of real newlines — visible as one wall of text in apm show. This was a quoting accident when the ticket was filed via --context. Rewrite the Approach with proper section breaks so an implementer can scan it. Section structure to preserve: Step 1 (server endpoint) with the three file/handler/route bullets, then Step 2 (UI modal component) with the PromptModal.tsx and TicketDetail.tsx bullets.
- [x] Server-side endpoint in the protected router needs an integration-test note in the Approach: what auth fixture or token does the test use? The existing /api/tickets/:id/body endpoint pattern is the right reference; just point to it so the implementer doesn't have to rediscover it.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:46Z | groomed | in_design | philippe |
| 2026-05-15T01:53Z | in_design | specd | default-0515-0146-1530 |
| 2026-05-15T19:56Z | specd | ammend | philippe |
| 2026-05-15T21:54Z | ammend | in_design | philippe |
| 2026-05-15T21:58Z | in_design | specd | default-0515-2154-9ed8 |
| 2026-05-21T22:52Z | specd | ready | philippepascal |
| 2026-05-22T00:11Z | ready | in_progress | philippepascal |
| 2026-05-22T00:27Z | in_progress | implemented | claude-0522-0011-fbd8 |
| 2026-05-22T02:23Z | implemented | closed | philippepascal(apm-sync) |
