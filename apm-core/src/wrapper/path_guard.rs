use std::path::{Component, Path, PathBuf};
use globset::{Glob, GlobSetBuilder};

pub struct PathGuard {
    worktree: PathBuf,             // canonicalised APM_TICKET_WORKTREE
    write_protected: Vec<PathBuf>, // APM_BIN, APM_SYSTEM_PROMPT_FILE, APM_USER_MESSAGE_FILE
}

impl PathGuard {
    pub fn new(
        worktree: &Path,
        read_allow_patterns: &[String],
        write_protected: &[PathBuf],
    ) -> anyhow::Result<Self> {
        let worktree = std::fs::canonicalize(worktree)
            .unwrap_or_else(|_| canonicalize_lenient(worktree));

        // Validate read_allow patterns so callers get an error on bad globs,
        // even though read-only commands are always permitted (they produce no
        // write targets that need checking).
        let mut builder = GlobSetBuilder::new();
        for pattern in read_allow_patterns {
            let expanded = expand_home_str(pattern);
            builder.add(Glob::new(&expanded).map_err(|e| anyhow::anyhow!("invalid glob {pattern:?}: {e}"))?);
        }
        builder.build().map_err(|e| anyhow::anyhow!("glob build failed: {e}"))?;

        let write_protected = write_protected
            .iter()
            .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| canonicalize_lenient(p)))
            .collect();

        Ok(PathGuard { worktree, write_protected })
    }

    pub fn check_write(&self, path: &Path) -> Result<(), String> {
        let resolved = canonicalize_lenient(path);

        // write_protected entries are always rejected — even if inside worktree
        if self.write_protected.iter().any(|p| p == &resolved) {
            return Err(rejection_msg(path, &self.worktree));
        }

        if resolved.starts_with(&self.worktree) {
            return Ok(());
        }

        Err(rejection_msg(path, &self.worktree))
    }

    pub fn check_bash(&self, cmd: &str) -> Result<(), String> {
        let targets = detect_write_targets(cmd);
        for target_str in targets {
            let path = PathBuf::from(&target_str);
            self.check_write(&path)?;
        }
        Ok(())
    }
}

/// Build a human-readable rejection message.
fn rejection_msg(requested: &Path, worktree: &Path) -> String {
    format!(
        "path outside ticket worktree; isolation enforced by APM wrapper.\n  Requested: {}\n  APM_TICKET_WORKTREE = {}",
        requested.display(),
        worktree.display()
    )
}

/// Canonicalize a path, following symlinks for components that exist on disk
/// and appending non-existent components lexically.
///
/// This ensures that existing intermediate symlinks are resolved while still
/// accepting paths to files that do not yet exist (e.g. the target of a Write
/// call that would create a new file).
pub fn canonicalize_lenient(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(p) => {
                result = PathBuf::from(p.as_os_str());
            }
            Component::RootDir => {
                result.push(component);
            }
            Component::CurDir => {
                // skip "."
            }
            Component::ParentDir => {
                // Try to canonicalize the path with ".." so the OS resolves symlinks
                let candidate = result.join("..");
                if candidate.exists() {
                    result = std::fs::canonicalize(&candidate).unwrap_or(candidate);
                } else {
                    // Non-existent parent: lexically remove the last component
                    result.pop();
                }
            }
            Component::Normal(_) => {
                result.push(component);
                if result.exists() {
                    result = std::fs::canonicalize(&result).unwrap_or_else(|_| result.clone());
                }
            }
        }
    }

    result
}

/// Expand `~` at the start of a path string to the user's home directory.
fn expand_home_str(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return format!("{home}/{rest}");
            }
        }
    }
    s.to_string()
}

/// Expand `~/` prefix to the home directory.
fn expand_home(s: &str) -> String {
    expand_home_str(s)
}

fn is_path_token(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("~/")
}

fn is_shell_sep(s: &str) -> bool {
    matches!(s, ";" | "&&" | "||" | "|" | "&")
}

