+++
id = "52e12de8"
title = "Derive worktree gitignore pattern from config; validate enforces it"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/52e12de8-derive-worktree-gitignore-pattern-from-c"
created_at = "2026-04-28T19:54:13.505295Z"
updated_at = "2026-04-28T20:17:43.629359Z"
+++

## Spec

### Problem

`apm init`'s gitignore writer hardcodes `/worktrees/` regardless of the configured `worktrees.dir`, and `apm validate` doesn't check that the configured in-repo worktree dir is gitignored. Together these mean a user who customizes `worktrees.dir` ends up with worktree contents visible to git, with no detection at runtime.

**Concrete incident:** user changed `.apm/config.toml` from `dir = "../apm--worktrees"` (external) to `dir = ".apm--worktrees"` (in-repo, hidden). `.gitignore` was not updated. `apm validate` ran clean. The user only noticed when they opened `git status` and saw worktree contents staged for inclusion.

**Fix 1: `ensure_gitignore` must derive from config.**

Location: `apm-core/src/init.rs:194-217`, the `entries` array currently includes the literal `"/worktrees/"`.

Change to read `config.worktrees.dir` and emit the gitignore pattern from it:
- If the path is external (starts with `/` for absolute, or `..` for parent-traversal): skip — gitignore doesn't help here.
- Otherwise: emit `/<dir>/` (root-anchored, directory-only). For example `worktrees` → `/worktrees/`; `.apm--worktrees` → `/.apm--worktrees/`; `build/wt` → `/build/wt/`.
- The comment line `# apm worktrees` stays as-is.
- Idempotency check still applies — only append if the exact line is missing.

`ensure_gitignore` currently doesn't take `Config`; it takes `path: &Path`. Either pass the config in, or have `setup()` (the caller in `init.rs`) compute the pattern and pass it as a parameter.

**Fix 2: `apm validate` must check the gitignore.**

Location: `apm-core/src/validate.rs` and `apm/src/cmd/validate.rs`.

Add a check: when `config.worktrees.dir` is in-repo (not external), `.gitignore` must contain a pattern that matches it. Use a loose substring match against any of these forms (any one is acceptable):
- `/<dir>/`
- `/<dir>`
- `<dir>/`
- `<dir>`

Rationale for loose match: gitignore has multiple equivalent ways to ignore a directory; a strict literal-match would reject configs that are functionally correct.

Edge cases:
- `.gitignore` missing entirely → fail with a clear message; suggest re-running `apm init` or adding the line manually. `--fix` should append it (and the comment line) idempotently.
- External path (starts with `/` or `..`) → skip the check entirely; gitignore is irrelevant for paths outside the repo.
- The user's manually-added `.apm--worktrees` (no anchors) — passes the loose match.

This is the "(e)" check that was discussed when 38976b4b shipped but never filed. The hash-trip on config change (b10d957a) already runs `apm validate` on the next command after a config edit, so this check fires automatically when a user changes `worktrees.dir` — they get a clear validate failure pointing at the gitignore drift.

**Test pointers:**

- `init.rs`: `setup` writes `/<configured-dir>/` to `.gitignore`. Verify with custom `worktrees.dir` values: `worktrees`, `.apm--worktrees`, `build/wt`, `/abs/path`, `../external`. The last two should NOT add a worktree line.
- `validate.rs`: missing `.gitignore` for in-repo worktree dir → error. Pattern present in any of the four forms → ok. External worktree dir → no check fires regardless of gitignore content.
- Integration: edit `config.toml` to change `worktrees.dir` without updating `.gitignore`, run an apm command → hash-trip → validate fails with a pointer to the missing gitignore entry.

**Out of scope:**

