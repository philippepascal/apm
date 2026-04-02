use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn create(root: &Path, title: &str) -> Result<String> {
    let id = crate::git::gen_hex_id();
    let slug = crate::ticket::slugify(title);
    let branch = format!("epic/{id}-{slug}");

    // Fetch origin/main; propagate error if it doesn't exist.
    let fetch_out = Command::new("git")
        .current_dir(root)
        .args(["fetch", "origin", "main"])
        .output()
        .map_err(|e| anyhow::anyhow!("git not found: {e}"))?;
    if !fetch_out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&fetch_out.stderr).trim()
        );
    }

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

    let add_out = Command::new("git")
        .current_dir(root)
        .args([
            "worktree",
            "add",
            "-b",
            &branch,
            &wt_path.to_string_lossy(),
            "origin/main",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("git not found: {e}"))?;
    if !add_out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&add_out.stderr).trim()
        );
    }

    let result = (|| -> Result<()> {
        let epic_md = wt_path.join("EPIC.md");
        std::fs::write(&epic_md, format!("# {title}\n"))?;

        let stage_out = Command::new("git")
            .current_dir(&wt_path)
            .args(["add", "EPIC.md"])
            .output()?;
        if !stage_out.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&stage_out.stderr).trim());
        }

        let commit_msg = format!("epic({id}): create {title}");
        let commit_out = Command::new("git")
            .current_dir(&wt_path)
            .args(["commit", "-m", &commit_msg])
            .output()?;
        if !commit_out.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&commit_out.stderr).trim());
        }
        Ok(())
    })();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["worktree", "remove", "--force", &wt_path.to_string_lossy()])
        .output();
    let _ = std::fs::remove_dir_all(&wt_path);

    result?;

    crate::git::push_branch_tracking(root, &branch)?;

    Ok(branch)
}
