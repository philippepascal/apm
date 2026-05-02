use anyhow::Result;
use apm_core::config::Config;
use apm_core::wrapper::WrapperKind;
use std::path::Path;

pub fn run_list(root: &Path) -> Result<()> {
    let config = Config::load(root)?;
    let entries = apm_core::agents::list_wrappers(root, &config)?;

    let name_w = entries.iter().map(|e| e.name.len()).max().unwrap_or(4).max(4);
    let kind_w = "built-in".len();
    let parser_w = "canonical".len();

    println!(
        "{:<name_w$}  {:<kind_w$}  {:<parser_w$}  STATUS",
        "NAME", "KIND", "PARSER"
    );
    for entry in &entries {
        let kind_str = match &entry.kind {
            WrapperKind::Builtin(_) => "built-in",
            WrapperKind::Custom { .. } => "project",
        };
        let status = entry.configured_as.join(", ");
        println!(
            "{:<name_w$}  {:<kind_w$}  {:<parser_w$}  {}",
            entry.name, kind_str, entry.parser, status
        );
    }
    Ok(())
}

pub fn run_new(root: &Path, name: &str, force: bool) -> Result<()> {
    apm_core::agents::scaffold_wrapper(root, name, force)?;

    let dir = root.join(".apm").join("agents").join(name);
    println!("Created:");
    println!("  {}", dir.join("wrapper.sh").display());
    println!("  {}", dir.join("manifest.toml").display());
    println!("  {}", dir.join("apm.worker.md").display());
    println!("  {}", dir.join("apm.spec-writer.md").display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/wrapper.sh to invoke your AI tool", dir.display());
    println!("  2. Run: apm agents test {name}");
    Ok(())
}

pub fn run_test(root: &Path, name: &str) -> Result<()> {
    let report = apm_core::agents::test_wrapper(root, name)?;

    let label = if report.passed { "PASS" } else { "FAIL" };
    println!(
        "{}  exit={}  events={}  non-canonical={}  stderr={}  wall={}ms",
        label,
        report.exit_code,
        report.canonical_events,
        report.non_canonical_lines,
        report.stderr_lines,
        report.wall_millis,
    );

    if !report.passed {
        anyhow::bail!("wrapper test failed");
    }
    Ok(())
}

pub fn run_eject(root: &Path, name: &str) -> Result<()> {
    apm_core::agents::eject_wrapper(root, name)?;

    let script = root.join(".apm").join("agents").join(name).join("wrapper.sh");
    println!("Ejected to: {}", script.display());
    println!("Run: apm agents test {name}");
    Ok(())
}
