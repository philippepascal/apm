use std::fs;
use tempfile::TempDir;

fn setup(config_toml: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let apm_dir = dir.path().join(".apm");
    fs::create_dir_all(&apm_dir).unwrap();
    fs::write(apm_dir.join("config.toml"), config_toml).unwrap();
    dir
}

/// Minimal valid workflow appended to fixture configs that need to pass re-validation.
const MINIMAL_WORKFLOW: &str = r#"
[[workflow.states]]
id = "new"
label = "New"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#;

#[test]
fn test_fix_migrates_claude_command() {
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
command = "claude"
args = ["--print", "--output-format", "stream-json"]
model = "sonnet"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), true, "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    let workers = parsed["workers"].as_table().unwrap();
    assert_eq!(workers.get("agent").and_then(|v| v.as_str()), Some("claude"), "agent should be set");
    assert!(workers.get("command").is_none(), "command should be removed");
    assert!(workers.get("args").is_none(), "args should be removed");
    assert!(workers.get("model").is_none(), "model should be removed from [workers]");

    let options = workers.get("options")
        .or_else(|| parsed.get("workers").and_then(|w| w.as_table()).and_then(|t| t.get("options")));
    // options may be a subtable in the parsed value
    let options_model = parsed["workers"]["options"]["model"].as_str();
    assert_eq!(options_model, Some("sonnet"), "model should be in options");
    let _ = options; // suppress unused warning
}

#[test]
fn test_fix_noop_on_non_claude_command() {
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
command = "my-ai"
model = "opus"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);
    let original = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), false, "expected no migration for non-claude command");

    let after = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    assert_eq!(original, after, "file must be unchanged when command is not claude");
}

#[test]
fn test_fix_noop_on_non_claude_profile_command() {
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worker_profiles.impl_agent]
command = "my-ai"
model = "opus"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);
    let original = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), false, "expected no migration for non-claude profile command");

    let after = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    assert_eq!(original, after, "file must be unchanged when profile command is not claude");
}

#[test]
fn test_fix_mixed_legacy_and_new_fields() {
    // agent already present but leftover model in [workers]
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
agent = "claude"
model = "opus"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), true, "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    let workers = parsed["workers"].as_table().unwrap();
    assert_eq!(workers.get("agent").and_then(|v| v.as_str()), Some("claude"), "agent must be preserved");
    assert!(workers.get("model").is_none(), "legacy model must be removed");
    assert_eq!(parsed["workers"]["options"]["model"].as_str(), Some("opus"), "model must be in options");
}

#[test]
fn test_fix_already_migrated_noop() {
    // Fully migrated config — no legacy fields
    let config = r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
agent = "claude"

[workers.options]
model = "sonnet"
"#;
    let dir = setup(config);
    let original = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), false, "expected no-op on already-migrated config");

    let after = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    assert_eq!(original, after, "file must be byte-identical for already-migrated config");
}

#[test]
fn test_fix_preserves_comments() {
    let config = format!(
        r#"# Top-level project comment
[project]
name = "test"

[tickets]
dir = "tickets"

# Worker section comment
[workers]
command = "claude"
model = "sonnet"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), true, "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    assert!(
        written.contains("# Top-level project comment"),
        "top-level comment must survive"
    );
    assert!(
        written.contains("# Worker section comment"),
        "worker section comment must survive"
    );
}

#[test]
fn test_fix_profile_model_migration() {
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worker_profiles.spec_agent]
model = "opus"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap(), true, "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    let profile = parsed["worker_profiles"]["spec_agent"].as_table().unwrap();
    assert!(profile.get("model").is_none(), "profile model key must be removed");

    let options_model = parsed["worker_profiles"]["spec_agent"]["options"]["model"].as_str();
    assert_eq!(options_model, Some("opus"), "model must appear in profile options");
}

#[test]
fn test_fix_revalidate_passes() {
    // After migration, validate_config must return no errors.
    let config = format!(
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
command = "claude"
model = "sonnet"
{MINIMAL_WORKFLOW}"#
    );
    let dir = setup(&config);

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    // apply_config_migration_fixes internally calls validate_config and bails if it fails.
    // A successful Ok(true) here means re-validation passed.
    assert!(result.is_ok(), "apply_config_migration_fixes should not error: {result:?}");
    assert_eq!(result.unwrap(), true, "migration should have occurred");

    // Double-check by calling validate_config directly on the migrated config.
    let migrated = apm_core::config::Config::load(dir.path()).unwrap();
    let errors = apm_core::validate::validate_config(&migrated, dir.path());
    assert!(
        errors.is_empty(),
        "validate_config must return no errors after migration; got: {errors:?}"
    );
}
