use anyhow::Result;
use std::path::Path;

pub enum HashTripOutcome {
    /// Stamp matched; no action taken.
    Clean,
    /// Hash changed, validate clean, stamp written.
    PassedAndRefreshed,
    /// Hash changed, validate failed; (subject, message) pairs.
    Failed(Vec<(String, String)>),
}

pub fn is_exempt_command(cmd: &super::Command) -> bool {
    matches!(
        cmd,
        super::Command::Validate { .. }
            | super::Command::Init { .. }
            | super::Command::Help { .. }
    )
}

pub fn is_read_only_command(cmd: &super::Command) -> bool {
    matches!(
        cmd,
        super::Command::List { .. }
            | super::Command::Show { .. }
            | super::Command::Next { .. }
            | super::Command::Verify { .. }
    )
}

pub fn run(root: &Path) -> Result<HashTripOutcome> {
    if !root.join(".apm").join("config.toml").exists() {
        return Ok(HashTripOutcome::Clean);
    }

    let live = apm_core::hash_stamp::config_hash(root)?;
    let stored = apm_core::hash_stamp::read_stamp(root);

    if stored.as_deref() == Some(live.as_str()) {
        return Ok(HashTripOutcome::Clean);
    }

    let config = apm_core::config::Config::load(root)?;
    let tickets =
        apm_core::ticket::load_all_from_git(root, &config.tickets.dir).unwrap_or_default();

    let mut issues: Vec<(String, String)> = Vec::new();

    for err in apm_core::validate::validate_config(&config, root) {
        issues.push(("config".into(), err));
    }

    for (subject, msg) in apm_core::validate::validate_depends_on(&config, &tickets) {
        issues.push((subject, msg));
    }

    if issues.is_empty() {
        apm_core::hash_stamp::write_stamp(root, &live)?;
        Ok(HashTripOutcome::PassedAndRefreshed)
    } else {
        Ok(HashTripOutcome::Failed(issues))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_is_exempt() {
        let cmd = super::super::Command::Validate {
            fix: false,
            json: false,
            config_only: false,
            no_aggressive: false,
        };
        assert!(is_exempt_command(&cmd));
    }

    #[test]
    fn init_is_exempt() {
        let cmd = super::super::Command::Init {
            no_claude: false,
            migrate: false,
            with_docker: false,
            quiet: false,
        };
        assert!(is_exempt_command(&cmd));
    }

    #[test]
    fn list_is_read_only() {
        let cmd = super::super::Command::List {
            state: None,
            unassigned: false,
            all: false,
            actionable: None,
            no_aggressive: false,
            mine: false,
            author: None,
            owner: None,
        };
        assert!(is_read_only_command(&cmd));
    }

    #[test]
    fn new_is_not_read_only() {
        let cmd = super::super::Command::New {
            title: "t".into(),
            no_edit: false,
            side_note: false,
            context: None,
            context_section: None,
            no_aggressive: false,
            section: vec![],
            set: vec![],
            epic: None,
            depends_on: vec![],
        };
        assert!(!is_read_only_command(&cmd));
    }

    #[test]
    fn state_is_not_read_only() {
        let cmd = super::super::Command::State {
            id: "abcd1234".into(),
            state: "closed".into(),
            no_aggressive: false,
            force: false,
        };
        assert!(!is_read_only_command(&cmd));
    }

    #[test]
    fn verify_is_read_only() {
        let cmd = super::super::Command::Verify {
            fix: false,
            no_aggressive: false,
        };
        assert!(is_read_only_command(&cmd));
    }
}
