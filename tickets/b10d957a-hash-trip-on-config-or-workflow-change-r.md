+++
id = "b10d957a"
title = "Hash-trip on config or workflow change runs apm validate"
state = "in_progress"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b10d957a-hash-trip-on-config-or-workflow-change-r"
created_at = "2026-04-27T20:28:59.343081Z"
updated_at = "2026-04-27T23:40:44.419068Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["e845127e"]
+++

## Spec

### Problem

When a user modifies `.apm/config.toml` (e.g., switching the completion strategy from `merge` to `pr`) or `.apm/workflow.toml` after tickets with `depends_on` relationships have already been created, existing dependencies can silently become invalid. APM currently has no mechanism to detect this drift: the changed config takes effect immediately, but the tickets that were created under the old rules remain unchanged and unchecked.

The result is that tickets proceed through the workflow carrying stale, invalid dependency configurations. Violations only surface later as confusing failures in branch topology or merge conflicts — not as a clear diagnostic at the moment the configuration changed.

`docs/strategy-and-dependencies.md` (§ 'Hash-trip on config change') specifies the detection mechanism: APM stores a SHA-256 hash of both config files in a local stamp file (`.apm/.validate-stamp`, gitignored). On every `apm` invocation, the live hash is compared to the stored stamp. If they differ, `apm validate` is run automatically in-process. Mutating commands (`apm new`, `apm state`, `apm set`, `apm spec`, `apm start`) are blocked if validation fails; read-only commands (`apm list`, `apm show`, `apm next`) warn but proceed. The stamp is refreshed only after a clean validation pass.

This ticket wires the trigger mechanism. The dependency-rule validation logic itself (`validate_depends_on`, `check_depends_on_rules`) is implemented in ticket e845127e and must land before this ticket is implemented.

### Acceptance criteria

- [x] When `.apm/config.toml` and `.apm/workflow.toml` are unchanged since the last successful hash-trip, `apm` commands run without invoking validation (no extra output, negligible overhead beyond a hash comparison)
- [x] When `.apm/.validate-stamp` is absent, the hash-trip runs on the next invocation; if validation passes, the stamp is created and the command proceeds normally
- [x] When either config file changes and validation passes, the stamp is updated to the new hash and the command proceeds normally with no user-visible output
- [ ] When either config file changes and validation fails, `apm new`, `apm state`, `apm set`, `apm spec`, and `apm start` exit with a non-zero code and a message explaining that mutating commands are blocked until validation passes
- [ ] When validation fails after a config change, `apm list`, `apm show`, `apm next`, and `apm verify` run normally but print a warning to stderr that the config has changed and validation is failing; `apm sync`, `apm work`, and `apm clean` are blocked with a non-zero exit code like other mutating commands
- [ ] `apm validate` is never blocked by the hash-trip gate (it must always be runnable so users can diagnose and fix issues)
- [ ] `apm init` is never blocked by the hash-trip gate (it runs before or during initial config creation)
- [ ] When `apm validate` completes with no issues, it updates the stamp file, clearing any stale hash-trip block on subsequent commands
- [ ] `.apm/.validate-stamp` does not appear in `git status` output (it is gitignored via `.apm/.gitignore`)
- [ ] When `.apm/config.toml` does not exist (not an APM repo), the hash-trip logic is skipped entirely and no stamp file is written

### Out of scope

- Auto-fixing dependency violations — no safe automatic correction exists; requires user intervention
- Enforcing dependency rules at `apm new` or `apm set` write time — ticket a3dc64db
- Implementing `validate_depends_on` and `check_depends_on_rules` — ticket e845127e; this ticket only wires the trigger
- Hash-tripping on changes to ticket files, `agents.md`, or any file other than `config.toml` and `workflow.toml`
- Network-based or CI-triggered re-validation
- A dedicated `apm stamp reset` or `apm stamp clear` command
- Sharing the stamp file across machines or storing it in git — the stamp is intentionally machine-local and gitignored
- Blocking `apm workers`, `apm sessions`, `apm revoke`, `apm version`, `apm register`, `apm show`, `apm list`, `apm next`, or other read-only / administrative commands (they warn but are not blocked)
- Changing the default completion strategy (ticket 941e57fa) or removing the per-epic max_workers override (ticket 6e3f9e91)

### Approach

**1. Add sha2 to apm-core/Cargo.toml**

Under [dependencies], add:

    sha2 = "0.10"

sha2 (RustCrypto) is pure Rust with no C dependencies.

**2. New apm-core/src/hash_stamp.rs**

Expose via pub mod hash_stamp; in apm-core/src/lib.rs.

Three public functions:

pub fn config_hash(root: &Path) -> Result<String>
Reads <root>/.apm/config.toml then <root>/.apm/workflow.toml (in that fixed order). For each file, feeds its raw bytes into a sha2::Sha256 hasher; a missing file contributes zero bytes (not an error). Returns the final digest as a lowercase hex string. The fixed ordering ensures the hash is stable.

pub fn read_stamp(root: &Path) -> Option<String>
Reads <root>/.apm/.validate-stamp. Returns the trimmed content on success, None if absent or unreadable.

pub fn write_stamp(root: &Path, hash: &str) -> Result<()>
Before writing the stamp, ensures <root>/.apm/.gitignore exists and contains the line .validate-stamp (append-only, idempotent). Then writes hash to <root>/.apm/.validate-stamp. This handles both new and existing repos.

