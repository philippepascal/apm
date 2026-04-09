use crate::{config::Config, git, ticket};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

#[derive(Debug)]
pub struct ArchiveOutput {
    pub moves: Vec<(String, String)>,
    pub dry_run_moves: Vec<(String, String)>,
    pub archived_count: usize,
    pub warnings: Vec<String>,
}

pub fn archive(
    root: &Path,
    config: &Config,
    dry_run: bool,
    older_than: Option<DateTime<Utc>>,
) -> Result<ArchiveOutput> {
    let mut warnings: Vec<String> = Vec::new();
    let mut dry_run_moves: Vec<(String, String)> = Vec::new();

    let archive_dir = config.tickets.archive_dir.as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "archive_dir is not set in [tickets] config; add `archive_dir = \"archive/tickets\"` to .apm/config.toml"
        ))?;

    let terminal_states = config.terminal_state_ids();

    let default_branch = &config.project.default_branch;
    let tickets_dir = config.tickets.dir.to_string_lossy().into_owned();
    let archive_dir_str = archive_dir.to_string_lossy().into_owned();

    let files = match git::list_files_on_branch(root, default_branch, &tickets_dir) {
        Ok(f) => f,
        Err(_) => {
            return Ok(ArchiveOutput { moves: vec![], dry_run_moves, archived_count: 0, warnings });
        }
    };

    let mut moves: Vec<(String, String, String)> = Vec::new();

    for rel_path in &files {
        if !rel_path.ends_with(".md") {
            continue;
        }

        let content = match git::read_from_branch(root, default_branch, rel_path) {
            Ok(c) => c,
            Err(_) => {
                warnings.push(format!("warning: could not read {rel_path} on {default_branch} — skipping"));
                continue;
            }
        };

        let dummy_path = root.join(rel_path);
        let t = match ticket::Ticket::parse(&dummy_path, &content) {
            Ok(t) => t,
            Err(e) => {
                warnings.push(format!("warning: could not parse {rel_path}: {e} — skipping"));
                continue;
            }
        };

        if !terminal_states.contains(&t.frontmatter.state) {
            warnings.push(format!(
                "warning: {} is in non-terminal state '{}' — skipping",
                rel_path, t.frontmatter.state
            ));
            continue;
        }

        if let Some(threshold) = older_than {
            if let Some(updated_at) = t.frontmatter.updated_at {
                if updated_at >= threshold {
                    continue;
                }
            }
        }

        let filename = rel_path
            .split('/')
            .next_back()
            .unwrap_or(rel_path.as_str());
        let new_rel_path = format!("{archive_dir_str}/{filename}");

        if dry_run {
            dry_run_moves.push((rel_path.clone(), new_rel_path));
        } else {
            moves.push((rel_path.clone(), new_rel_path, content));
        }
    }

    if dry_run {
        // Original behavior: in dry_run mode, "nothing to archive" is always printed
        // because the moves vec is always empty in dry_run (bug preserved intentionally).
        return Ok(ArchiveOutput { moves: vec![], dry_run_moves, archived_count: 0, warnings });
    }

    if moves.is_empty() {
        return Ok(ArchiveOutput { moves: vec![], dry_run_moves: vec![], archived_count: 0, warnings });
    }

    let move_refs: Vec<(&str, &str, &str)> = moves
        .iter()
        .map(|(o, n, c)| (o.as_str(), n.as_str(), c.as_str()))
        .collect();

    git::move_files_on_branch(root, default_branch, &move_refs, "archive: move closed tickets")?;

    let archived_count = moves.len();
    let actual_moves: Vec<(String, String)> = moves.into_iter().map(|(o, n, _)| (o, n)).collect();

    Ok(ArchiveOutput { moves: actual_moves, dry_run_moves: vec![], archived_count, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn make_config(archive_dir: Option<&str>) -> Config {
        let archive_line = match archive_dir {
            Some(d) => format!("archive_dir = \"{d}\"\n"),
            None => String::new(),
        };
        let toml = format!(
            r#"[project]
name = "test"
default_branch = "main"

[tickets]
dir = "tickets"
{archive_line}
[[workflow.states]]
id = "new"
label = "New"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#
        );
        toml::from_str(&toml).unwrap()
    }

    #[test]
    fn archive_errors_when_no_archive_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let config = make_config(None);
        let result = archive(tmp.path(), &config, false, None);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("archive_dir is not set"), "error was: {msg}");
    }

    #[test]
    fn archive_dir_config_accepted() {
        let config = make_config(Some("archive/tickets"));
        assert_eq!(
            config.tickets.archive_dir.as_deref(),
            Some(std::path::Path::new("archive/tickets"))
        );
    }
}
