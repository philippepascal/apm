use anyhow::Result;
use std::path::Path;

pub fn run(cli_cmd: clap::Command, root: &Path, role: Option<&str>, ticket_id: Option<&str>) -> Result<()> {
    let commands = extract_commands(&cli_cmd);
    let current_state = ticket_id.and_then(|id| resolve_ticket_state(root, id));
    let text = apm_core::instructions::generate(root, role, ticket_id, &commands, current_state.as_deref())?;
    print!("{}", text);
    Ok(())
}

fn resolve_ticket_state(root: &Path, ticket_id: &str) -> Option<String> {
    use apm_core::{config::Config, git, ticket_fmt, ticket};
    let config = Config::load(root).ok()?;
    let branches = git::ticket_branches(root).ok()?;
    let branch = ticket_fmt::resolve_ticket_branch(&branches, ticket_id).ok()?;
    let suffix = branch.trim_start_matches("ticket/");
    let filename = format!("{suffix}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let dummy_path = root.join(&rel_path);
    let content = git::read_from_branch(root, &branch, &rel_path).ok()?;
    let t = ticket::Ticket::parse(&dummy_path, &content).ok()?;
    Some(t.frontmatter.state.clone())
}

fn extract_commands(cli_cmd: &clap::Command) -> Vec<(String, String)> {
    let mut cmds: Vec<&clap::Command> = cli_cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .collect();
    cmds.sort_by_key(|c| c.get_name());
    cmds.iter()
        .map(|c| {
            let name = c.get_name().to_string();
            let about = c.get_about().map(|a| a.to_string()).unwrap_or_default();
            (name, about)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cmd() -> clap::Command {
        clap::Command::new("testapp")
            .subcommand(
                clap::Command::new("show")
                    .about("Show a ticket"),
            )
            .subcommand(
                clap::Command::new("start")
                    .about("Claim a ticket"),
            )
            .subcommand(
                clap::Command::new("state")
                    .about("Transition state"),
            )
            .subcommand(
                clap::Command::new("spec")
                    .about("Read or write spec sections"),
            )
            .subcommand(
                clap::Command::new("new")
                    .about("Create a new ticket"),
            )
            .subcommand(
                clap::Command::new("sync")
                    .about("Sync with remote"),
            )
            .subcommand(
                clap::Command::new("list")
                    .about("List tickets"),
            )
            .subcommand(
                clap::Command::new("next")
                    .about("Return next actionable ticket"),
            )
            .subcommand(
                clap::Command::new("set")
                    .about("Set a field"),
            )
            .subcommand(clap::Command::new("_hook").about("Hidden").hide(true))
    }

    #[test]
    fn run_no_role_returns_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let result = run(make_test_cmd(), tmp.path(), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn run_with_role_returns_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let result = run(make_test_cmd(), tmp.path(), Some("worker"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn extract_commands_excludes_hidden() {
        let commands = extract_commands(&make_test_cmd());
        let names: Vec<&str> = commands.iter().map(|(n, _)| n.as_str()).collect();
        assert!(!names.contains(&"_hook"), "hidden command should be excluded");
        assert!(names.contains(&"show"), "show should be included");
    }

    #[test]
    fn extract_commands_sorted() {
        let commands = extract_commands(&make_test_cmd());
        let names: Vec<&str> = commands.iter().map(|(n, _)| n.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "commands should be sorted alphabetically");
    }

    #[test]
    fn generate_no_ansi_via_run() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = extract_commands(&make_test_cmd());
        let out = apm_core::instructions::generate(tmp.path(), None, None, &commands, None).unwrap();
        assert!(!out.contains('\x1b'), "ANSI escape code found in output");
    }

    #[test]
    fn generate_contains_all_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = extract_commands(&make_test_cmd());
        // No-role now returns a role index, not the full sections.
        let out = apm_core::instructions::generate(tmp.path(), None, None, &commands, None).unwrap();
        assert!(out.contains("coder"), "coder missing from role index");
        assert!(out.contains("spec-writer"), "spec-writer missing from role index");
        assert!(!out.contains("## State Machine"), "State Machine should be absent with no role");
    }

    #[test]
    fn worker_role_includes_show_and_set() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = extract_commands(&make_test_cmd());
        let out = apm_core::instructions::generate(tmp.path(), Some("worker"), None, &commands, None).unwrap();
        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];
        assert!(cr_section.contains("apm show"), "apm show not found in worker command reference");
        assert!(cr_section.contains("apm set"), "apm set not found in worker command reference");
    }

    #[test]
    fn worker_role_excludes_start() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = extract_commands(&make_test_cmd());
        let out = apm_core::instructions::generate(tmp.path(), Some("worker"), None, &commands, None).unwrap();
        let cr_pos = out.find("## Command Reference").unwrap();
        assert!(
            !out[cr_pos..].contains("apm start"),
            "apm start found in worker command reference but should be excluded"
        );
    }
}
