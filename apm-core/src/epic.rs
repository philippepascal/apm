use anyhow::Result;
use std::path::Path;

use crate::config::StateConfig;
use crate::{git_util, worktree};

pub fn epic_is_quiescent(
    root: &Path,
    epic_id: &str,
    config: &crate::config::Config,
    worktrees: &[(std::path::PathBuf, String)],
) -> Result<Vec<String>> {
    let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)?;
    let mut blockers = Vec::new();

    for t in all_tickets.iter().filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id)) {
        let id = &t.frontmatter.id;
        let title = &t.frontmatter.title;
        let state_id = &t.frontmatter.state;

        let state_cfg = config.workflow.states.iter().find(|s| &s.id == state_id);
        let is_terminal = state_cfg.map(|s| s.terminal).unwrap_or(false);
        let is_worker_end = state_cfg.map(|s| s.worker_end).unwrap_or(false);

        let state_blocks = !is_terminal && !is_worker_end;
        if state_blocks {
            blockers.push(format!("  {id} — {title} (state: {state_id})"));
            continue;
        }

        let ticket_branch = t.frontmatter.branch.clone()
            .or_else(|| crate::ticket_fmt::branch_name_from_path(&t.path));
        if let Some(branch) = ticket_branch {
            if let Some((wt_path, _)) = worktrees.iter().find(|(_, b)| b == &branch) {
                let pid_file = wt_path.join(".apm-worker.pid");
                if pid_file.exists() {
                    if let Ok((pid, _)) = crate::worker::read_pid_file(&pid_file) {
                        if crate::worker::is_alive(pid) {
                            blockers.push(format!("  {id} — {title} (live worker)"));
                        }
                    }
                }
            }
        }
    }

    Ok(blockers)
}

