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
    },
    /// Show a ticket
    Show { id: u32 },
    /// Create a new ticket
    New { title: String },
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
        Command::List { state, unassigned, all, supervisor } => cmd::list::run(&root, state, unassigned, all, supervisor),
        Command::Show { id } => cmd::show::run(&root, id),
        Command::New { title } => cmd::new::run(&root, title),
        Command::State { id, state } => cmd::state::run(&root, id, state),
        Command::Set { id, field, value } => cmd::set::run(&root, id, field, value),
        Command::Next { json } => cmd::next::run(&root, json),
        Command::Start { id } => cmd::start::run(&root, id),
        Command::Sync { offline, quiet } => cmd::sync::run(&root, offline, quiet),
        Command::Agents => cmd::agents::run(&root),
    }
}

use apm::cmd;
