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
        /// Context to insert into the Problem section
        #[arg(long)]
        context: Option<String>,
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
        id: u32,
        #[arg(long)]
        no_aggressive: bool,
        /// Launch a claude worker subprocess in the background
        #[arg(long)]
        spawn: bool,
        /// Pass --dangerously-skip-permissions to the worker (use with --spawn)
        #[arg(long, short = 'P')]
        skip_permissions: bool,
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
        Command::New { title, no_edit, side_note, context, no_aggressive } => cmd::new::run(&root, title, no_edit, side_note, context, no_aggressive),
        Command::State { id, state, no_aggressive } => cmd::state::run(&root, id, state, no_aggressive),
        Command::Set { id, field, value } => cmd::set::run(&root, id, field, value),
        Command::Next { json } => cmd::next::run(&root, json),
        Command::Start { id, no_aggressive, spawn, skip_permissions } => cmd::start::run(&root, id, no_aggressive, spawn, skip_permissions),
        Command::Sync { offline, quiet, no_aggressive, auto_close } => cmd::sync::run(&root, offline, quiet, no_aggressive, auto_close),
        Command::Take { id, no_aggressive } => cmd::take::run(&root, id, no_aggressive),
        Command::Worktrees { add, remove } => cmd::worktrees::run(&root, add, remove),
        Command::Review { id, to, no_aggressive } => cmd::review::run(&root, id, to, no_aggressive),
        Command::Verify { fix } => cmd::verify::run(&root, fix),
        Command::Validate { fix, json } => cmd::validate::run(&root, fix, json),
        Command::Hook { hook_name, .. } => { cmd::hook::run(&root, &hook_name); Ok(()) }
        Command::Agents => cmd::agents::run(&root),
    }
}

use apm::cmd;