/// Derive the display state of an epic from the `StateConfig`s of its tickets.
///
/// Rules (evaluated in order):
/// 1. Empty slice → "empty"
/// 2. Any state has neither `satisfies_deps` nor `terminal` → "in_progress"
/// 3. All states have `terminal = true` → "done"
/// 4. All states have `satisfies_deps = true` or `terminal = true`, but not
///    all are terminal → "implemented"
/// 5. Otherwise → "in_progress"
pub fn derive_epic_state(states: &[&StateConfig]) -> &'static str {
    if states.is_empty() {
        return "empty";
    }
    if states.iter().any(|s| !matches!(s.satisfies_deps, crate::config::SatisfiesDeps::Bool(true)) && !s.terminal) {
        return "in_progress";
    }
    if states.iter().all(|s| s.terminal) {
        return "done";
    }
    "implemented"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StateConfig;

    #[test]
    fn branch_to_title_basic() {
        assert_eq!(branch_to_title("epic/ab12cd34-user-authentication"), "User Authentication");
    }

    #[test]
    fn branch_to_title_single_word() {
        assert_eq!(branch_to_title("epic/ab12cd34-dashboard"), "Dashboard");
    }

    #[test]
    fn branch_to_title_many_words() {
        assert_eq!(branch_to_title("epic/ab12cd34-add-oauth-login-flow"), "Add Oauth Login Flow");
    }

    #[test]
    fn branch_to_title_no_slug() {
        assert_eq!(branch_to_title("epic/ab12cd34"), "Ab12cd34");
    }

    #[test]
    fn epic_id_from_branch_happy_path() {
        assert_eq!(epic_id_from_branch("epic/57bce963-refactor-apm-core"), "57bce963");
    }

    #[test]
    fn epic_id_from_branch_no_epic_prefix() {
        assert_eq!(epic_id_from_branch("57bce963-refactor"), "57bce963");
    }

    #[test]
    fn epic_id_from_branch_no_dash() {
        assert_eq!(epic_id_from_branch("nodash"), "nodash");
    }

    fn git_cmd(dir: &std::path::Path, args: &[&str]) {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .unwrap();
    }

    fn setup_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        git_cmd(p, &["init", "-q", "-b", "main"]);
        git_cmd(p, &["config", "user.email", "test@test.com"]);
        git_cmd(p, &["config", "user.name", "test"]);
        // Initial commit so commit_to_branch can use worktrees.
        std::fs::write(p.join("README.md"), "init\n").unwrap();
        git_cmd(p, &["add", "README.md"]);
        git_cmd(p, &["commit", "-m", "init"]);
        tmp
    }

    const TOML_WITH_STATES: &str = concat!(
        "[project]\nname = \"test\"\n\n",
        "[tickets]\ndir = \"tickets\"\n\n",
        "[[workflow.states]]\nid = \"ready\"\nlabel = \"Ready\"\nterminal = false\n\n",
        "[[workflow.states]]\nid = \"closed\"\nlabel = \"Closed\"\nterminal = true\n",
    );

    fn make_ticket_content(id: &str, state: &str, epic: &str) -> String {
        format!(
            "+++\nid = \"{id}\"\ntitle = \"Ticket {id}\"\nstate = \"{state}\"\nepic = \"{epic}\"\n+++\n\nBody.\n"
        )
    }

    #[test]
    fn set_epic_owner_updates_non_terminal_skips_terminal() {
        let tmp = setup_repo();
        let p = tmp.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/config.toml"), TOML_WITH_STATES).unwrap();
        std::fs::write(p.join(".apm/local.toml"), "username = \"alice\"\n").unwrap();

        let config = crate::config::Config::load(p).unwrap();

        // Non-terminal ticket in this epic.
        let content_a = make_ticket_content("aaaa1234", "ready", "epic1234");
        crate::git::commit_to_branch(p, "ticket/aaaa1234-t1", "tickets/aaaa1234-t1.md", &content_a, "add t1").unwrap();

        // Terminal ticket in this epic — should be skipped.
        let content_b = make_ticket_content("bbbb5678", "closed", "epic1234");
        crate::git::commit_to_branch(p, "ticket/bbbb5678-t2", "tickets/bbbb5678-t2.md", &content_b, "add t2").unwrap();

        // Ticket in a different epic — should be ignored.
        let content_c = make_ticket_content("cccc9012", "ready", "other123");
        crate::git::commit_to_branch(p, "ticket/cccc9012-t3", "tickets/cccc9012-t3.md", &content_c, "add t3").unwrap();

        let (changed, skipped) = set_epic_owner(p, "epic1234", "alice", &config).unwrap();
        assert_eq!(changed, 1, "one non-terminal ticket should be changed");
        assert_eq!(skipped, 1, "one terminal ticket should be skipped");
    }

    #[test]
    fn set_epic_owner_all_terminal_returns_zero_changed() {
        let tmp = setup_repo();
        let p = tmp.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/config.toml"), TOML_WITH_STATES).unwrap();

        let config = crate::config::Config::load(p).unwrap();

        let content_a = make_ticket_content("dddd1111", "closed", "epic5678");
        crate::git::commit_to_branch(p, "ticket/dddd1111-t4", "tickets/dddd1111-t4.md", &content_a, "add t4").unwrap();
        let content_b = make_ticket_content("eeee2222", "closed", "epic5678");
        crate::git::commit_to_branch(p, "ticket/eeee2222-t5", "tickets/eeee2222-t5.md", &content_b, "add t5").unwrap();

        let (changed, skipped) = set_epic_owner(p, "epic5678", "bob", &config).unwrap();
        assert_eq!(changed, 0);
        assert_eq!(skipped, 2);
    }

    const TOML_WITH_WORKER_END: &str = concat!(
        "[project]\nname = \"test\"\n\n",
        "[tickets]\ndir = \"tickets\"\n\n",
        "[[workflow.states]]\nid = \"ready\"\nlabel = \"Ready\"\nterminal = false\nworker_end = false\n\n",
        "[[workflow.states]]\nid = \"implemented\"\nlabel = \"Implemented\"\nterminal = false\nworker_end = true\n\n",
        "[[workflow.states]]\nid = \"closed\"\nlabel = \"Closed\"\nterminal = true\n",
    );

    #[test]
    fn epic_is_quiescent_all_done() {
        let tmp = setup_repo();
        let p = tmp.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/config.toml"), TOML_WITH_WORKER_END).unwrap();
        std::fs::write(p.join(".apm/local.toml"), "username = \"alice\"\n").unwrap();

        let config = crate::config::Config::load(p).unwrap();

        let closed = make_ticket_content("aaaa0001", "closed", "epic0001");
        crate::git::commit_to_branch(p, "ticket/aaaa0001-t1", "tickets/aaaa0001-t1.md", &closed, "add t1").unwrap();

        let implemented = make_ticket_content("bbbb0002", "implemented", "epic0001");
        crate::git::commit_to_branch(p, "ticket/bbbb0002-t2", "tickets/bbbb0002-t2.md", &implemented, "add t2").unwrap();

        let blockers = epic_is_quiescent(p, "epic0001", &config, &[]).unwrap();
        assert!(blockers.is_empty(), "expected no blockers, got: {blockers:?}");
    }

    #[test]
    fn epic_is_quiescent_state_blocker() {
        let tmp = setup_repo();
        let p = tmp.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/config.toml"), TOML_WITH_WORKER_END).unwrap();
        std::fs::write(p.join(".apm/local.toml"), "username = \"alice\"\n").unwrap();

        let config = crate::config::Config::load(p).unwrap();

        let content = make_ticket_content("cccc0003", "ready", "epic0002");
        crate::git::commit_to_branch(p, "ticket/cccc0003-t3", "tickets/cccc0003-t3.md", &content, "add t3").unwrap();

        let blockers = epic_is_quiescent(p, "epic0002", &config, &[]).unwrap();
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].contains("cccc0003"));
        assert!(blockers[0].contains("(state: ready)"));
    }

    #[test]
    fn epic_is_quiescent_live_worker_blocker() {
        let tmp = setup_repo();
        let p = tmp.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/config.toml"), TOML_WITH_WORKER_END).unwrap();
        std::fs::write(p.join(".apm/local.toml"), "username = \"alice\"\n").unwrap();

        let config = crate::config::Config::load(p).unwrap();

        // Ticket in worker_end state (quiescent by state check).
        let content = make_ticket_content("dddd0004", "implemented", "epic0003");
        crate::git::commit_to_branch(p, "ticket/dddd0004-t4", "tickets/dddd0004-t4.md", &content, "add t4").unwrap();

        // Simulate a worktree with a live .apm-worker.pid (current process PID).
        let wt_path = tmp.path().join("fake-worktree-dddd0004");
        std::fs::create_dir_all(&wt_path).unwrap();
        let pid = std::process::id();
        std::fs::write(
            wt_path.join(".apm-worker.pid"),
            format!(r#"{{"pid":{pid},"ticket_id":"dddd0004","started_at":"2026-01-01T00:00:00Z"}}"#),
        ).unwrap();

        let worktrees = vec![(wt_path, "ticket/dddd0004-t4".to_string())];
        let blockers = epic_is_quiescent(p, "epic0003", &config, &worktrees).unwrap();
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].contains("dddd0004"));
        assert!(blockers[0].contains("(live worker)"));
    }

    fn make_state(terminal: bool, satisfies_deps: bool, actionable: Vec<&str>) -> StateConfig {
        StateConfig {
            id: "x".to_string(),
            label: "x".to_string(),
            description: String::new(),
            terminal,
            worker_end: false,
            satisfies_deps: crate::config::SatisfiesDeps::Bool(satisfies_deps),
            dep_requires: None,
            transitions: vec![],
            actionable: actionable.into_iter().map(|s| s.to_string()).collect(),
            instructions: None,
        }
    }

    #[test]
    fn empty_slice_is_empty() {
        assert_eq!(derive_epic_state(&[]), "empty");
    }

    #[test]
    fn all_terminal_is_done() {
        let a = make_state(true, false, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "done");
    }

    #[test]
    fn all_satisfies_deps_not_all_terminal_is_implemented() {
        let a = make_state(false, true, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "implemented");
    }

    #[test]
    fn any_neither_satisfies_nor_terminal_is_in_progress() {
        let a = make_state(false, false, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "in_progress");
    }

    #[test]
    fn mixed_non_terminal_non_satisfies_is_in_progress() {
        let a = make_state(false, false, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "in_progress");
    }
}

