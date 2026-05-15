+++
id = "177b68b3"
title = "UI shows assembled worker prompt for a ticket"
state = "in_design"
priority = 0
effort = 4
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/177b68b3-ui-shows-assembled-worker-prompt-for-a-t"
created_at = "2026-05-14T21:14:45.432859Z"
updated_at = "2026-05-15T01:52:56.615052Z"
depends_on = ["ba121f45", "de2588b4"]
+++

## Spec

### Problem

The apm UI's ticket-detail view has no way to inspect the system prompt a worker would receive before dispatch. The only path is to launch a live worker, which is slow and gives no chance to catch misconfigured agents or instructions before they consume compute. After ba121f45 and de2588b4 land, `build_system_prompt()` is deterministic and accessible via `apm prompt <id>` — but only from the CLI.\n\nThis ticket wires that capability into the UI. The goal is twofold: supervisors can verify "is this really the prompt my worker will see?" before clicking a transition button, and they can experiment with different agent-name overrides without committing to them, which is the primary debugging path for small-model agents (pi, phi4, etc.) that misbehave unexpectedly.

### Acceptance criteria

- [ ] `GET /api/tickets/:id/prompt` returns `{"prompt": "<text>"}` for a ticket in any state.
- [ ] `GET /api/tickets/:id/prompt?agent=<name>` returns the prompt computed as if the ticket's `agent` frontmatter field were set to `<name>`.
- [ ] `GET /api/tickets/:id/prompt` returns 404 when the ticket ID matches no ticket.
- [ ] The ticket-detail header shows a "Prompt" button adjacent to the existing "Review" button; it is only rendered when a ticket is loaded.
- [ ] Clicking "Prompt" opens a modal that fetches and displays the assembled system prompt in a scrollable monospace block.
- [ ] The modal contains an agent-override text input pre-filled with the ticket's current `agent` frontmatter value (empty when the field is absent).
- [ ] Submitting a new agent name in the input (blur or Enter) refetches the prompt and re-renders the content without closing the modal.
- [ ] Closing the modal (× button or Escape key) does not write any change to the ticket.

### Out of scope

- Editing the prompt text from the UI (the modal is read-only)
- Persisting the agent override selected in the modal to the ticket frontmatter (`apm set <id> agent <name>` covers that workflow)
- A role-override input in the modal (role defaults to "worker"; the CLI `apm prompt` handles edge cases)
- A dropdown of known agent names (a text input avoids needing a new `/api/agents/list` endpoint)
- Pre-flight prompt confirmation integrated into the transition buttons (a future ticket could add that)

### Approach

Step 1 — server endpoint (apm-server)\n\napm-server/src/models.rs — add PromptResponse { prompt: String } (Serialize) and PromptQuery { agent: Option<String>, role: Option<String> } (Deserialize).\n\napm-server/src/handlers/tickets.rs — add handler get_ticket_prompt: (1) accept Path(id) and Query(params): Query<PromptQuery>; (2) return 501 if state.git_root() is None; (3) load tickets, resolve full ID with resolve_id_in_slice, return 404 if missing; (4) in spawn_blocking call apm_core::prompt::run(root, &full_id, params.agent.as_deref(), params.role.as_deref()) — the Result<String> function from ba121f45; (5) return Json(PromptResponse { prompt }) on success or a 500 JSON error body on failure.\n\napm-server/src/main.rs — import get_ticket_prompt and add .route("/api/tickets/:id/prompt", get(get_ticket_prompt)) to the protected router after /api/tickets/:id/body.\n\nDependency note: if ba121f45 places the run function in start.rs rather than a new prompt module, update the call site accordingly. Expected interface: fn(root: &Path, id: &str, agent: Option<&str>, role: Option<&str>) -> Result<String>.\n\nStep 2 — UI modal component (apm-ui)\n\napm-ui/src/components/PromptModal.tsx (new file). Props: ticketId string, initialAgent string-or-undefined, onClose callback. State: agentInput (controlled input) and committedAgent (debounce target — updated on blur or Enter), both initialized from initialAgent. useQuery on key ['ticket-prompt', ticketId, committedAgent] fetches GET /api/tickets/{id}/prompt with ?agent=<name> appended when committedAgent is non-empty (refetchOnWindowFocus false, no interval). Renders a fixed full-screen overlay with a centred card: header with title and close button; agent text input with placeholder 'default' that commits on blur/Enter; scrollable pre block for the prompt text (monospace, whitespace-pre-wrap); loading skeleton; error message. useEffect registers Escape to call onClose. Backdrop click calls onClose.\n\napm-ui/src/components/TicketDetail.tsx — three changes: (1) add agent?: string to the local TicketDetail interface; (2) add const [showPrompt, setShowPrompt] = useState(false); (3) add a Prompt button immediately before the Review button in the header (guarded by data), and conditionally mount PromptModal after TransitionButtons when showPrompt && data, passing ticketId, initialAgent={data.agent}, onClose.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:46Z | groomed | in_design | philippe |