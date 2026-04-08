use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "apm",
    about = "Agent Project Manager",
    long_about = "Agent Project Manager — a git-native ticket system for human+AI teams.

Tickets live as Markdown files on per-ticket branches. State is stored in
TOML frontmatter; the state machine is defined in .apm/apm.toml.

Workflow states (typical path):
  new → in_design → specd → ready → in_progress → implemented → closed

Side paths:
  * ammend  — supervisor requests spec changes (from specd)
  * blocked — agent is stuck, needs a supervisor decision (from in_progress)
  * question — spec author needs clarification (from in_design)

Actors:
  * agent      — autonomous worker; picks up `ready` tickets via `apm next`
  * supervisor — human reviewer; approves specs, reviews implementations
  * engineer   — human developer; may do either role

Common entry points:
  apm next       — for agents: find the highest-priority actionable ticket
  apm list       — for humans: browse all tickets
  apm start <id> — claim a ticket and provision its worktree"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum EpicCommand {
    /// Create a new epic branch
    New {
        /// Title for the epic
        title: String,
    },
    /// Open a PR from the epic branch to the default branch
    Close {
        /// Epic ID (4–8 char hex prefix)
        id: String,
    },
    /// List all epics with derived state and ticket counts
    List,
    /// Show an epic and its tickets
    Show {
        /// Epic ID (4–8 char hex prefix)
        id: String,
        /// Skip automatic git fetch before reading data
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Set a field on an epic (e.g. max_workers)
    Set {
        /// Epic ID (4–8 char hex prefix)
        id: String,
        /// Field to update (e.g. max_workers)
        field: String,
        /// New value (use "-" to clear)
        value: String,
    },
}