- Already-tracked files inside the worktree dir (gitignore doesn't affect those — separate one-time migration concern).
- `.git/info/exclude` as an alternative ignore source (intentionally focus on `.gitignore` because it's committed and team-shareable).
- Renaming the worktree directory pattern across all places APM uses it (e.g. clean's filesystem walks).

### Acceptance criteria

- [ ] `apm init` with default config (`worktrees.dir = "worktrees"`) writes `/worktrees/` to `.gitignore`
- [ ] `apm init` with `worktrees.dir = ".apm--worktrees"` writes `/.apm--worktrees/` to `.gitignore`
- [ ] `apm init` with `worktrees.dir = "build/wt"` (nested relative) writes `/build/wt/` to `.gitignore`
- [ ] `apm init` with `worktrees.dir = "/abs/path"` (absolute) does NOT add a worktree line to `.gitignore`
- [ ] `apm init` with `worktrees.dir = "../external"` (parent-relative) does NOT add a worktree line to `.gitignore`
- [ ] Running `apm init` twice with the same config is idempotent: the worktree pattern appears exactly once in `.gitignore`
- [ ] `apm init` writes the `# apm worktrees` comment alongside the pattern when the path is in-repo; the comment is NOT written for external paths
- [ ] `apm validate` fails with an error message when `worktrees.dir` is in-repo and `.gitignore` is absent; the message names the dir and suggests `apm init` or manual addition
- [ ] `apm validate` fails when `worktrees.dir` is in-repo and `.gitignore` exists but does not cover the dir in any recognized form
- [ ] `apm validate` passes when `.gitignore` contains `/<dir>/` (root-anchored, trailing slash)
- [ ] `apm validate` passes when `.gitignore` contains `/<dir>` (root-anchored, no trailing slash)
- [ ] `apm validate` passes when `.gitignore` contains `<dir>/` (unanchored, trailing slash)
- [ ] `apm validate` passes when `.gitignore` contains `<dir>` (bare dirname)
- [ ] `apm validate` emits no gitignore error when `worktrees.dir = "../external"`, even if `.gitignore` is absent
- [ ] `apm validate` emits no gitignore error when `worktrees.dir = "/abs/path"`, even if `.gitignore` is absent
- [ ] `apm validate --fix` appends the worktree pattern and `# apm worktrees` comment to an existing `.gitignore` when they are absent
- [ ] `apm validate --fix` creates `.gitignore` (with all standard APM entries including the worktree pattern) when the file is absent

### Out of scope

- Already-tracked files inside the worktree dir (`git rm --cached` is a separate migration concern)
- `.git/info/exclude` as an alternative ignore source (intentionally focus on `.gitignore` because it is committed and team-shareable)
- Removing stale gitignore patterns when `worktrees.dir` changes (old pattern stays; only new pattern is added)
- Renaming or updating the worktree directory pattern in other APM filesystem walks (e.g., the `clean` command)
- Windows-style backslash path separators

### Approach

Three files change. Changes are purely additive; no existing behaviour is removed.

---

### apm-core/src/init.rs

**New helper** add pub fn worktree_gitignore_pattern(dir: &Path) -> Option<String>:
- Return None if dir starts with / or .. (external path).
- Otherwise return Some(format!("/{s}/")) where s = dir.to_string_lossy().
- Make it pub so the CLI fix path can reuse it without duplicating the logic.

**Signature change** ensure_gitignore(path, messages) gains a new second param: worktree_pattern: Option<&str>.

**Body change** replace the static entries array with a Vec<&str>. Start with the 6 static entries,
then conditionally push "# apm worktrees" and the owned pattern string when worktree_pattern is Some.

**Call-site in setup()** before the ensure_gitignore call at line 137, load config and compute pattern:

    let wt_pattern = crate::config::Config::load(root)
        .ok()
        .and_then(|c| worktree_gitignore_pattern(&c.worktrees.dir));
    ensure_gitignore(&gitignore, wt_pattern.as_deref(), &mut messages)?;

Config is guaranteed to exist at this point (written at line 97 or pre-existing).

**Existing tests to update** (pass None as the new second argument):
- ensure_gitignore_creates_file
- ensure_gitignore_appends_missing_entry
- ensure_gitignore_idempotent
- ensure_gitignore_worktrees_idempotent -- pass Some("/worktrees/") instead;
  update the assertion to check the pattern appears exactly once.

**New unit tests for worktree_gitignore_pattern** -- assert Some/None for:
- "worktrees" -> Some("/worktrees/")
- ".apm--worktrees" -> Some("/.apm--worktrees/")
- "build/wt" -> Some("/build/wt/")
- "/abs/path" -> None
- "../external" -> None

The existing setup_gitignore_includes_worktrees_pattern test needs no change (default config uses
dir = "worktrees" which is in-repo, so setup() still writes /worktrees/).

---

### apm-core/src/validate.rs

**New private helpers** (add near top of file, outside validate_config):

    fn is_external_worktree(dir: &Path) -> bool {
        let s = dir.to_string_lossy();
        s.starts_with("/") || s.starts_with("..")
    }

    fn gitignore_covers_dir(content: &str, dir: &str) -> bool {
        content.lines().any(|line| {
            let line = line.trim();
            line == format!("/{dir}/")
                || line == format!("/{dir}")
                || line == format!("{dir}/")
                || line == dir
        })
    }

**New check inside validate_config()** append after all existing checks, before the final errors return:

    if !is_external_worktree(&config.worktrees.dir) {
        let dir_str = config.worktrees.dir.to_string_lossy();
        let gitignore = root.join(".gitignore");
        match std::fs::read_to_string(&gitignore) {
            Err(_) => errors.push(format!(
                "config: worktrees.dir '{dir_str}' is in-repo but .gitignore is missing; "
                "run 'apm init' or add '/{dir_str}/' manually"
            )),
            Ok(content) if !gitignore_covers_dir(&content, &dir_str) => errors.push(format!(
                "config: worktrees.dir '{dir_str}' is in-repo but .gitignore does not cover it; "
                "add '/{dir_str}/' or run 'apm init'"
            )),
            Ok(_) => {}
        }
    }

**New unit tests** each uses a TempDir, creates an optional .gitignore, builds a minimal config TOML
with [worktrees] dir = "...", and calls validate_config(&config, tmp.path()):

- validate_config_gitignore_missing_in_repo_wt -- in-repo dir, no .gitignore -> error containing dir name
- validate_config_gitignore_covered_anchored_slash -- /<dir>/ -> no error
- validate_config_gitignore_covered_anchored_no_slash -- /<dir> -> no error
- validate_config_gitignore_covered_unanchored_slash -- <dir>/ -> no error
- validate_config_gitignore_covered_bare -- bare dirname on its own line -> no error
- validate_config_gitignore_not_covered -- .gitignore exists but lacks the dir -> error
- validate_config_external_dotdot_no_check -- worktrees.dir = "../ext", no .gitignore -> no gitignore error
- validate_config_external_absolute_no_check -- worktrees.dir = "/abs/path", no .gitignore -> no gitignore error

---

### apm/src/cmd/validate.rs

**Fix path** after the if config_only / else block and before let has_errors = ..., add:

    if fix {
        let pattern = apm_core::init::worktree_gitignore_pattern(&config.worktrees.dir);
        if let Some(p) = pattern {
            let mut msgs = Vec::new();
            apm_core::init::ensure_gitignore(&root.join(".gitignore"), Some(&p), &mut msgs)?;
            for m in &msgs {
                println!("  fixed: {m}");
            }
        }
    }

The config variable is in scope at this point in both config_only and full paths. The current
invocation still exits with the error count after fixing (consistent with branch-field fix behaviour);
a rerun of validate will pass.

Add apm_core::init to the use imports if not already referenced by path.

---

### Constraints / gotchas

- WorktreesConfig::default() in config.rs uses "../worktrees" (external), while default_config() in
  init.rs emits dir = "worktrees" (in-repo). The validate check fires for the in-repo case. Users
  who used apm init and never customised worktrees.dir get the check automatically.
- The idempotency check in ensure_gitignore is a contents.contains(entry) substring match (pre-existing).
  Do not change this in this ticket.
- gitignore_covers_dir uses .trim() + exact equality per line to avoid false positives where one dirname
  is a prefix of another (e.g. "wt" should not match a line containing "wt-old").

### Open questions


### Amendment requests

- [ ] Approach must include the new `ensure_gitignore` function signature explicitly: `pub fn ensure_gitignore(path: &Path, worktree_pattern: Option<&str>, messages: &mut Vec<String>) -> Result<()>`. The current spec says "either pass config in, or have setup() compute the pattern" but never shows the resulting signature. Concrete signature prevents implementer drift.
- [ ] `gitignore_covers_dir` must use exact-line matching (after trimming whitespace and optional leading/trailing `/`), not substring matching. The current loose-match approach would false-positive on lines like `worktree` matching a configured dir of `worktrees`. Spec the exact match algorithm: split file into lines; for each non-comment, non-empty line, trim whitespace, strip optional leading and trailing `/`, then compare equality against the configured dir name (similarly normalized).
- [ ] Add an ordering note: this ticket depends on `50649e84` (verify → validate merge) landing first because the AC references tests that live in `apm-core/tests/verify.rs` today and will move into validate's test surface when 50649e84 lands. Either add `--depends-on 50649e84` or rephrase the test references to reflect post-merge structure.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:54Z | — | new | philippepascal |
| 2026-04-28T19:54Z | new | groomed | philippepascal |
| 2026-04-28T19:54Z | groomed | in_design | philippepascal |
| 2026-04-28T20:02Z | in_design | specd | claude-0428-1954-6858 |
| 2026-04-28T20:17Z | specd | ammend | philippepascal |
| 2026-04-28T20:17Z | ammend | in_design | philippepascal |
