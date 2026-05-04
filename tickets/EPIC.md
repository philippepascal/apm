# EPIC.md — per-epic context document

Each epic branch writes its own `tickets/EPIC.md` with the epic title and any
notes added by the supervisor. APM reads it from the epic branch to build the
context bundle injected into worker prompts for tickets that belong to that epic.

The copy on `main` is this placeholder and is never read by APM at runtime.
The `tickets/EPIC.md merge=ours` rule in `.gitattributes` prevents merge
conflicts when multiple epics are open simultaneously.