#[derive(Subcommand)]
enum Command {
    /// Initialize apm in the current repository
    #[command(long_about = "Initialize apm in the current repository.

Creates the .apm/ directory containing:
  * apm.toml      — project config and state-machine definition
  * apm.agents.md — agent onboarding instructions

Also installs git hooks (.git/hooks/post-merge, post-checkout) so apm can
detect branch merges automatically.

Unless --no-claude is passed, adds apm commands to .claude/settings.json
so that Claude Code's allow list does not prompt for every apm call.

Use --migrate if you have an existing root-level apm.toml and apm.agents.md
that need to be moved into .apm/.")]
    Init {
        /// Skip updating .claude/settings.json allow list
        #[arg(long)]
        no_claude: bool,
        /// Migrate root-level apm.toml and apm.agents.md to .apm/
        #[arg(long)]
        migrate: bool,
        /// Generate .apm/Dockerfile.apm-worker and print build instructions
        #[arg(long)]
        with_docker: bool,
    },
    /// List tickets
    #[command(long_about = "List tickets (read-only query).

All filter flags are combinable. By default, tickets in terminal states
(closed, etc.) are hidden; pass --all to include them.

Examples:
  apm list                          # all non-closed tickets
  apm list --state ready            # only tickets awaiting an agent
  apm list --unassigned             # no agent assigned yet
  apm list --actionable agent       # tickets an agent can act on now
  apm list --all                    # everything including closed
  apm list --mine                   # only your tickets
  apm list --author alice           # only tickets by alice")]
    List {
        /// Filter by state (e.g. new, ready, in_progress, implemented, closed)
        #[arg(long)]
        state: Option<String>,
        /// Show only tickets with no agent assigned
        #[arg(long)]
        unassigned: bool,
        /// Include terminal-state tickets (e.g. closed)
        #[arg(long)]
        all: bool,
        /// Show only tickets actionable by this actor (agent, supervisor, engineer)
        #[arg(long, value_name = "ACTOR")]
        actionable: Option<String>,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Show only tickets authored by the current user
        #[arg(long)]
        mine: bool,
        /// Show only tickets authored by USERNAME
        #[arg(long, value_name = "USERNAME", conflicts_with = "mine")]
        author: Option<String>,
        /// Show only tickets owned by USERNAME (owner field)
        #[arg(long, value_name = "USERNAME", conflicts_with = "mine")]
        owner: Option<String>,
    },
    /// Show a ticket
    #[command(long_about = "Show the full content of a ticket.

Reads the ticket file directly from its branch blob in the git object store,
so the working tree does not need to be checked out on that branch.

By default, `apm show` fetches the latest remote state first. Pass
--no-aggressive to skip the fetch (faster for scripts or offline use).

The ticket ID can be supplied as:
  * a plain integer (e.g. 42 → pads to 0042)
  * a 4+ char hex prefix (e.g. 00ab)
  * the full 8-char hex ID")]
    Show {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Open the ticket in $VISUAL / $EDITOR (falls back to vi) instead of printing to stdout
        #[arg(long)]
        edit: bool,
    },
    /// Create a new ticket
    #[command(long_about = "Create a new ticket and its branch.

Creates a ticket Markdown file on a new branch (ticket/<id>-<slug>) and
opens $EDITOR so you can fill in the spec immediately.

Agents must always pass --no-edit to skip the interactive editor:
  apm new --no-edit \"Short title\"

Use --side-note during implementation to capture an out-of-scope observation
without interrupting the current ticket:
  apm new --side-note \"Spotted issue\" --context \"What was observed\"

After creating a ticket the typical next step is:
  apm state <id> in_design   # claim the spec for writing")]
    New {
        /// Short title for the ticket
        #[arg(value_name = "TITLE")]
        title: String,
        /// Skip opening $EDITOR after creation
        #[arg(long)]
        no_edit: bool,
        /// Mark this ticket as a side-note (out-of-scope observation)
        #[arg(long)]
        side_note: bool,
        /// Context to insert into a ticket section
        #[arg(long)]
        context: Option<String>,
        /// Section to route --context into (defaults to first tickets.sections entry or "Problem")
        #[arg(long)]
        context_section: Option<String>,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Section name to pre-populate (repeat paired with --set)
        #[arg(long, value_name = "NAME")]
        section: Vec<String>,
        /// Content for the section named by the preceding --section (repeat paired with --section)
        #[arg(long, value_name = "TEXT")]
        set: Vec<String>,
        /// Epic ID (8 hex chars); ticket branch will be created from epic/<ID>-* tip
        #[arg(long, value_name = "ID")]
        epic: Option<String>,
        /// Comma-separated ticket IDs this ticket depends on (repeatable)
        #[arg(long, value_name = "IDS")]
        depends_on: Vec<String>,
    },
    /// Transition a ticket's state
    #[command(long_about = "Transition a ticket to a new state.

Valid target states depend on the ticket's current state. The allowed
transitions are defined in .apm/apm.toml under [[workflow.states]].
Illegal transitions are rejected with an error.

Run `apm show <id>` first to check the current state, then choose a
target from the edges listed for that state in apm.toml.

Use --force to bypass the transition rules (escape hatch for stuck tickets).
The target state must still exist in the config; document-level validations
(spec completeness, unchecked criteria) are still enforced.

Examples:
  apm state 42 in_design       # claim a new ticket for spec writing
  apm state 42 specd           # submit spec for supervisor review
  apm state 42 implemented     # mark implementation done (open PR first)
  apm state 42 new --force     # reset a stuck in_design ticket
  apm state 42 ready --force   # reset a stuck in_progress ticket")]
    State {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Target state (e.g. in_design, specd, ready, in_progress, implemented, closed)
        #[arg(value_name = "STATE")]
        state: String,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Bypass transition rules (escape hatch for stuck tickets)
        #[arg(long)]
        force: bool,
    },
    /// Set a field on a ticket
    #[command(long_about = "Set a metadata field on a ticket.

Valid field names:
  priority    — integer; higher = picked first by `apm next`
  effort      — integer 1-10; implementation scale estimate
  risk        — integer 1-10; technical risk estimate
  title       — short human-readable summary
  agent       — name of the assigned agent (use \"-\" to clear)
  branch      — override the ticket's branch name (use \"-\" to clear)
  depends_on  — comma-separated list of blocker IDs (use \"-\" to clear)

Examples:
  apm set 42 priority 5
  apm set 42 agent alice
  apm set 42 agent -               # clear agent field
  apm set 42 depends_on abc123     # single blocker
  apm set 42 depends_on \"abc123,def456\"  # multiple blockers
  apm set 42 depends_on -          # clear depends_on")]
    Set {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Field to update: priority, effort, risk, title, agent, branch, depends_on
        #[arg(value_name = "FIELD")]
        field: String,
        /// New value for the field (use "-" to clear agent/branch)
        #[arg(value_name = "VALUE")]
        value: String,
        /// Skip automatic git fetch/push
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Claim a ticket and check out its branch
    #[command(long_about = "Claim a ticket and provision its permanent worktree.

Sets the ticket's agent field to $APM_AGENT_NAME and transitions state to
in_progress, then provisions (or reuses) a permanent git worktree for the
ticket branch. Prints the worktree path so the caller can cd into it or use
`git -C <path>` for all subsequent git operations.

--spawn launches a Claude Code subprocess that picks up the ticket
autonomously. The subprocess receives the project allow list by default;
add -P to also pass --dangerously-skip-permissions.

--next auto-selects the highest-priority actionable ticket; mutually
exclusive with an explicit ID.

Examples:
  apm start 42                   # claim ticket 42
  apm start --next               # claim whatever apm next would return
  apm start --spawn 42           # hand ticket 42 to a background agent
  apm start --spawn --next -P    # background agent, skip permissions")]
    Start {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer); omit when using --next
        id: Option<String>,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Launch a claude worker subprocess in the background
        #[arg(long)]
        spawn: bool,
        /// Pass --dangerously-skip-permissions to the worker (use with --spawn)
        #[arg(long, short = 'P')]
        skip_permissions: bool,
        /// Auto-select the highest-priority actionable ticket
        #[arg(long)]
        next: bool,
    },
    /// Return the highest-priority actionable ticket
    #[command(long_about = "Return the highest-priority ticket actionable right now.

Considers only tickets in states that the current actor can act on (agent
by default). Selects by priority descending, then by id ascending as a
tiebreaker.

Returns nothing (exit 0, empty output) when there is no actionable ticket.

--json outputs the result as a JSON object — useful in agent startup loops:
  apm next --json   # {\"id\": \"0042\", \"title\": \"...\", \"state\": \"ready\", ...}

Typical agent startup sequence:
  apm sync
  apm next --json   # check for work
  apm start --next  # claim and provision in one step")]
    Next {
        /// Output result as JSON instead of human-readable text
        #[arg(long)]
        json: bool,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Sync with remote (poll events, detect merges)
    #[command(long_about = "Fetch from remote and reconcile the local ticket cache.

What sync does:
  1. git fetch (unless --offline)
  2. Detects ticket branches that have been merged into main
  3. For each merged branch, closes the ticket immediately; use --auto-close
     to skip the confirmation prompt in CI
  4. Updates the local branch cache

Run sync at the start of each agent session to ensure local state reflects
what has happened on the remote since last time.

Examples:
  apm sync                    # interactive, fetch from remote
  apm sync --offline          # re-process local branches only
  apm sync --auto-close       # close all merged tickets silently
  apm sync --quiet            # suppress non-error output")]
    Sync {
        /// Skip git fetch; re-process local branches only
        #[arg(long)]
        offline: bool,
        /// Suppress non-error output
        #[arg(long)]
        quiet: bool,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
        /// Automatically close merged/stale tickets without prompting
        #[arg(long)]
        auto_close: bool,
    },
    /// Assign a ticket to an owner
    #[command(long_about = "Set the owner field on any ticket, regardless of its current state.

Use this to assign a ticket to a user or agent, or to clear the owner field.

Examples:
  apm assign 42 alice        # assign ticket 42 to alice
  apm assign 42 -            # clear the owner field")]
    Assign {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Username to assign (use \"-\" to clear)
        #[arg(value_name = "USERNAME")]
        username: String,
        /// Skip automatic git fetch/push
        #[arg(long)]
        no_aggressive: bool,
    },
    /// List or remove permanent git worktrees
    #[command(long_about = "Manage permanent git worktrees for ticket branches.

APM uses permanent worktrees (in the apm--worktrees/ sibling directory by
default) so that agents can work on a ticket branch without disturbing the
main working tree. These worktrees survive `apm sync` and are reused across
sessions.

--add is idempotent: if the worktree already exists it just prints the path.
Always use `apm worktrees --add <id>` rather than `git worktree add` by hand,
so the path is recorded in the ticket's metadata.

Examples:
  apm worktrees              # list all known worktrees
  apm worktrees --add 42     # provision worktree for ticket 42, print path
  apm worktrees --remove 42  # remove the worktree for ticket 42")]
    Worktrees {
        /// Remove the worktree for the given ticket ID
        #[arg(long, value_name = "ID")]
        remove: Option<String>,
    },
    /// Supervisor: edit ticket spec and optionally transition state
    #[command(long_about = "Supervisor command: review and edit a ticket spec, then transition state.

Opens $EDITOR on the ticket file so the supervisor can read the spec, leave
feedback in amendment-request boxes, or update acceptance criteria. After
the editor closes, prompts for a state transition unless --to is supplied.

Common review flows:
  apm review 42 --to specd      # approve spec as-is
  apm review 42 --to ammend     # request changes (fill in amendment boxes first)
  apm review 42 --to ready      # approve and queue for implementation
  apm review 42 --to implemented  # accept implementation

--to skips the interactive prompt — useful in scripts or when the transition
is already decided before opening the editor.")]
    Review {
        /// Ticket ID to review (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Transition to this state after editing (skips interactive prompt)
        #[arg(long, value_name = "STATE")]
        to: Option<String>,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Check ticket and cache integrity
    #[command(long_about = "Check ticket and local cache integrity.

Scans for inconsistencies between the local branch cache and what is on
disk: dangling worktrees, branches missing ticket files, cache entries that
do not match the branch blob, etc.

--fix attempts automatic repairs where safe (removing stale cache entries,
re-indexing branches). Anything it cannot fix is reported for manual
attention.

Run this if `apm list` or `apm show` is behaving unexpectedly.")]
    Verify {
        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Validate config and ticket integrity
    #[command(long_about = "Validate apm.toml correctness and cross-ticket integrity.

Checks performed:
  * apm.toml parses without errors
  * All state transitions reference known states
  * Every ticket's branch field matches its actual branch name
  * No two tickets share the same branch

--fix repairs branch-field mismatches automatically and re-commits the
ticket file on its branch.

--json outputs the full results as JSON — useful in CI pipelines:
  apm validate --json | jq '.errors'

--config-only skips per-ticket checks and validates only the config file.")]
    Validate {
        /// Auto-fix repairable issues (branch field mismatches)
        #[arg(long)]
        fix: bool,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Run only config cross-checks, skip ticket integrity checks
        #[arg(long)]
        config_only: bool,
        /// Skip automatic git fetch before reading ticket data
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Internal git hook dispatcher (used by .git/hooks/*)
    #[command(name = "_hook", hide = true)]
    Hook {
        /// Name of the git hook being dispatched (e.g. post-merge)
        #[arg(value_name = "HOOK")]
        hook_name: String,
        /// Extra args passed by git (remote, url) — ignored
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        _extra: Vec<String>,
    },
    /// Print agent instructions from apm.agents.md
    #[command(long_about = "Print the contents of apm.agents.md to stdout.

Useful for onboarding a new agent subprocess: pipe or paste the output into
the agent's context so it knows the workflow, branch conventions, and shell
discipline rules without needing file-system access to the repo.

Example:
  apm agents | pbcopy          # copy to clipboard
  apm agents > /tmp/agents.md  # write to a temp file for injection")]
    Agents,
    /// Orchestrate workers: dispatch apm start --next --spawn in a loop
    #[command(long_about = "Orchestration loop: repeatedly dispatch agents until no work remains.

Calls `apm start --next --spawn` in a loop, launching one Claude subprocess
per actionable ticket, until `apm next` returns null (no more tickets).

--dry-run prints the ticket IDs that would be started without actually
spawning any subprocesses — useful to preview the work queue.

-P passes --dangerously-skip-permissions to every spawned worker.

--daemon keeps the process alive after the queue is exhausted, polling at
--interval seconds (default 30) and dispatching new workers as slots open
or tickets become actionable. Ctrl-C stops the daemon; already-running
workers continue independently.

Example:
  apm work --dry-run           # preview
  apm work                     # run with normal permissions
  apm work -P                  # run with skipped permissions
  apm work --daemon            # run forever, poll every 30s
  apm work --daemon --interval 60  # poll every 60s")]
    Work {
        /// Pass --dangerously-skip-permissions to spawned workers
        #[arg(long, short = 'P')]
        skip_permissions: bool,
        /// Print which tickets would be started without dispatching
        #[arg(long)]
        dry_run: bool,
        /// Keep running after the queue is exhausted; re-check as slots open
        #[arg(long, short = 'd')]
        daemon: bool,
        /// Poll interval in seconds when running as a daemon (default: 30)
        #[arg(long, default_value = "30")]
        interval: u64,
        /// Restrict dispatching to tickets in this epic (8-char ID)
        #[arg(long, value_name = "EPIC_ID")]
        epic: Option<String>,
    },
    /// Force-close a ticket from any state (supervisor only)
    #[command(long_about = "Force-close a ticket from any state (supervisor only).

Bypasses the normal state machine and closes the ticket immediately,
regardless of current state. An optional reason is appended to the ticket's
## History section for the record.

This is an escape hatch for tickets that are abandoned, duplicated, or
otherwise need to be removed from the active queue without following the
normal flow. Prefer the standard `apm state <id> closed` transition when
the ticket has been properly resolved.

Example:
  apm close 42 --reason \"duplicate of #38\"")]
    Close {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        id: String,
        /// Optional reason appended to the history entry
        #[arg(long)]
        reason: Option<String>,
        /// Skip automatic git fetch/push
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Move closed ticket files to the archive directory
    #[command(long_about = "Move terminal-state ticket files from tickets/ to the configured archive_dir.

Requires `archive_dir` under the [tickets] section of .apm/config.toml:

  [tickets]
  archive_dir = \"archive/tickets\"

Examples:
  apm archive                        # archive all closed tickets
  apm archive --dry-run              # preview which files would be moved
  apm archive --older-than 30d       # archive only tickets updated >30 days ago
  apm archive --older-than 2026-01-01  # ISO date threshold")]
    Archive {
        /// Print which files would be moved without modifying any branches
        #[arg(long)]
        dry_run: bool,
        /// Only archive tickets whose updated_at is older than this threshold (e.g. \"30d\" or \"2026-01-01\")
        #[arg(long, value_name = "THRESHOLD")]
        older_than: Option<String>,
    },
    /// Remove worktrees and local branches for closed tickets
    #[command(long_about = "Remove worktrees (and optionally branches) for terminal-state tickets.

Default (no flags): removes worktrees only. Local and remote branches are
never touched without an explicit flag.

  apm clean                              # remove worktrees only
  apm clean --dry-run                    # preview worktree removals
  apm clean --branches                   # also delete local ticket/* branches
  apm clean --branches --dry-run         # preview worktrees + branches
  apm clean --untracked                  # also remove untracked non-temp files
  apm clean --force                      # bypass merge/divergence checks
  apm clean --remote --older-than 30d    # delete remote branches older than 30 days
  apm clean --remote --older-than 2026-01-01  # ISO date threshold
  apm clean --remote --older-than 30d --yes   # skip per-branch confirmation
  apm clean --remote --older-than 30d --dry-run  # preview remote deletions

Known temp files (.apm-worker.pid, .apm-worker.log, pr-body.md, body.md,
ac.txt) are always removed automatically without needing --untracked.")]
    Clean {
        /// Print what would be removed without modifying anything
        #[arg(long)]
        dry_run: bool,
        /// Skip per-branch confirmation prompts (used with --remote)
        #[arg(long, short = 'y')]
        yes: bool,
        /// Bypass merge and divergence checks; always prompts before each removal
        #[arg(long)]
        force: bool,
        /// Also delete local ticket/* branches (default: worktrees only)
        #[arg(long)]
        branches: bool,
        /// Delete remote ticket/* branches in terminal states older than --older-than
        #[arg(long)]
        remote: bool,
        /// Age threshold for --remote: e.g. "30d" or "2026-01-01" (YYYY-MM-DD)
        #[arg(long, value_name = "THRESHOLD", requires = "remote")]
        older_than: Option<String>,
        /// Remove untracked non-temp files from worktrees before removal
        #[arg(long)]
        untracked: bool,
    },
    /// List and manage running worker processes
    Workers {
        /// Tail the worker log for the given ticket ID
        #[arg(long, value_name = "ID")]
        log: Option<String>,
        /// Kill the worker for the given ticket ID
        #[arg(long, value_name = "ID")]
        kill: Option<String>,
    },
    /// Manage epics
    Epic {
        #[command(subcommand)]
        command: EpicCommand,
    },
    /// Read or write individual spec sections of a ticket
    #[command(long_about = "Read or write individual sections of a ticket's spec.

--section alone reads the named section and prints it to stdout:
  apm spec 42 --section Problem

--section combined with --set writes new content to that section (use \"-\"
to read the new content from stdin):
  apm spec 42 --section Approach --set \"New approach text\"
  echo \"text\" | apm spec 42 --section Approach --set -

--check validates that all required sections defined in apm.toml are
present and non-empty:
  apm spec 42 --check

--mark checks off the first unchecked item in --section whose text contains
the given substring:
  apm spec 42 --section \"Acceptance criteria\" --mark \"output is JSON\"")]
    Spec {
        /// Ticket ID (8-char hex, 4+ char prefix, or plain integer)
        #[arg(value_name = "ID")]
        id: String,
        /// Section name (e.g. "Problem", "Approach")
        #[arg(long)]
        section: Option<String>,
        /// New content for the section; use "-" to read from stdin
        #[arg(long, allow_hyphen_values = true)]
        set: Option<String>,
        /// Read new section content from this file
        #[arg(long, value_name = "PATH", conflicts_with = "set")]
        set_file: Option<String>,
        /// Check that all required sections are non-empty
        #[arg(long)]
        check: bool,
        /// Mark the first unchecked item matching this text in --section as done
        #[arg(long)]
        mark: Option<String>,
        /// Skip automatic git fetch/push
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Generate a one-time password for device registration (requires apm-server)
    Register {
        /// Username to register (defaults to GitHub username)
        username: Option<String>,
    },
    /// List active sessions (requires apm-server)
    Sessions,
    /// Revoke sessions (requires apm-server)
    Revoke {
        /// Username whose sessions to revoke (required unless --all)
        #[arg(value_name = "USERNAME")]
        username: Option<String>,
        /// Only revoke sessions matching this device hint
        #[arg(long, value_name = "HINT")]
        device: Option<String>,
        /// Revoke all sessions for all users
        #[arg(long, conflicts_with = "device")]
        all: bool,
    },
}

pub fn repo_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git not found")?;
    if !output.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = repo_root()?;
    if let Ok(ref config) = apm_core::config::Config::load(&root) {
        if config.logging.enabled {
            let log_path = apm_core::logger::resolve_log_path(
                &config.project.name,
                config.logging.file.as_deref(),
            );
            if let Some(parent) = log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let agent = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".to_string());
            apm_core::logger::init(&root, &log_path, &agent);
        }
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    apm_core::logger::log("cmd", &args.join(" "));
    match cli.command {
        Command::Init { no_claude, migrate, with_docker } => cmd::init::run(&root, no_claude, migrate, with_docker),
        Command::List { state, unassigned, all, actionable, no_aggressive, mine, author, owner } => cmd::list::run(&root, state, unassigned, all, actionable, no_aggressive, mine, author, owner),
        Command::New { title, no_edit, side_note, context, context_section, no_aggressive, section, set, epic, depends_on } => cmd::new::run(&root, title, no_edit, side_note, context, context_section, no_aggressive, section, set, epic, depends_on),
        Command::Show { id, no_aggressive, edit } => cmd::show::run(&root, &id, no_aggressive, edit),
        Command::State { id, state, no_aggressive, force } => cmd::state::run(&root, &id, state, no_aggressive, force),
        Command::Set { id, field, value, no_aggressive } => cmd::set::run(&root, &id, field, value, no_aggressive),
        Command::Next { json, no_aggressive } => cmd::next::run(&root, json, no_aggressive),
        Command::Start { id, no_aggressive, spawn, skip_permissions, next } => {
            match (next, id) {
                (true, Some(_)) => anyhow::bail!("--next and an explicit ID are mutually exclusive"),
                (true, None) => cmd::start::run_next(&root, no_aggressive, spawn, skip_permissions),
                (false, Some(id)) => {
                    let agent_name = apm_core::start::resolve_caller_name();
                    cmd::start::run(&root, &id, no_aggressive, spawn, skip_permissions, &agent_name)
                }
                (false, None) => anyhow::bail!("provide a ticket ID or use --next"),
            }
        }
        Command::Sync { offline, quiet, no_aggressive, auto_close } => cmd::sync::run(&root, offline, quiet, no_aggressive, auto_close),
        Command::Assign { id, username, no_aggressive } => cmd::assign::run(&root, &id, &username, no_aggressive),
        Command::Worktrees { remove } => cmd::worktrees::run(&root, remove.as_deref()),
        Command::Review { id, to, no_aggressive } => cmd::review::run(&root, &id, to, no_aggressive),
        Command::Verify { fix, no_aggressive } => cmd::verify::run(&root, fix, no_aggressive),
        Command::Validate { fix, json, config_only, no_aggressive } => cmd::validate::run(&root, fix, json, config_only, no_aggressive),
        Command::Hook { hook_name, .. } => { cmd::hook::run(&root, &hook_name); Ok(()) }
        Command::Agents => cmd::agents::run(&root),
        Command::Work { skip_permissions, dry_run, daemon, interval, epic } => cmd::work::run(&root, skip_permissions, dry_run, daemon, interval, epic),
        Command::Close { id, reason, no_aggressive } => cmd::close::run(&root, &id, reason, no_aggressive),
        Command::Archive { dry_run, older_than } => cmd::archive::run(&root, dry_run, older_than),
        Command::Clean { dry_run, yes, force, branches, remote, older_than, untracked } => cmd::clean::run(&root, dry_run, yes, force, branches, remote, older_than, untracked),
        Command::Spec { id, section, set, set_file, check, mark, no_aggressive } => cmd::spec::run(&root, &id, section, set, set_file, check, mark, no_aggressive),
        Command::Workers { log, kill } => cmd::workers::run(&root, log.as_deref(), kill.as_deref()),
        Command::Epic { command: EpicCommand::New { title } } => cmd::epic::run_new(&root, title),
        Command::Epic { command: EpicCommand::Close { id } } => cmd::epic::run_close(&root, &id),
        Command::Epic { command: EpicCommand::List } => cmd::epic::run_list(&root),
        Command::Epic { command: EpicCommand::Show { id, no_aggressive } } => cmd::epic::run_show(&root, &id, no_aggressive),
        Command::Epic { command: EpicCommand::Set { id, field, value } } => cmd::epic::run_set(&root, &id, &field, &value),
        Command::Register { username } => {
            let inferred = username.is_none();
            let config = apm_core::config::Config::load(&root)?;
            let username = username.unwrap_or_else(|| {
                apm_core::config::try_github_username(&config.git_host)
                    .expect("could not detect GitHub username; pass one explicitly")
            });
            cmd::register::run(&root, &username, inferred)
        }
        Command::Sessions => cmd::sessions::run(&root),
        Command::Revoke { username, device, all } => {
            if !all && username.is_none() {
                eprintln!("error: provide a username or use --all");
                std::process::exit(1);
            }
            cmd::revoke::run(&root, username.as_deref(), device.as_deref(), all)
        }
    }
}

use apm::cmd;