pub fn create(root: &Path, title: &str, config: &crate::config::Config) -> Result<String> {
    let id = crate::ticket_fmt::gen_hex_id();
    let slug = crate::ticket::slugify(title);
    let branch = format!("epic/{id}-{slug}");

    let default_branch = &config.project.default_branch;
    git_util::fetch_branch(root, default_branch)?;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let wt_path = std::env::temp_dir().join(format!(
        "apm-{}-{}-{}",
        std::process::id(),
        unique,
        branch.replace('/', "-"),
    ));

    let wt_path_str = wt_path.to_string_lossy();
    git_util::run(root, &["worktree", "add", "-b", &branch, &wt_path_str, &format!("origin/{default_branch}")])?;

    let result = (|| -> Result<()> {
        let epic_md = wt_path.join("EPIC.md");
        std::fs::write(&epic_md, format!("# {title}\n"))?;

        git_util::stage_files(&wt_path, &["EPIC.md"])?;

        let commit_msg = format!("epic({id}): create {title}");
        git_util::commit(&wt_path, &commit_msg)?;
        Ok(())
    })();

    let _ = worktree::remove_worktree(root, &wt_path, true);
    let _ = std::fs::remove_dir_all(&wt_path);

    result?;

    crate::git::push_branch_tracking(root, &branch)?;

    Ok(branch)
}

