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
    Init,
    /// List tickets
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        unassigned: bool,
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
    /// Return the highest-priority actionable ticket
    Next {
        #[arg(long)]
        json: bool,
    },
    /// Sync with remote (poll events, detect merges)
    Sync,
}

fn repo_root() -> Result<PathBuf> {
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
    match cli.command {
        Command::Init => cmd::init::run(),
        Command::List { state, unassigned } => cmd::list::run(state, unassigned),
        Command::Show { id } => cmd::show::run(id),
        Command::New { title } => cmd::new::run(title),
        Command::State { id, state } => cmd::state::run(id, state),
        Command::Set { id, field, value } => cmd::set::run(id, field, value),
        Command::Next { json } => cmd::next::run(json),
        Command::Sync => cmd::sync::run(),
    }
}

mod cmd {
    pub mod init;
    pub mod list;
    pub mod show;
    pub mod new;
    pub mod state;
    pub mod set;
    pub mod next;
    pub mod sync;
}
