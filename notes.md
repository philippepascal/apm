Today (apm-core/src/config.toml & start.rs):

  [workers]
  command = "claude"
  args    = ["--print"]
  model   = "sonnet"

  // start.rs spawn paths:
  cmd.arg(&params.command);                  // "claude"          ← config
  for arg in &params.args { cmd.arg(arg); }  // "--print"         ← config
  cmd.args(["--model", model]);              // "sonnet"          ← config (transformed)
  cmd.args(["--output-format", "stream-json"]); //                ← HARDCODED
  cmd.arg("--verbose");                       //                  ← HARDCODED
  cmd.args(["--system-prompt", worker_system]); //                ← runtime value
  if skip_permissions { cmd.arg("--dangerously-skip-permissions"); } // ← runtime
  cmd.arg(ticket_content);                   //                   ← runtime

  So: command, args (one entry), model are configurable. Everything else is hardcoded. The --print/--output-format/--verbose trio is split — one in config, two in code — and the
  dependency between them (the trio is required together) is invisible to the user.

  Two categories worth distinguishing:

  ┌───────────────────────────┬──────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────────────────────────────────────┐
  │           Type            │ Current home │                    Examples                     │                           Where it should live                            │
  ├───────────────────────────┼──────────────┼─────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
  │ Static driver flags       │ mixed        │ --print, --output-format=stream-json, --verbose │ config (all together, visible)                                            │
  ├───────────────────────────┼──────────────┼─────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
  │ Runtime-substituted flags │ code         │ --system-prompt <text>, --model <value>         │ code (templates: code knows the substitution; config can rename the flag) │
  ├───────────────────────────┼──────────────┼─────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
  │ Per-invocation toggles    │ code         │ --dangerously-skip-permissions                  │ code (driven by apm start --spawn -P, not config)                         │
  ├───────────────────────────┼──────────────┼─────────────────────────────────────────────────┼───────────────────────────────────────────────────────────────────────────┤
  │ Positional args           │ code         │ <ticket_content> (the user message)             │ code                                                                      │
  └───────────────────────────┴──────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────────────────────────────────────┘

  The fix-shaped answer: put the static trio into config's args. Today's args = ["--print"] becomes args = ["--print", "--output-format=stream-json", "--verbose"]. The [workers]
  section then exposes the complete static contract with the driver. The trio dependency becomes user-visible.

  Code keeps the runtime substitutions (system-prompt, model, skip-permissions, ticket content) because those values are computed per-spawn — config can name the flag but can't
  supply the value.

  What this changes:

  1. apm init writes the new default args list (["--print", "--output-format=stream-json", "--verbose"]).
  2. Existing repos with args = ["--print"] keep working until they edit config — but their workers won't get transcript capture (only stdout) until they update args. The
  transcript-capture probe should detect this and warn (or fail if transcript capture is required).
  3. apm validate could check that args contain the trio — but that's opinionated; some users might intentionally drop transcript capture. Maybe warn, not error.
  4. The Claude-specific quirk knowledge moves out of code — the user owns what flags they pass. If Claude renames --verbose next year, edit config; no APM rebuild needed.

  What this doesn't fix:

  - The driver-agnosticism is still shallow. APM's .apm-worker.log parsing assumes the JSONL stream-json format. Switching to a non-Claude driver still requires code changes for
  parsing.
  - A user with bad config (missing required flag) gets the same crash-on-spawn we just fixed — config doesn't prevent the class of bug, just relocates the responsibility.

  Recommendation: worth doing as a small follow-up ticket. It's mostly a config-template change + removing two cmd.arg lines. Validate-warn on missing transcript-capture flags is
   optional polish.

  Want me to file the ticket?