pub fn find_epic_branch(root: &Path, short_id: &str) -> Option<String> {
    let pattern = format!("epic/{short_id}-*");
    let local = crate::git_util::run(root, &["branch", "--list", &pattern]).ok()?;
    for b in local.lines().map(|l| l.trim().trim_start_matches(['*', '+']).trim()) {
        if !b.is_empty() {
            return Some(b.to_string());
        }
    }
    let remote_pattern = format!("origin/epic/{short_id}-*");
    let remote = crate::git_util::run(root, &["branch", "-r", "--list", &remote_pattern]).ok()?;
    for b in remote.lines().map(|l| l.trim()) {
        if !b.is_empty() {
            return Some(b.trim_start_matches("origin/").to_string());
        }
    }
    None
}

pub fn find_epic_branches(root: &Path, id_prefix: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    let local = crate::git_util::run(root, &["branch", "--list", "epic/*"]).unwrap_or_default();
    for b in local.lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim())
        .filter(|l| !l.is_empty())
    {
        let id_part = b.trim_start_matches("epic/").split('-').next().unwrap_or("");
        if id_part.starts_with(id_prefix) && seen.insert(b.to_string()) {
            result.push(b.to_string());
        }
    }

    let remote = crate::git_util::run(root, &["branch", "-r", "--list", "origin/epic/*"]).unwrap_or_default();
    for b in remote.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        let short = b.trim_start_matches("origin/");
        let id_part = short.trim_start_matches("epic/").split('-').next().unwrap_or("");
        if id_part.starts_with(id_prefix) && seen.insert(short.to_string()) {
            result.push(short.to_string());
        }
    }

    result
}

pub fn epic_branches(root: &Path) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let mut branches = Vec::new();

    let local = crate::git_util::run(root, &["branch", "--list", "epic/*"]).unwrap_or_default();
    for b in local.lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.to_string()) {
            branches.push(b.to_string());
        }
    }

    let remote = crate::git_util::run(root, &["branch", "-r", "--list", "origin/epic/*"]).unwrap_or_default();
    for b in remote.lines()
        .map(|l| l.trim().trim_start_matches("origin/").to_string())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.clone()) {
            branches.push(b);
        }
    }

    branches.sort();
    Ok(branches)
}

pub fn branch_to_title(branch: &str) -> String {
    let rest = branch.trim_start_matches("epic/");
    let slug = match rest.find('-') {
        Some(pos) => &rest[pos + 1..],
        None => rest,
    };
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn epic_id_from_branch(branch: &str) -> &str {
    let rest = branch.trim_start_matches("epic/");
    match rest.find('-') {
        Some(pos) => &rest[..pos],
        None => rest,
    }
}

pub fn set_epic_owner(
    root: &Path,
    epic_id: &str,
    new_owner: &str,
    config: &crate::config::Config,
) -> Result<(usize, usize)> {
    let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)?;
    let terminal = config.terminal_state_ids();

    let (mut to_change, skipped): (Vec<_>, Vec<_>) = all_tickets
        .into_iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .partition(|t| !terminal.contains(&t.frontmatter.state));

    for t in &to_change {
        crate::ticket::check_owner(root, t)?;
    }

    for t in &mut to_change {
        crate::ticket::set_field(&mut t.frontmatter, "owner", new_owner)?;
        let content = t.serialize()?;
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );
        let ticket_branch = t.frontmatter.branch.clone()
            .or_else(|| crate::ticket_fmt::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{}", t.frontmatter.id));
        crate::git::commit_to_branch(
            root,
            &ticket_branch,
            &rel_path,
            &content,
            &format!("ticket({}): bulk set owner = {}", t.frontmatter.id, new_owner),
        )?;
    }

    Ok((to_change.len(), skipped.len()))
}

pub fn create_epic_branch(root: &Path, title: &str, config: &crate::config::Config) -> Result<(String, String)> {
    let id = crate::ticket_fmt::gen_hex_id();
    let slug = crate::ticket::slugify(title);
    let branch = format!("epic/{id}-{slug}");
    let default_branch = &config.project.default_branch;
    let _ = crate::git_util::run(root, &["fetch", "origin", default_branch]);
    if crate::git_util::run(root, &["branch", &branch, &format!("origin/{default_branch}")]).is_err() {
        crate::git_util::run(root, &["branch", &branch, default_branch])?;
    }
    crate::git_util::commit_to_branch(root, &branch, "EPIC.md", &format!("# {title}\n"), "epic: init")?;
    let _ = crate::git_util::push_branch(root, &branch);
    Ok((id, branch))
}
