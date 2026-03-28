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
    Show { id: u32 },
    /// Create a new ticket
    New {
        title: String,
        #[arg(long)]
        no_edit: bool,
    },
    /// Transition a ticket's state
    State { id: u32, state: String },
    /// Set a field on a ticket
    Set {
        id: u32,
        field: String,
        value: String,
    },
    /// Claim a ticket and check out its branch
    Start { id: u32 },
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
    },
    /// Take over a ticket from another agent
    Take { id: u32 },
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
    },
    /// Check ticket and cache integrity
    Verify {
        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,
    },
    /// Internal git hook dispatcher (used by .git/hooks/*)
    #[command(name = "_hook")]
    Hook { hook_name: String },
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
    match cli.command {
        Command::Init { no_claude } => cmd::init::run(&root, no_claude),
        Command::List { state, unassigned, all, supervisor, actionable } => cmd::list::run(&root, state, unassigned, all, supervisor, actionable),
        Command::Show { id } => cmd::show::run(&root, id),
        Command::New { title, no_edit } => cmd::new::run(&root, title, no_edit),
        Command::State { id, state } => cmd::state::run(&root, id, state),
        Command::Set { id, field, value } => cmd::set::run(&root, id, field, value),
        Command::Next { json } => cmd::next::run(&root, json),
        Command::Start { id } => cmd::start::run(&root, id),
        Command::Sync { offline, quiet } => cmd::sync::run(&root, offline, quiet),
        Command::Take { id } => cmd::take::run(&root, id),
        Command::Worktrees { add, remove } => cmd::worktrees::run(&root, add, remove),
        Command::Review { id, to } => cmd::review::run(&root, id, to),
        Command::Verify { fix } => cmd::verify::run(&root, fix),
        Command::Hook { hook_name } => { cmd::hook::run(&root, &hook_name); Ok(()) }
        Command::Agents => cmd::agents::run(&root),
    }
}

use apm::cmd;