Unit tests in #[cfg(test)] mod tests inside hash_stamp.rs:
- hash_is_deterministic: config_hash called twice on the same directory yields identical strings
- hash_changes_on_file_mutation: write a temp config.toml, hash it, change one byte, hash again -> hashes differ
- missing_files_are_stable: config_hash on an empty tempdir succeeds and returns a consistent value across two calls
- stamp_round_trip: write_stamp followed by read_stamp returns the same hash string

**3. New apm/src/hash_trip.rs**

Expose via mod hash_trip; in apm/src/main.rs.

Enum:

    pub enum HashTripOutcome {
        Clean,                          // stamp matched; no action taken
        PassedAndRefreshed,             // hash changed, validate clean, stamp written
        Failed(Vec<(String, String)>),  // hash changed, validate failed; (subject, message) pairs
    }

pub fn is_exempt_command(cmd: &Command) -> bool
Returns true for Command::Validate and Command::Init. Exempt commands skip the hash-trip gate; Validate handles stamp refresh itself (see step 5), and Init runs before a valid config exists.

pub fn is_read_only_command(cmd: &Command) -> bool
Returns true for Command::List, Command::Show, Command::Next, Command::Verify. These warn but are not blocked when validation fails. Everything else (including Command::Sync, Command::Work, Command::Clean, and all mutating commands) is blocked.

pub fn run(root: &Path) -> Result<HashTripOutcome>
1. If <root>/.apm/config.toml is absent -> return Ok(HashTripOutcome::Clean) (not an APM repo).
2. Compute live = apm_core::hash_stamp::config_hash(root)?
3. Read stored = apm_core::hash_stamp::read_stamp(root)
4. If stored matches live -> return Ok(HashTripOutcome::Clean).
5. Load config via apm_core::config::load(root)
6. Load tickets (same pattern as cmd::validate::run uses)
7. Gather issues:
   - For each error from apm_core::validate::validate_config(&config): push ("config".into(), err.to_string())
   - For each (subject, msg) from apm_core::validate::validate_depends_on(&config, &tickets) (ticket e845127e): push as-is
8. If issues is empty -> write_stamp(root, &live)?; return Ok(PassedAndRefreshed)
9. Else -> return Ok(Failed(issues))

Unit tests inside hash_trip.rs:
- validate_is_exempt: is_exempt_command returns true for Validate
- init_is_exempt: is_exempt_command returns true for Init
- list_is_read_only: is_read_only_command returns true for List
- new_is_not_read_only: is_read_only_command returns false for New
- state_is_not_read_only: is_read_only_command returns false for State
- verify_is_read_only: is_read_only_command returns true for Verify

**4. Wire into apm/src/main.rs**

After repo_root() and before match cli.command:

    if !hash_trip::is_exempt_command(&cli.command) {
        match hash_trip::run(&root)? {
            HashTripOutcome::Clean | HashTripOutcome::PassedAndRefreshed => {}
            HashTripOutcome::Failed(issues) => {
                for (subject, msg) in &issues {
                    eprintln!("  {}: {}", subject, msg);
                }
                if hash_trip::is_read_only_command(&cli.command) {
                    eprintln!("warning: config has changed and apm validate is failing.");
                    eprintln!("Run apm validate to see details and fix the issues.");
                } else {
                    eprintln!("error: config has changed and validation is failing.");
                    eprintln!("Mutating commands are blocked. Run apm validate to fix.");
                    std::process::exit(2);
                }
            }
        }
    }

**5. Update apm/src/cmd/validate.rs**

At the end of run(), after confirming all issues vecs are empty, refresh the stamp:

    if config_errors.is_empty() && ticket_issues.is_empty() {
        if let Ok(hash) = apm_core::hash_stamp::config_hash(&root) {
            let _ = apm_core::hash_stamp::write_stamp(&root, &hash);
        }
    }

This ensures apm validate (when it passes) always refreshes the stamp, clearing any hash-trip block on subsequent commands.

**Order of implementation:**
1. apm-core/Cargo.toml -- add sha2
2. apm-core/src/hash_stamp.rs + tests + lib.rs registration
3. apm/src/hash_trip.rs + tests
4. Wire main.rs
5. Update cmd/validate.rs

Ticket e845127e must be merged before this ticket is implemented (step 3 calls validate_depends_on).

### Open questions


### Amendment requests

- [x] Add `Verify` to `is_read_only_command` so `apm verify` runs (with the warning) when config has changed and validation is failing. The "ok to run when config is broken" set is: `list`, `show`, `next`, `verify`, plus `validate` (already exempt). Everything else is blocked.
- [x] Update the AC that names the read-only/warn commands: replace the current list with `apm list`, `apm show`, `apm next`, `apm verify` warn-but-run; explicitly note that `apm sync`, `apm work`, `apm clean` are blocked.
- [x] Add a unit test for `is_read_only_command(Command::Verify) == true`.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:44Z | new | groomed | philippepascal |
| 2026-04-27T21:24Z | groomed | in_design | philippepascal |
| 2026-04-27T21:32Z | in_design | specd | claude-0427-2124-9ed0 |
| 2026-04-27T22:11Z | specd | ammend | philippepascal |
| 2026-04-27T22:25Z | ammend | in_design | philippepascal |
| 2026-04-27T22:28Z | in_design | specd | claude-0427-2225-e100 |
| 2026-04-27T22:55Z | specd | ready | philippepascal |
| 2026-04-27T23:40Z | ready | in_progress | philippepascal |