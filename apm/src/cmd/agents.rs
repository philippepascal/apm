use anyhow::Result;
use apm_core::config::Config;
use apm_core::wrapper::WrapperKind;
use std::path::Path;
use apm_core::start::AgentDiagnostic;

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

pub fn run_resolve(root: &Path, ticket_id: &str, json: bool) -> Result<()> {
    let diag = apm_core::start::resolve_for_diagnostic(root, ticket_id)?;
    if json {
        print_resolve_json(&diag)
    } else {
        print_resolve_human(&diag)
    }
}

fn print_resolve_human(d: &AgentDiagnostic) -> Result<()> {
    println!("Agent assignment for {} (state: {}):", d.ticket_id, d.ticket_state);
    if !d.dispatchable {
        println!("  note: state {:?} has no worker dispatch; showing resolution for {:?}", d.ticket_state, d.transition_label);
    }
    if d.transition_label == "none" {
        println!("  (no command:start transition defined in this workflow)");
        return Ok(());
    }

    const W: usize = 14;
    let model_val = d.model.as_deref().unwrap_or("—");
    let container_val = d.container.as_deref().unwrap_or("—");
    let manifest_status = if d.manifest_present { "[present]" } else { "[absent]" };

    println!("  {:<W$} {}  ({})", "agent", d.agent, d.agent_source);
    println!("  {:<W$} {}  ({})", "role", d.role, d.role_source);
    println!("  {:<W$} {}  ({})", "model", model_val, d.model_source);
    println!("  {:<W$} {}  ({})", "container", container_val, d.container_source);
    println!("  {:<W$} {}  {}", "manifest", d.manifest_path, manifest_status);

    if d.env.is_empty() {
        println!("  {:<W$} (none)", "env");
    } else {
        println!("  {:<W$}", "env");
        for (k, v, src) in &d.env {
            println!("    {k}={v}  ({src})");
        }
    }

    if d.keychain.is_empty() {
        println!("  {:<W$} (none)", "keychain");
    } else {
        println!("  {:<W$}", "keychain");
        let mut entries: Vec<_> = d.keychain.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (k, v) in entries {
            println!("    {k} → {v}");
        }
    }
    Ok(())
}

fn print_resolve_json(d: &AgentDiagnostic) -> Result<()> {
    let env_arr: Vec<serde_json::Value> = d.env.iter()
        .map(|(k, v, src)| serde_json::json!({"key": k, "value": v, "source": src}))
        .collect();
    let keychain_obj: serde_json::Map<String, serde_json::Value> = d.keychain.iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let obj = serde_json::json!({
        "ticket_id": d.ticket_id,
        "ticket_state": d.ticket_state,
        "dispatchable": d.dispatchable,
        "resolved_from_state": d.resolved_from_state,
        "transition_label": d.transition_label,
        "worker_profile_str": d.worker_profile_str,
        "profile_source": d.profile_source,
        "agent": d.agent,
        "agent_source": d.agent_source,
        "role": d.role,
        "role_source": d.role_source,
        "model": d.model,
        "model_source": d.model_source,
        "container": d.container,
        "container_source": d.container_source,
        "manifest_path": d.manifest_path,
        "manifest_present": d.manifest_present,
        "env": env_arr,
        "keychain": keychain_obj,
    });
    println!("{}", serde_json::to_string_pretty(&obj)?);
    Ok(())
}
