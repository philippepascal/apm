use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "apm", about = "Agent Project Manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize apm in the current repository
    Init {
        /// Skip updating .claude/settings.json allow list
        #[arg(long)]
        no_claude: bool,
        /// Migrate root-level apm.toml and apm.agents.md to .apm/
        #[arg(long)]
        migrate: bool,
    },
    /// List tickets
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        unassigned: bool,
        /// Include terminal-state tickets (e.g. closed)
        #[arg(long)]
        all: bool,
        #[arg(long)]
        supervisor: Option<String>,
        /// Show only tickets actionable by this actor (agent, supervisor, engineer)
        #[arg(long, value_name = "ACTOR")]
        actionable: Option<String>,
    },
    /// Show a ticket
    Show {
        id: u32,
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Create a new ticket
    New {
        title: String,
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
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Transition a ticket's state
    State {
        id: u32,
        state: String,
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Set a field on a ticket
    Set {
        id: u32,
        field: String,
        value: String,
    },
    /// Claim a ticket and check out its branch
    Start {
        /// Ticket ID; omit when using --next
        id: Option<u32>,
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
    Next {
        #[arg(long)]
        json: bool,
    },
    /// Sync with remote (poll events, detect merges)
    Sync {
        /// Skip git fetch; re-process local branches only
        #[arg(long)]
        offline: bool,
        /// Suppress non-error output
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        no_aggressive: bool,
        /// Automatically close accepted/stale tickets without prompting
        #[arg(long)]
        auto_close: bool,
        /// Automatically accept merged tickets without prompting
        #[arg(long)]
        auto_accept: bool,
    },
    /// Take over a ticket from another agent
    Take {
        id: u32,
        #[arg(long)]
        no_aggressive: bool,
    },
    /// List or remove permanent git worktrees
    Worktrees {
        /// Provision a permanent worktree for the given ticket ID (any state)
        #[arg(long, value_name = "ID")]
        add: Option<u32>,
        /// Remove the worktree for the given ticket ID
        #[arg(long, value_name = "ID")]
        remove: Option<u32>,
    },
    /// Supervisor: edit ticket spec and optionally transition state
    Review {
        id: u32,
        /// Transition to this state after editing (skips interactive prompt)
        #[arg(long, value_name = "STATE")]
        to: Option<String>,
        #[arg(long)]
        no_aggressive: bool,
    },
    /// Check ticket and cache integrity
    Verify {
        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,
    },
    /// Validate config and ticket integrity
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
    },
    /// Internal git hook dispatcher (used by .git/hooks/*)
    #[command(name = "_hook")]
    Hook {
        hook_name: String,
        /// Extra args passed by git (remote, url) — ignored
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        _extra: Vec<String>,
    },
    /// Print agent instructions from apm.agents.md
    Agents,
    /// Orchestrate workers: dispatch apm start --next --spawn in a loop
    Work {
        /// Pass --dangerously-skip-permissions to spawned workers
        #[arg(long, short = 'P')]
        skip_permissions: bool,
        /// Print which tickets would be started without dispatching
        #[arg(long)]
        dry_run: bool,
    },
    /// Force-close a ticket from any state (supervisor only)
    Close {
        id: u32,
        /// Optional reason appended to the history entry
        #[arg(long)]
        reason: Option<String>,
    },
    /// Remove worktrees and local branches for closed tickets
    Clean {
        /// Print what would be removed without modifying anything
        #[arg(long)]
        dry_run: bool,
    },
    /// Read or write individual spec sections of a ticket
    Spec {
        id: u32,
        /// Section name (e.g. "Problem", "Approach")
        #[arg(long)]
        section: Option<String>,
        /// New content for the section; use "-" to read from stdin
        #[arg(long)]
        set: Option<String>,
        /// Check that all required sections are non-empty
        #[arg(long)]
        check: bool,
        /// Mark the first unchecked item matching this text in --section as done
        #[arg(long)]
        mark: Option<String>,
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
        Command::Init { no_claude, migrate } => cmd::init::run(&root, no_claude, migrate),
        Command::List { state, unassigned, all, supervisor, actionable } => cmd::list::run(&root, state, unassigned, all, supervisor, actionable),
        Command::Show { id, no_aggressive } => cmd::show::run(&root, id, no_aggressive),
        Command::New { title, no_edit, side_note, context, context_section, no_aggressive } => cmd::new::run(&root, title, no_edit, side_note, context, context_section, no_aggressive),
        Command::State { id, state, no_aggressive } => cmd::state::run(&root, id, state, no_aggressive),
        Command::Set { id, field, value } => cmd::set::run(&root, id, field, value),
        Command::Next { json } => cmd::next::run(&root, json),
        Command::Start { id, no_aggressive, spawn, skip_permissions, next } => {
            match (next, id) {
                (true, Some(_)) => anyhow::bail!("--next and an explicit ID are mutually exclusive"),
                (true, None) => cmd::start::run_next(&root, no_aggressive, spawn, skip_permissions),
                (false, Some(id)) => cmd::start::run(&root, id, no_aggressive, spawn, skip_permissions),
                (false, None) => anyhow::bail!("provide a ticket ID or use --next"),
            }
        }
        Command::Sync { offline, quiet, no_aggressive, auto_close, auto_accept } => cmd::sync::run(&root, offline, quiet, no_aggressive, auto_close, auto_accept),
        Command::Take { id, no_aggressive } => cmd::take::run(&root, id, no_aggressive),
        Command::Worktrees { add, remove } => cmd::worktrees::run(&root, add, remove),
        Command::Review { id, to, no_aggressive } => cmd::review::run(&root, id, to, no_aggressive),
        Command::Verify { fix } => cmd::verify::run(&root, fix),
        Command::Validate { fix, json, config_only } => cmd::validate::run(&root, fix, json, config_only),
        Command::Hook { hook_name, .. } => { cmd::hook::run(&root, &hook_name); Ok(()) }
        Command::Agents => cmd::agents::run(&root),
        Command::Work { skip_permissions, dry_run } => cmd::work::run(&root, skip_permissions, dry_run),
        Command::Close { id, reason } => cmd::close::run(&root, id, reason),
        Command::Clean { dry_run } => cmd::clean::run(&root, dry_run),
        Command::Spec { id, section, set, check, mark } => cmd::spec::run(&root, id, section, set, check, mark),
    }
}

use apm::cmd;
