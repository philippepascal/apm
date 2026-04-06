+++
id = "720d2e3b"
title = "Interactive agent chat panel in supervisor UI"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/720d2e3b-interactive-agent-chat-panel-in-supervis"
created_at = "2026-04-06T01:41:26.571412Z"
updated_at = "2026-04-06T01:41:26.571412Z"
+++

## Spec

### Problem

The supervisor currently interacts with the delegator agent through a terminal running Claude Code. This means switching between the browser (supervisor UI) and a terminal window to dispatch work, review agent output, and approve actions. The UI has buttons for sync, clean, and ticket management, but the core agent interaction loop — giving instructions, reading responses, approving tool calls — lives entirely outside the UI.

The goal is a chat panel embedded in the supervisor view that connects to a persistent Claude Code agent session running server-side. The supervisor types instructions (e.g. "dispatch the next 3 ready tickets", "what's blocking?"), the agent responds with streaming text, and tool calls (apm commands, git operations) are shown inline with approve/deny controls for risky actions.

Architecture outline:
- **WebSocket endpoint** (`/api/agent/ws`) on apm-server that manages a Claude Code Agent SDK session. On connect, initializes the agent with CLAUDE.md and agents.md as system context. Messages from the UI are forwarded to the agent; responses stream back as tokens.
- **Chat UI component** in SupervisorView — message list with markdown rendering, streaming response display, expandable tool-call blocks, approve/deny buttons for dangerous operations.
- **Session management** — agent session lives server-side with reconnect support and idle timeout cleanup.

The Claude Code Agent SDK (TypeScript) handles context management, tool execution, and streaming. The main work is the websocket plumbing, the chat UI component, and the tool-approval flow.

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
| 2026-04-06T01:41Z | — | new | philippepascal |