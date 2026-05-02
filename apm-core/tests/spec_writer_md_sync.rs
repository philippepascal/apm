/// Asserts that the `## Style rules` section in
/// `apm-core/src/default/agents/claude/apm.spec-writer.md` and
/// `.apm/agents/claude/apm.spec-writer.md` are identical.
///
/// The rest of each file may differ (the project file is legitimately
/// customisable), but the style rules must stay in sync so both files
/// give the spec-writer the same instructions.
#[test]
fn spec_writer_style_rules_section_is_identical() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let default_path = std::path::Path::new(manifest_dir)
        .join("src/default/agents/claude/apm.spec-writer.md");
    let project_path = std::path::Path::new(manifest_dir)
        .parent()
        .expect("apm-core has a parent directory")
        .join(".apm/agents/claude/apm.spec-writer.md");

    let default_content = std::fs::read_to_string(&default_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", default_path.display()));
    let project_content = std::fs::read_to_string(&project_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", project_path.display()));

    let default_section = extract_style_rules_section(&default_content);
    let project_section = extract_style_rules_section(&project_content);

    if default_section == project_section {
        return;
    }

    // Produce a line-level diff of just the diverging section.
    let default_lines: Vec<&str> = default_section.lines().collect();
    let project_lines: Vec<&str> = project_section.lines().collect();
    let max = default_lines.len().max(project_lines.len());
    let mut diff_lines: Vec<String> = Vec::new();
    for i in 0..max {
        match (default_lines.get(i), project_lines.get(i)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                diff_lines.push(format!("-line {}: {:?}", i + 1, a));
                diff_lines.push(format!("+line {}: {:?}", i + 1, b));
            }
            (Some(a), None) => {
                diff_lines.push(format!(
                    "-line {}: {:?} (missing in project file)",
                    i + 1,
                    a
                ));
            }
            (None, Some(b)) => {
                diff_lines.push(format!(
                    "+line {}: {:?} (missing in default file)",
                    i + 1,
                    b
                ));
            }
            (None, None) => {}
        }
    }

    panic!(
        "## Style rules section has diverged between:\n  \
         {}\n  \
         {}\n\
         Edit both files together to keep them in sync.\n\
         Diff:\n{}",
        default_path.display(),
        project_path.display(),
        diff_lines.join("\n")
    );
}

/// Extracts lines from the `## Style rules` heading to the next `##`-level
/// heading or EOF. Returns the extracted slice as a String.
fn extract_style_rules_section(content: &str) -> String {
    let mut lines = content.lines();
    let mut in_section = false;
    let mut result: Vec<&str> = Vec::new();

    for line in lines.by_ref() {
        if line.starts_with("## Style rules") {
            in_section = true;
            result.push(line);
            continue;
        }
        if in_section {
            // Stop at the next `##`-level heading (but not `###` or deeper).
            if line.starts_with("## ") {
                break;
            }
            result.push(line);
        }
    }

    result.join("\n")
}
