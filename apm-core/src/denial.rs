/// Transcript denial scanner for APM worker logs.
///
/// # Event format (stream-json JSONL from `claude --output-format stream-json`)
///
/// Tool use — carried in an assistant message event:
/// ```json
/// {
///   "type": "assistant",
///   "message": {
///     "role": "assistant",
///     "content": [
///       {
///         "type": "tool_use",
///         "id": "toolu_01JZREMrBXn3AkQaBfgyaFvc",
///         "name": "Bash",
///         "input": { "command": "apm doesnotexist", "description": "..." }
///       }
///     ]
///   }
/// }
/// ```
///
/// Tool result — carried in a user message event (includes timestamp):
/// ```json
/// {
///   "type": "user",
///   "message": {
///     "role": "user",
///     "content": [
///       {
///         "type": "tool_result",
///         "tool_use_id": "toolu_01JZREMrBXn3AkQaBfgyaFvc",
///         "is_error": true,
///         "content": "cannot be auto-allowed"
///       }
///     ]
///   },
///   "timestamp": "2026-05-02T03:28:24.500Z"
/// }
/// ```
///
/// # Discriminating permission denials from regular errors
///
/// Both use `is_error: true`.  Regular Bash failures have content starting with
/// `"Exit code "` followed by a digit.  Permission denials never do.
///
/// Confirmed denial substrings (from real `.apm-worker.log` files):
/// - `"but you haven't granted it yet"` — Write/Edit to an unapproved path
/// - `"was blocked. For security"` — Bash output redirection blocked
/// - `"cannot be auto-allowed"` — Bash pattern rule mismatch (e.g. `find -exec`)
/// - `"Approve only if you trust it"` — compound `cd && git` safety warning

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Classification of a permission denial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialClass {
    /// Denied a Bash call whose command starts with `apm `.  APM should never
    /// deny its own commands; this indicates a default-allowlist gap.
    ApmCommandDenial,
    /// Denied an Edit or Write whose path falls outside the ticket worktree.
    OutsideWorktree,
    /// Any other denial not matching the two patterns above.
    UnknownPattern,
}

/// One denied tool call extracted from the transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenialEntry {
    /// ISO-8601 timestamp from the tool-result event, or empty string.
    pub timestamp: String,
    /// Tool name, e.g. `"Bash"`, `"Edit"`, `"Write"`.
    pub tool: String,
    /// Tool input (command string for Bash; serialised JSON for others),
    /// truncated to ≤200 chars.
    pub input: String,
    pub classification: DenialClass,
}

/// Summary written alongside `.apm-worker.log` on worker exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenialSummary {
    pub ticket_id: String,
    /// ISO-8601 timestamp of when the scan ran (worker exit time).
    pub worker_exited_at: String,
    /// Absolute path to the `.apm-worker.log` file.
    pub log_path: String,
    pub denial_count: usize,
    pub denials: Vec<DenialEntry>,
}

/// Scan `log_path` for permission-denial events and return a summary.
///
/// Returns an empty summary (zero denials) if the file is missing or
/// unreadable.
pub fn scan_transcript(log_path: &Path, worktree: &Path, ticket_id: &str) -> DenialSummary {
    let content = match std::fs::read_to_string(log_path) {
        Ok(c) => c,
        Err(_) => {
            return empty_summary(log_path, ticket_id);
        }
    };

    // Pass 1 — build tool_use_id → (tool_name, input_value, timestamp) map.
    // The timestamp on assistant-message lines is often absent; we capture it
    // in case it is present, but the denial timestamp comes from the
    // tool-result line in pass 2.
    let mut tool_uses: HashMap<String, (String, serde_json::Value, String)> = HashMap::new();

    for line in content.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["type"] != "assistant" {
            continue;
        }
        let ts = v["timestamp"].as_str().unwrap_or("").to_string();
        if let Some(arr) = v["message"]["content"].as_array() {
            for item in arr {
                if item["type"] != "tool_use" {
                    continue;
                }
                let id = item["id"].as_str().unwrap_or("").to_string();
                if id.is_empty() {
                    continue;
                }
                let name = item["name"].as_str().unwrap_or("").to_string();
                let input = item["input"].clone();
                tool_uses.insert(id, (name, input, ts.clone()));
            }
        }
    }

    // Pass 2 — find denied tool_result events.
    let canon_worktree = std::fs::canonicalize(worktree)
        .unwrap_or_else(|_| worktree.to_path_buf());

    let mut denials: Vec<DenialEntry> = Vec::new();

    for line in content.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["type"] != "user" {
            continue;
        }
        let result_ts = v["timestamp"].as_str().unwrap_or("").to_string();
        let Some(arr) = v["message"]["content"].as_array() else { continue };

        for item in arr {
            if item["type"] != "tool_result" {
                continue;
            }
            if item["is_error"] != true {
                continue;
            }
            // Discriminate denial from regular error: denials never start with "Exit code "
            let content_str = match item["content"].as_str() {
                Some(s) => s,
                None => continue,
            };
            if content_str.starts_with("Exit code ") {
                continue;
            }

            let tool_use_id = item["tool_use_id"].as_str().unwrap_or("");
            let Some((tool_name, input_obj, _)) = tool_uses.get(tool_use_id) else { continue };

            let (input_str, classification) =
                classify_denial(tool_name, input_obj, &canon_worktree, worktree);

            denials.push(DenialEntry {
                timestamp: result_ts.clone(),
                tool: tool_name.clone(),
                input: truncate_str(&input_str, 200),
                classification,
            });
        }
    }

    DenialSummary {
        ticket_id: ticket_id.to_string(),
        worker_exited_at: chrono::Utc::now().to_rfc3339(),
        log_path: log_path.to_string_lossy().into_owned(),
        denial_count: denials.len(),
        denials,
    }
}