/// Detect write-target paths from a bash command string.
///
/// Handles:
/// - `>` and `>>` redirect targets (space-separated and embedded)
/// - `tee` first non-flag argument
/// - `cp` / `mv` last non-flag path argument (the destination)
/// - `truncate` path argument
///
/// Known false negatives (documented limitation, not in scope):
/// - Paths stored in shell variables: `OUT=/x; echo foo > "$OUT"`
/// - Subshell expansion: `echo foo > $(cat /tmp/path)`
/// - eval: `eval "echo foo > /x"`
fn detect_write_targets(cmd: &str) -> Vec<String> {
    let mut targets = Vec::new();

    // Phase 1: scan for redirect operators (>, >>) at the character level
    detect_redirects(cmd, &mut targets);

    // Phase 2: command-specific write patterns via token scan
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    detect_command_writes(&tokens, &mut targets);

    targets
}

/// Scan the command string character by character for `>` and `>>` operators,
/// skipping quoted strings, and collecting the path that follows.
fn detect_redirects(cmd: &str, targets: &mut Vec<String>) {
    let chars: Vec<char> = cmd.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // Skip single-quoted strings
        if c == '\'' {
            i += 1;
            while i < n && chars[i] != '\'' {
                i += 1;
            }
            if i < n {
                i += 1;
            }
            continue;
        }

        // Skip double-quoted strings
        if c == '"' {
            i += 1;
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1; // skip escaped character
                }
                if i < n {
                    i += 1;
                }
            }
            if i < n {
                i += 1;
            }
            continue;
        }

        if c == '>' {
            let is_double = i + 1 < n && chars[i + 1] == '>';
            let advance = if is_double { 2 } else { 1 };

            // Skip whitespace after the redirect operator
            let mut j = i + advance;
            while j < n && chars[j] == ' ' {
                j += 1;
            }

            // Collect path token if it starts with / or ~/
            if j < n
                && (chars[j] == '/'
                    || (chars[j] == '~' && j + 1 < n && chars[j + 1] == '/'))
            {
                let path_start = j;
                while j < n
                    && !chars[j].is_whitespace()
                    && !matches!(chars[j], ';' | '|' | '&' | '(')
                {
                    j += 1;
                }
                let path: String = chars[path_start..j].iter().collect();
                targets.push(expand_home(&path));
            }

            i += advance;
            continue;
        }

        i += 1;
    }
}

