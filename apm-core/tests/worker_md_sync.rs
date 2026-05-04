/// Asserts that `apm-core/src/default/agents/default/apm.worker.md` and
/// `.apm/agents/default/apm.worker.md` are byte-for-byte identical. Any
/// divergence fails with a diff so the developer can see exactly what changed.
#[test]
fn default_and_project_apm_worker_md_are_identical() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let default_path = std::path::Path::new(manifest_dir).join("src/default/agents/default/apm.worker.md");
    let project_path = std::path::Path::new(manifest_dir)
        .parent()
        .expect("apm-core has a parent directory")
        .join(".apm/agents/default/apm.worker.md");

    let default_bytes = std::fs::read(&default_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", default_path.display()));
    let project_bytes = std::fs::read(&project_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", project_path.display()));

    if default_bytes == project_bytes {
        return;
    }

    let default_str = String::from_utf8_lossy(&default_bytes);
    let project_str = String::from_utf8_lossy(&project_bytes);

    // Produce a simple line-level diff so the developer sees what diverged.
    let default_lines: Vec<&str> = default_str.lines().collect();
    let project_lines: Vec<&str> = project_str.lines().collect();
    let max = default_lines.len().max(project_lines.len());
    let mut diff_lines: Vec<String> = Vec::new();
    for i in 0..max {
        match (default_lines.get(i), project_lines.get(i)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                diff_lines.push(format!("line {}: default=  {:?}", i + 1, a));
                diff_lines.push(format!("line {}: project=  {:?}", i + 1, b));
            }
            (Some(a), None) => {
                diff_lines.push(format!("line {}: default=  {:?} (missing in project)", i + 1, a));
            }
            (None, Some(b)) => {
                diff_lines.push(format!("line {}: project=  {:?} (missing in default)", i + 1, b));
            }
            (None, None) => {}
        }
    }

    panic!(
        "apm-core/src/default/agents/default/apm.worker.md and \
         .apm/agents/default/apm.worker.md have diverged.\n\
         Edit both files together to keep them in sync.\n\
         Diff (first differences):\n{}",
        diff_lines.join("\n")
    );
}

/// Asserts that `apm-core/src/default/agents/claude/apm.worker.md` and
/// `.apm/agents/claude/apm.worker.md` are byte-for-byte identical. Any
/// divergence fails with a diff so the developer can see exactly what changed.
#[test]
fn default_and_per_agent_apm_worker_md_are_identical() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let default_path =
        std::path::Path::new(manifest_dir).join("src/default/agents/claude/apm.worker.md");
    let project_path = std::path::Path::new(manifest_dir)
        .parent()
        .expect("apm-core has a parent directory")
        .join(".apm/agents/claude/apm.worker.md");

    let default_bytes = std::fs::read(&default_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", default_path.display()));
    let project_bytes = std::fs::read(&project_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", project_path.display()));

    if default_bytes == project_bytes {
        return;
    }

    let default_str = String::from_utf8_lossy(&default_bytes);
    let project_str = String::from_utf8_lossy(&project_bytes);

    // Produce a simple line-level diff so the developer sees what diverged.
    let default_lines: Vec<&str> = default_str.lines().collect();
    let project_lines: Vec<&str> = project_str.lines().collect();
    let max = default_lines.len().max(project_lines.len());
    let mut diff_lines: Vec<String> = Vec::new();
    for i in 0..max {
        match (default_lines.get(i), project_lines.get(i)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                diff_lines.push(format!("line {}: default=  {:?}", i + 1, a));
                diff_lines.push(format!("line {}: project=  {:?}", i + 1, b));
            }
            (Some(a), None) => {
                diff_lines.push(format!(
                    "line {}: default=  {:?} (missing in project)",
                    i + 1,
                    a
                ));
            }
            (None, Some(b)) => {
                diff_lines.push(format!(
                    "line {}: project=  {:?} (missing in default)",
                    i + 1,
                    b
                ));
            }
            (None, None) => {}
        }
    }

    panic!(
        "apm-core/src/default/agents/claude/apm.worker.md and \
         .apm/agents/claude/apm.worker.md have diverged.\n\
         Edit both files together to keep them in sync.\n\
         Diff (first differences):\n{}",
        diff_lines.join("\n")
    );
}

/// Asserts that `apm-core/src/default/agents/default/apm.spec-writer.md` and
/// `.apm/agents/default/apm.spec-writer.md` are byte-for-byte identical. Any
/// divergence fails with a diff so the developer can see exactly what changed.
#[test]
fn default_and_project_apm_spec_writer_md_are_identical() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let default_path =
        std::path::Path::new(manifest_dir).join("src/default/agents/default/apm.spec-writer.md");
    let project_path = std::path::Path::new(manifest_dir)
        .parent()
        .expect("apm-core has a parent directory")
        .join(".apm/agents/default/apm.spec-writer.md");

    let default_bytes = std::fs::read(&default_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", default_path.display()));
    let project_bytes = std::fs::read(&project_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", project_path.display()));

    if default_bytes == project_bytes {
        return;
    }

    let default_str = String::from_utf8_lossy(&default_bytes);
    let project_str = String::from_utf8_lossy(&project_bytes);

    let default_lines: Vec<&str> = default_str.lines().collect();
    let project_lines: Vec<&str> = project_str.lines().collect();
    let max = default_lines.len().max(project_lines.len());
    let mut diff_lines: Vec<String> = Vec::new();
    for i in 0..max {
        match (default_lines.get(i), project_lines.get(i)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                diff_lines.push(format!("line {}: default=  {:?}", i + 1, a));
                diff_lines.push(format!("line {}: project=  {:?}", i + 1, b));
            }
            (Some(a), None) => {
                diff_lines.push(format!(
                    "line {}: default=  {:?} (missing in project)",
                    i + 1,
                    a
                ));
            }
            (None, Some(b)) => {
                diff_lines.push(format!(
                    "line {}: project=  {:?} (missing in default)",
                    i + 1,
                    b
                ));
            }
            (None, None) => {}
        }
    }

    panic!(
        "apm-core/src/default/agents/default/apm.spec-writer.md and \
         .apm/agents/default/apm.spec-writer.md have diverged.\n\
         Edit both files together to keep them in sync.\n\
         Diff (first differences):\n{}",
        diff_lines.join("\n")
    );
}
