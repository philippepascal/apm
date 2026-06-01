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
    assert!(result.unwrap(), "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    let workers = parsed["workers"].as_table().unwrap();
    assert_eq!(workers.get("default").and_then(|v| v.as_str()), Some("claude/coder"), "default should be set");
    assert!(workers.get("command").is_none(), "command should be removed");
    assert!(workers.get("args").is_none(), "args should be removed");
    assert!(workers.get("agent").is_none(), "agent should be removed");
    assert_eq!(workers.get("model").and_then(|v| v.as_str()), Some("sonnet"), "model should be at top level");
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
    assert!(!result.unwrap(), "expected no migration for non-claude command");

    let after = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    assert_eq!(original, after, "file must be unchanged when command is not claude");
}

#[test]
fn test_fix_removes_worker_profiles() {
    // [worker_profiles] is removed in V3 regardless of command — user gets a warning
    // and must add worker_profile = "<agent>/<role>" to workflow transitions manually.
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

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert!(result.unwrap(), "expected migration to remove worker_profiles");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();
    assert!(parsed.get("worker_profiles").is_none(), "worker_profiles must be removed");
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
    assert!(result.unwrap(), "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    let workers = parsed["workers"].as_table().unwrap();
    assert_eq!(workers.get("default").and_then(|v| v.as_str()), Some("claude/coder"), "default must be set");
    assert!(workers.get("agent").is_none(), "agent must be removed");
    assert_eq!(workers.get("model").and_then(|v| v.as_str()), Some("opus"), "model must be at top level");
}

#[test]
fn test_fix_already_migrated_noop() {
    // Fully migrated config — V3 format, no legacy fields
    let config = r#"[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
default = "claude/coder"
model = "sonnet"
"#;
    let dir = setup(config);
    let original = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();

    let result = apm::cmd::validate::apply_config_migration_fixes(dir.path());
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert!(!result.unwrap(), "expected no-op on already-migrated config");

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
    assert!(result.unwrap(), "expected migration to occur");

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
    assert!(result.unwrap(), "expected migration to occur");

    let written = fs::read_to_string(dir.path().join(".apm/config.toml")).unwrap();
    let parsed: toml::Value = toml::from_str(&written).unwrap();

    assert!(parsed.get("worker_profiles").is_none(), "worker_profiles must be removed entirely");
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
    assert!(result.unwrap(), "migration should have occurred");

    // Double-check by calling validate_config directly on the migrated config.
    let migrated = apm_core::config::Config::load(dir.path()).unwrap();
    let errors = apm_core::validate::validate_config(&migrated, dir.path());
    assert!(
        errors.is_empty(),
        "validate_config must return no errors after migration; got: {errors:?}"
    );
}