/// Scan tokenised command for command-specific write patterns.
fn detect_command_writes(tokens: &[&str], targets: &mut Vec<String>) {
    let n = tokens.len();
    let mut i = 0;

    while i < n {
        let tok = tokens[i];

        match tok {
            "tee" => {
                // First non-flag absolute path argument
                for j in (i + 1)..n {
                    let arg = tokens[j];
                    if is_shell_sep(arg) {
                        break;
                    }
                    if arg.starts_with('-') {
                        continue;
                    }
                    if is_path_token(arg) {
                        targets.push(expand_home(arg));
                    }
                    break;
                }
            }
            "cp" | "mv" => {
                // Destination = last non-flag absolute path argument
                let mut last: Option<String> = None;
                for j in (i + 1)..n {
                    let arg = tokens[j];
                    if is_shell_sep(arg) {
                        break;
                    }
                    if !arg.starts_with('-') && is_path_token(arg) {
                        last = Some(expand_home(arg));
                    }
                }
                if let Some(p) = last {
                    targets.push(p);
                }
            }
            "truncate" => {
                let mut j = i + 1;
                while j < n {
                    let arg = tokens[j];
                    if is_shell_sep(arg) {
                        break;
                    }
                    // -s / --size each consume the next token as the size value
                    if arg == "-s" || arg == "--size" {
                        j += 2;
                        continue;
                    }
                    if !arg.starts_with('-') && is_path_token(arg) {
                        targets.push(expand_home(arg));
                    }
                    j += 1;
                }
            }
            _ => {}
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- canonicalize_lenient ----

    #[test]
    fn canonicalize_lenient_absolute_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().to_path_buf();
        let result = canonicalize_lenient(&p);
        // Should resolve the tempdir path (may follow symlinks)
        assert!(result.is_absolute());
    }

    #[test]
    fn canonicalize_lenient_nonexistent_leaf() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("nonexistent.txt");
        let result = canonicalize_lenient(&p);
        // Parent must be resolved; leaf appended lexically
        assert!(result.is_absolute());
        assert_eq!(result.file_name().unwrap().to_str().unwrap(), "nonexistent.txt");
    }

    #[test]
    fn canonicalize_lenient_dotdot_inside_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        // sub/.. == tmp.path()
        let candidate = sub.join("..").join("other.txt");
        let result = canonicalize_lenient(&candidate);
        // result should be tmp.path()/other.txt
        let expected_parent = std::fs::canonicalize(tmp.path()).unwrap();
        assert_eq!(result.parent().unwrap(), expected_parent);
    }

    #[test]
    fn canonicalize_lenient_dotdot_escape_stays_out() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("worktree");
        let sub = wt.join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        // worktree/subdir/../../etc/passwd
        let path = sub.join("..").join("..").join("etc").join("passwd");
        let result = canonicalize_lenient(&path);
        // Should resolve to tmp.path()/etc/passwd — outside wt
        let canon_wt = std::fs::canonicalize(&wt).unwrap();
        assert!(!result.starts_with(&canon_wt));
    }

    #[test]
    fn canonicalize_lenient_symlink_inside_worktree_resolves_outside() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let outside = tmp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        let link = wt.join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        let target = link.join("secret.txt");
        let result = canonicalize_lenient(&target);
        let canon_wt = std::fs::canonicalize(&wt).unwrap();
        assert!(!result.starts_with(&canon_wt));
    }

    // ---- PathGuard::check_write ----

    fn make_guard(wt: &Path) -> PathGuard {
        PathGuard::new(wt, &[], &[]).unwrap()
    }

    #[test]
    fn check_write_inside_worktree_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        assert!(guard.check_write(&wt.join("file.txt")).is_ok());
    }

    #[test]
    fn check_write_outside_worktree_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        let outside = tmp.path().join("outside.txt");
        let err = guard.check_write(&outside).unwrap_err();
        assert!(err.contains("path outside ticket worktree"));
        assert!(err.contains("APM_TICKET_WORKTREE"));
    }

    #[test]
    fn check_write_rejection_message_contains_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        let err = guard.check_write(&tmp.path().join("x")).unwrap_err();
        assert!(err.contains("APM_TICKET_WORKTREE"));
    }

    #[test]
    fn check_write_dotdot_escape_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        let sub = wt.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        let guard = make_guard(&wt);
        // wt/sub/../../etc/passwd
        let path = sub.join("..").join("..").join("etc").join("passwd");
        assert!(guard.check_write(&path).is_err());
    }

    #[test]
    fn check_write_symlink_to_outside_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let outside = tmp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        let link = wt.join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        let guard = make_guard(&wt);
        assert!(guard.check_write(&link.join("file.txt")).is_err());
    }

    #[test]
    fn check_write_protected_inside_worktree_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        // apm_bin inside the worktree (e.g. target/debug/apm)
        let apm_bin = wt.join("target").join("debug").join("apm");
        std::fs::create_dir_all(apm_bin.parent().unwrap()).unwrap();
        std::fs::write(&apm_bin, "binary").unwrap();
        let guard = PathGuard::new(&wt, &[], &[apm_bin.clone()]).unwrap();
        let err = guard.check_write(&apm_bin).unwrap_err();
        assert!(err.contains("path outside ticket worktree"));
    }

    #[test]
    fn check_write_apm_bin_outside_worktree_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let apm_bin = tmp.path().join("usr").join("bin").join("apm");
        std::fs::create_dir_all(apm_bin.parent().unwrap()).unwrap();
        std::fs::write(&apm_bin, "binary").unwrap();
        let guard = PathGuard::new(&wt, &[], &[apm_bin.clone()]).unwrap();
        assert!(guard.check_write(&apm_bin).is_err());
    }

    // ---- detect_write_targets (bash heuristic) ----

    #[test]
    fn bash_redirect_gt_detected() {
        let targets = detect_write_targets("echo foo > /outside/file");
        assert!(targets.iter().any(|t| t == "/outside/file"), "got: {targets:?}");
    }

    #[test]
    fn bash_redirect_gtgt_detected() {
        let targets = detect_write_targets("cat data >> /outside/append.log");
        assert!(targets.iter().any(|t| t == "/outside/append.log"), "got: {targets:?}");
    }

    #[test]
    fn bash_tee_detected() {
        let targets = detect_write_targets("some-cmd | tee /outside/output.txt");
        assert!(targets.iter().any(|t| t == "/outside/output.txt"), "got: {targets:?}");
    }

    #[test]
    fn bash_tee_flag_skipped() {
        let targets = detect_write_targets("some-cmd | tee -a /outside/output.txt");
        assert!(targets.iter().any(|t| t == "/outside/output.txt"), "got: {targets:?}");
    }

    #[test]
    fn bash_cp_dest_detected() {
        let targets = detect_write_targets("cp /inside/src /outside/dest");
        assert!(targets.iter().any(|t| t == "/outside/dest"), "got: {targets:?}");
        // /inside/src should NOT be a target
        assert!(!targets.iter().any(|t| t == "/inside/src"), "src should not be write target: {targets:?}");
    }

    #[test]
    fn bash_mv_dest_detected() {
        let targets = detect_write_targets("mv /inside/file /outside/dest");
        assert!(targets.iter().any(|t| t == "/outside/dest"), "got: {targets:?}");
    }

    #[test]
    fn bash_truncate_detected() {
        let targets = detect_write_targets("truncate -s 0 /outside/file");
        assert!(targets.iter().any(|t| t == "/outside/file"), "got: {targets:?}");
    }

    #[test]
    fn bash_cat_not_detected() {
        let targets = detect_write_targets("cat /etc/resolv.conf");
        assert!(targets.is_empty(), "cat should produce no write targets: {targets:?}");
    }

    #[test]
    fn bash_grep_not_detected() {
        let targets = detect_write_targets("grep pattern /etc/hosts");
        assert!(targets.is_empty(), "grep should produce no write targets: {targets:?}");
    }

    #[test]
    fn bash_ls_not_detected() {
        let targets = detect_write_targets("ls /outside/dir");
        assert!(targets.is_empty(), "ls should produce no write targets: {targets:?}");
    }

    #[test]
    fn bash_diff_not_detected() {
        let targets = detect_write_targets("diff /file1 /file2");
        assert!(targets.is_empty(), "diff should produce no write targets: {targets:?}");
    }

    #[test]
    fn bash_wc_not_detected() {
        let targets = detect_write_targets("wc -l /var/log/syslog");
        assert!(targets.is_empty(), "wc should produce no write targets: {targets:?}");
    }

    #[test]
    fn bash_echo_no_path_not_detected() {
        let targets = detect_write_targets("echo hello");
        assert!(targets.is_empty(), "echo without path should produce no write targets: {targets:?}");
    }

    // ---- PathGuard::check_bash ----

    #[test]
    fn check_bash_redirect_outside_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        let outside = tmp.path().join("outside.txt");
        let cmd = format!("echo foo > {}", outside.display());
        assert!(guard.check_bash(&cmd).is_err());
    }

    #[test]
    fn check_bash_redirect_inside_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        let inside = wt.join("output.txt");
        let cmd = format!("echo foo > {}", inside.display());
        assert!(guard.check_bash(&cmd).is_ok());
    }

    #[test]
    fn check_bash_cat_read_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        assert!(guard.check_bash("cat /etc/resolv.conf").is_ok());
    }

    #[test]
    fn check_bash_tilde_gitconfig_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir(&wt).unwrap();
        let guard = make_guard(&wt);
        assert!(guard.check_bash("cat ~/.gitconfig").is_ok());
    }
}
