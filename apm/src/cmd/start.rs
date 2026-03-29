use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id: u32, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    let agent_name = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    // apm start is only valid from "ready" — spec-writing states (new, ammend)
    // use the branch directly; blocked tickets go back to ready before restarting.
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();

    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };

    let fm = &t.frontmatter;
    if fm.agent.is_some() {
        bail!("ticket already claimed — run `apm next`");
    }
    if !startable.is_empty() && !startable.contains(&fm.state.as_str()) {
        bail!(
            "ticket #{id} is in state {:?} — not startable\n\
             Use `apm start` only from: {}",
            fm.state,
            startable.join(", ")
        );
    }

    let now = Utc::now();
    let old_state = t.frontmatter.state.clone();
    t.frontmatter.agent = Some(agent_name.clone());
    t.frontmatter.state = "in_progress".into();
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    super::state::append_history(&mut t.body, &old_state, "in_progress", &when, &agent_name);

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    let default_branch = &config.project.default_branch;

    if aggressive {
        if let Err(e) = git::fetch_branch(root, &branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
        if let Err(e) = git::fetch_branch(root, default_branch) {
            eprintln!("warning: fetch {} failed: {e:#}", default_branch);
        }
    }

    git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): start — {old_state} → in_progress"))?;

    // Provision permanent worktree.
    // Worktree dir name: ticket-<id>-<slug> (branch name with / replaced by -)
    let wt_name = branch.replace('/', "-");
    let worktrees_base = root.join(&config.worktrees.dir);
    std::fs::create_dir_all(&worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);

    if git::find_worktree_for_branch(root, &branch).is_none() {
        git::add_worktree(root, &wt_path, &branch)?;
    }

    let wt_display = git::find_worktree_for_branch(root, &branch)
        .unwrap_or(wt_path);

    // Merge the default branch into the ticket branch so the agent starts from current code.
    let remote_ref = format!("origin/{default_branch}");
    let merge_ref = if std::process::Command::new("git")
        .args(["rev-parse", "--verify", &remote_ref])
        .current_dir(&wt_display)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        remote_ref.as_str()
    } else {
        default_branch.as_str()
    };
    match std::process::Command::new("git")
        .args(["merge", merge_ref, "--no-edit"])
        .current_dir(&wt_display)
        .output()
    {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.contains("Already up to date") {
                println!("Merged {merge_ref} into branch.");
            }
        }
        Ok(out) => eprintln!(
            "warning: merge {} failed: {}",
            merge_ref,
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(e) => eprintln!("warning: merge failed: {e}"),
    }

    println!("#{id}: {old_state} → in_progress (agent: {agent_name}, branch: {branch})");
    println!("Worktree: {}", wt_display.display());

    if !spawn {
        return Ok(());
    }

    // Generate worker agent name
    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    // Read apm.worker.md
    let worker_md_path = root.join("apm.worker.md");
    let worker_system = std::fs::read_to_string(&worker_md_path)
        .unwrap_or_else(|_| "You are an APM worker agent.".to_string());

    // Get ticket content
    let ticket_content = content;

    // Build log path
    let log_path = wt_display.join(".apm-worker.log");

    // Build claude command
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    cmd.args(["--system-prompt", &worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(&ticket_content);
    cmd.env("APM_AGENT_NAME", &worker_name);
    cmd.current_dir(&wt_display);

    // Redirect stdout+stderr to log file
    let log_file = std::fs::File::create(&log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);

    // Spawn detached
    let child = cmd.spawn()?;
    let pid = child.id();
    // Do not wait — drop child handle, process runs independently
    std::mem::drop(child);

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(())
}

fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() as u16
}