/// Write `summary` to `summary_path` as pretty-printed JSON.
/// Errors are logged and swallowed — this must not panic or crash the
/// wrapper exit path.
pub fn write_summary(summary_path: &Path, summary: &DenialSummary) {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            if let Err(e) = std::fs::write(summary_path, json) {
                crate::logger::log("worker-diag", &format!("write_summary failed: {e}"));
            }
        }
        Err(e) => {
            crate::logger::log("worker-diag", &format!("write_summary serialize failed: {e}"));
        }
    }
}

/// Read a previously written summary from `summary_path`.
/// Returns `None` if the file is absent or cannot be parsed.
pub fn read_summary(summary_path: &Path) -> Option<DenialSummary> {
    let content = std::fs::read_to_string(summary_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Derive the `.apm-worker.summary.json` path from the `.apm-worker.log` path.
/// If the log path ends in `.log`, replaces that suffix; otherwise appends
/// `.summary.json`.
pub fn summary_path_for(log_path: &Path) -> std::path::PathBuf {
    if log_path.extension().and_then(|e| e.to_str()) == Some("log") {
        log_path.with_extension("summary.json")
    } else {
        let mut p = log_path.to_path_buf();
        let name = p.file_name()
            .map(|n| format!("{}.summary.json", n.to_string_lossy()))
            .unwrap_or_else(|| "summary.json".to_string());
        p.set_file_name(name);
        p
    }
}

/// Return the unique command strings from all `ApmCommandDenial` entries in
/// `summary`, preserving first-seen order.
pub fn collect_unique_apm_commands(summary: &DenialSummary) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    summary
        .denials
        .iter()
        .filter(|d| d.classification == DenialClass::ApmCommandDenial)
        .filter(|d| seen.insert(d.input.clone()))
        .map(|d| d.input.clone())
        .collect()
}

// ── internal helpers ──────────────────────────────────────────────────────────

fn empty_summary(log_path: &Path, ticket_id: &str) -> DenialSummary {
    DenialSummary {
        ticket_id: ticket_id.to_string(),
        worker_exited_at: chrono::Utc::now().to_rfc3339(),
        log_path: log_path.to_string_lossy().into_owned(),
        denial_count: 0,
        denials: Vec::new(),
    }
}

/// Return (input_string, classification) for a single denial.
fn classify_denial(
    tool: &str,
    input_obj: &serde_json::Value,
    canon_worktree: &Path,
    raw_worktree: &Path,
) -> (String, DenialClass) {
    match tool {
        "Bash" => {
            let command = input_obj["command"].as_str().unwrap_or("").to_string();
            let class = if command.trim().starts_with("apm ") {
                DenialClass::ApmCommandDenial
            } else {
                DenialClass::UnknownPattern
            };
            (command, class)
        }
        "Edit" | "Write" => {
            let file_path_str = input_obj["file_path"].as_str().unwrap_or("");
            let class = if !file_path_str.is_empty()
                && is_outside_worktree(file_path_str, canon_worktree, raw_worktree)
            {
                DenialClass::OutsideWorktree
            } else {
                DenialClass::UnknownPattern
            };
            let input_str = serde_json::to_string(input_obj).unwrap_or_default();
            (input_str, class)
        }
        _ => {
            let input_str = serde_json::to_string(input_obj).unwrap_or_default();
            (input_str, DenialClass::UnknownPattern)
        }
    }
}

/// Return true if `file_path_str` resolves to a path outside `canon_worktree`.
///
/// Path resolution rules:
/// - `canon_worktree` is the result of `fs::canonicalize(worktree)`, or the raw
///   worktree path if canonicalization fails (worktree does not exist yet).
/// - Absolute `file_path_str`: attempt canonicalize; on failure use as-is.
/// - Relative `file_path_str`: join with `raw_worktree` first, then attempt
///   canonicalize; on failure use the joined form.
fn is_outside_worktree(file_path_str: &str, canon_worktree: &Path, raw_worktree: &Path) -> bool {
    let file_path = Path::new(file_path_str);

    let resolved: PathBuf = if file_path.is_absolute() {
        std::fs::canonicalize(file_path).unwrap_or_else(|_| file_path.to_path_buf())
    } else {
        let joined = raw_worktree.join(file_path);
        std::fs::canonicalize(&joined).unwrap_or(joined)
    };

    !resolved.starts_with(canon_worktree)
}

fn truncate_str(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        // Truncate at a char boundary
        let mut end = max_bytes;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s[..end].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn test_apm_command_denial() {
        let log_path = fixture_path("transcript_apm_denial.jsonl");
        let worktree = Path::new("/fake/worktree");
        let summary = scan_transcript(&log_path, worktree, "testticket");

        assert_eq!(summary.denial_count, 1, "expected 1 denial");
        assert_eq!(summary.denials[0].classification, DenialClass::ApmCommandDenial);
        assert_eq!(summary.denials[0].tool, "Bash");
        assert!(
            summary.denials[0].input.starts_with("apm "),
            "input should start with 'apm ', got: {:?}",
            summary.denials[0].input
        );
    }

    #[test]
    fn test_no_denials() {
        let log_path = fixture_path("transcript_no_denials.jsonl");
        let worktree = Path::new("/fake/worktree");
        let summary = scan_transcript(&log_path, worktree, "testticket");

        assert_eq!(summary.denial_count, 0, "expected 0 denials");
        assert!(summary.denials.is_empty());
    }

    #[test]
    fn test_outside_worktree() {
        let log_path = fixture_path("transcript_outside_worktree.jsonl");
        let worktree = Path::new("/fake/worktree");
        let summary = scan_transcript(&log_path, worktree, "testticket");

        assert_eq!(summary.denial_count, 1, "expected 1 denial");
        assert_eq!(summary.denials[0].classification, DenialClass::OutsideWorktree);
    }

    #[test]
    fn test_missing_transcript_returns_empty_summary() {
        let log_path = Path::new("/nonexistent/path/log.jsonl");
        let summary = scan_transcript(log_path, Path::new("/fake/worktree"), "t1");
        assert_eq!(summary.denial_count, 0);
    }

    #[test]
    fn test_regular_error_not_classified_as_denial() {
        // A Bash error starting with "Exit code" must not be treated as a denial
        let content = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"false"}}]}}
{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","is_error":true,"content":"Exit code 1"}]},"timestamp":"2026-01-01T00:00:00Z"}
"#;
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("test.jsonl");
        std::fs::write(&log, content).unwrap();
        let summary = scan_transcript(&log, dir.path(), "t");
        assert_eq!(summary.denial_count, 0);
    }

    #[test]
    fn test_truncate_str_at_boundary() {
        let s = "apm state xyz implemented";
        let truncated = truncate_str(s, 10);
        assert_eq!(truncated.len(), 10);
        assert!(s.starts_with(&truncated));
    }

    #[test]
    fn test_write_and_read_summary_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let summary = DenialSummary {
            ticket_id: "abc123".to_string(),
            worker_exited_at: "2026-01-01T00:00:00Z".to_string(),
            log_path: "/fake/log".to_string(),
            denial_count: 1,
            denials: vec![DenialEntry {
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                tool: "Bash".to_string(),
                input: "apm state abc implemented".to_string(),
                classification: DenialClass::ApmCommandDenial,
            }],
        };
        let path = dir.path().join("summary.json");
        write_summary(&path, &summary);
        let loaded = read_summary(&path).expect("should be readable");
        assert_eq!(loaded.ticket_id, "abc123");
        assert_eq!(loaded.denial_count, 1);
        assert_eq!(loaded.denials[0].classification, DenialClass::ApmCommandDenial);
    }
}
