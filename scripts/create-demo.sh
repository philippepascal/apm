#!/usr/bin/env bash
# scripts/create-demo.sh — create an apm-demo GitHub repository under your account.
#
# Usage:  bash scripts/create-demo.sh
#
# Steps:
#   0. Preflight checks
#   1. Create a temp working directory
#   2. Create & clone the GitHub repo
#   3. Write the Cargo project (Cargo.toml + src/main.rs)
#   4. Write .apm/config.toml and run apm init
#   5. Push main to GitHub (required before apm epic new)
#   6. Create the "Search feature" epic
#   7. Create 14 tickets, fill their specs, transition to target states
#   8. Write README.md
#   9. Final commit + push all branches
#
# Running this script is the post-merge manual step.  The script itself is
# the deliverable committed to the APM repo.

set -euo pipefail

GH_USER=$(gh api user --jq .login 2>/dev/null) || { echo "ERROR: cannot determine GitHub username"; exit 1; }
REPO="${GH_USER}/apm-demo"
REPO_URL="https://github.com/${REPO}.git"

# ─── 0. Preflight checks ──────────────────────────────────────────────────────

echo "==> Preflight checks"
command -v gh    >/dev/null 2>&1 || { echo "ERROR: gh not found — install GitHub CLI"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found — install Rust toolchain"; exit 1; }
command -v apm   >/dev/null 2>&1 || { echo "ERROR: apm not found — install apm"; exit 1; }
gh auth status   >/dev/null 2>&1 || { echo "ERROR: not authenticated — run: gh auth login"; exit 1; }

# ─── 1. Temp working directory ───────────────────────────────────────────────

WORKDIR=$(mktemp -d)
echo "Working directory: $WORKDIR"
# Worktrees land at $WORKDIR/jot--worktrees (inside WORKDIR) and are cleaned up
# automatically when the script exits.
trap 'echo "Cleaning up $WORKDIR"; rm -rf "$WORKDIR"' EXIT

# ─── 2. Create & clone GitHub repo ───────────────────────────────────────────

echo ""
echo "==> Creating GitHub repository ${REPO}"
gh repo create "${REPO}" \
    --public \
    --description "APM demo — explore apm with a realistic Rust CLI project" \
    2>/dev/null || echo "    (repo may already exist, continuing)"

echo "==> Cloning ${REPO_URL}"
for i in 1 2 3 4 5; do
    git clone "${REPO_URL}" "$WORKDIR/apm-demo" 2>/dev/null && break
    echo "    clone attempt $i failed, retrying in 3s..."
    sleep 3
done
[ -d "$WORKDIR/apm-demo" ] || { echo "ERROR: clone failed after 5 attempts"; exit 1; }
DEMO="$WORKDIR/apm-demo"

# All subsequent commands run from inside the demo repo.
cd "$DEMO"

# ─── 3. Write the Cargo project ───────────────────────────────────────────────

echo ""
echo "==> Writing Cargo project (jot)"
mkdir -p src

cat > Cargo.toml << 'CARGO_TOML'
[package]
name = "jot"
version = "0.1.0"
edition = "2021"
description = "A minimal command-line notes tool"

[[bin]]
name = "jot"
path = "src/main.rs"
CARGO_TOML

cat > src/main.rs << 'MAIN_RS'
//! jot — a minimal command-line notes tool.
//!
//! Implemented commands:
//!   jot add "<text>"    append a note to ~/.jot/notes.txt
//!   jot list            print all notes with indices
//!
//! Stubbed commands (in progress):
//!   jot delete <n>      remove note by 1-based index
//!   jot search <query>  full-text search across notes

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

fn notes_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".jot").join("notes.txt")
}

fn cmd_add(text: &str) -> io::Result<()> {
    let path = notes_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{text}")?;
    println!("Note added.");
    Ok(())
}

fn cmd_list() -> io::Result<()> {
    let path = notes_path();
    if !path.exists() {
        println!("(no notes yet — use `jot add \"<text>\"` to add one)");
        return Ok(());
    }
    let file = fs::File::open(&path)?;
    let reader = io::BufReader::new(file);
    let mut count = 0usize;
    for (i, line) in reader.lines().enumerate() {
        println!("{:>4}  {}", i + 1, line?);
        count += 1;
    }
    if count == 0 {
        println!("(no notes yet — use `jot add \"<text>\"` to add one)");
    }
    Ok(())
}

fn cmd_delete(arg: &str) -> io::Result<()> {
    let _ = arg.parse::<usize>().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "index must be a positive integer")
    })?;
    println!("jot delete: not yet implemented");
    Ok(())
}

fn cmd_search(_query: &str) -> io::Result<()> {
    unimplemented!("full-text search is not yet implemented")
}

fn usage() {
    eprintln!("jot — a minimal command-line notes tool");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  jot add \"<text>\"   append a note");
    eprintln!("  jot list            list all notes");
    eprintln!("  jot delete <n>      delete note by index  (not yet implemented)");
    eprintln!("  jot search <query>  search notes          (not yet implemented)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("add") => {
            if args.len() < 3 {
                eprintln!("Usage: jot add \"<text>\"");
                std::process::exit(1);
            }
            cmd_add(&args[2..].join(" "))
        }
        Some("list") => cmd_list(),
        Some("delete") => {
            if args.len() < 3 {
                eprintln!("Usage: jot delete <n>");
                std::process::exit(1);
            }
            cmd_delete(&args[2])
        }
        Some("search") => {
            if args.len() < 3 {
                eprintln!("Usage: jot search <query>");
                std::process::exit(1);
            }
            cmd_search(&args[2])
        }
        _ => {
            usage();
            return;
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
MAIN_RS

# ─── 4. Write .apm/config.toml and run apm init ──────────────────────────────

echo ""
echo "==> Initialising APM"
mkdir -p .apm

cat > .apm/config.toml << 'APM_CONFIG'
[project]
name = "jot"
description = "A minimal CLI notes tool"
default_branch = "main"
collaborators = []

[tickets]
dir = "tickets"

[worktrees]
dir = "../jot--worktrees"
agent_dirs = [".claude", ".cursor", ".windsurf"]

[agents]
max_concurrent = 3
instructions = ".apm/agents.md"

[workers]
command = "claude"
args = ["--print"]

[logging]
enabled = false
file = "~/.local/state/apm/jot.log"
APM_CONFIG

# apm init reads the config above (non-interactive — no TTY in script context).
# It creates workflow.toml, ticket.toml, agents.md, CLAUDE.md, .gitignore,
# tickets/ directory, and an initial commit on main.
apm init --no-claude

# ─── 5. Push main to GitHub (required before apm epic new) ───────────────────

echo ""
echo "==> Pushing main branch"
git push origin main

# ─── 6. Create epic ───────────────────────────────────────────────────────────

echo ""
echo "==> Creating epic: Search feature"
EPIC_BRANCH=$(apm epic new 'Search feature')
# apm epic new prints the branch name, e.g. "epic/ab12cd34-search-feature"
EPIC_ID=$(echo "$EPIC_BRANCH" | sed 's|epic/\([0-9a-f]*\)-.*|\1|')
echo "    Epic branch: $EPIC_BRANCH  (id: $EPIC_ID)"

echo ""
echo "==> Creating epic: Multi-notebook support"
EPIC2_BRANCH=$(apm epic new 'Multi-notebook support')
EPIC2_ID=$(echo "$EPIC2_BRANCH" | sed 's|epic/\([0-9a-f]*\)-.*|\1|')
echo "    EPIC2=$EPIC2_ID"

# ─── 7. Create tickets ────────────────────────────────────────────────────────
# Helper: extract ticket ID from "apm new" output.
# Output format: "Created ticket <id>: <filename> (branch: <branch>)"
extract_id() { echo "$1" | awk '{print $3}' | tr -d ':'; }

echo ""
echo "==> Creating 35 tickets"

# ── Ticket 1: Initial CLI scaffold → closed ───────────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Initial CLI scaffold')
T1=$(extract_id "$out")
echo "    T1=$T1  (Initial CLI scaffold)"

apm spec "$T1" --no-aggressive --section 'Problem' --set \
  'Set up the basic jot CLI binary with argument parsing and command dispatch.
This is the foundation every future command builds on.'

apm spec "$T1" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] cargo build succeeds without warnings
- [x] running jot with no arguments prints usage text to stderr
- [x] command dispatch skeleton routes add/list/delete/search subcommands
EOF

apm spec "$T1" --no-aggressive --section 'Out of scope' --set \
  'Any actual command logic — all subcommands may be stubs at this stage.'

apm spec "$T1" --no-aggressive --section 'Approach' --set \
  'Use std::env::args() to collect argv.  Match the first arg as the subcommand.
Print usage to stderr for unknown commands.  Each subcommand function is defined
but may initially just println! a placeholder.'

apm spec "$T1" --no-aggressive --section 'Code review' --set - << 'EOF'
- [x] binary builds cleanly on stable Rust
- [x] no unsafe, no deps outside std
EOF

apm state "$T1" --no-aggressive --force closed
echo "    T1 → closed"

# ── Ticket 2: Add note to file (jot add) → closed ─────────────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T1" 'Add note to file (jot add)')
T2=$(extract_id "$out")
echo "    T2=$T2  (Add note to file)"

apm spec "$T2" --no-aggressive --section 'Problem' --set \
  'Users need to add notes from the command line.  jot add should append a line
to ~/.jot/notes.txt and create the file and directory on first use.'

apm spec "$T2" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot add "some text" appends the text as a new line to ~/.jot/notes.txt
- [x] if ~/.jot/ does not exist, it is created automatically
- [x] if notes.txt does not exist, it is created automatically
- [x] consecutive calls append multiple lines in order
- [x] jot add prints "Note added." to stdout on success
EOF

apm spec "$T2" --no-aggressive --section 'Out of scope' --set \
  'Note editing, tagging, or any metadata beyond the raw text line.'

apm spec "$T2" --no-aggressive --section 'Approach' --set \
  'Open notes.txt with OpenOptions::create(true).append(true).  Use
fs::create_dir_all on the parent directory before opening.  Write the
text followed by a newline.'

apm spec "$T2" --no-aggressive --section 'Code review' --set - << 'EOF'
- [x] directory creation happens before file open — no race condition on first use
- [x] error path returns Err and main prints it cleanly
EOF

apm state "$T2" --no-aggressive --force closed
echo "    T2 → closed"

# ── Ticket 3: List notes command (jot list) → implemented ─────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T2" 'List notes command (jot list)')
T3=$(extract_id "$out")
echo "    T3=$T3  (List notes command)"

apm spec "$T3" --no-aggressive --section 'Problem' --set \
  'Users need a way to view all their notes.  jot list should print every line
from notes.txt with a 1-based index so notes can be referenced by number.'

apm spec "$T3" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot list prints each note on its own line, prefixed by a right-aligned 1-based index
- [x] when notes.txt does not exist, prints a friendly "no notes yet" message
- [x] when notes.txt is empty, prints the same friendly message
- [x] output is sent to stdout
EOF

apm spec "$T3" --no-aggressive --section 'Out of scope' --set \
  'Sorting, filtering, truncating long lines, or pagination (separate ticket).'

apm spec "$T3" --no-aggressive --section 'Approach' --set \
  'Open notes.txt with BufReader.  Enumerate lines starting at 1.  Print each
with format!("{:>4}  {}", i, line).  If the file is absent or empty, print
the friendly "no notes yet" message.'

apm state "$T3" --no-aggressive --force implemented
echo "    T3 → implemented"

# ── Ticket 4: Delete note command (jot delete) → in_progress ──────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" 'Delete note command (jot delete)')
T4=$(extract_id "$out")
echo "    T4=$T4  (Delete note command)"
apm set "$T4" priority 5 --no-aggressive

apm spec "$T4" --no-aggressive --section 'Problem' --set \
  'Users need to remove individual notes.  jot delete N should remove the note
at 1-based index N, renumbering the remaining notes implicitly.'

apm spec "$T4" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot delete N removes the Nth note (1-based)
- [ ] remaining notes keep their relative order
- [ ] jot delete with an out-of-range index prints a clear error and exits non-zero
- [ ] jot delete with a non-integer argument prints a clear error and exits non-zero
EOF

apm spec "$T4" --no-aggressive --section 'Out of scope' --set \
  'Undo, bulk delete, or deleting by content match.'

apm spec "$T4" --no-aggressive --section 'Approach' --set \
  'Read all lines into a Vec<String>.  Validate the index.  Remove the element.
Rewrite the file atomically via a temp file + rename to avoid corruption on crash.'

apm state "$T4" --no-aggressive --force in_progress
echo "    T4 → in_progress"

# ── Ticket 5: Add full-text search → in_progress (epic) ───────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" --epic "$EPIC_ID" \
  'Add full-text search')
T5=$(extract_id "$out")
echo "    T5=$T5  (Add full-text search)"
apm set "$T5" priority 5 --no-aggressive

apm spec "$T5" --no-aggressive --section 'Problem' --set \
  'Users need to find notes by keyword.  jot search <query> should scan notes.txt
for lines containing the query string and print matching lines with their indices.'

apm spec "$T5" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot search <query> prints matching notes with their original 1-based indices
- [ ] search is case-sensitive by default
- [ ] no matches produces a "no notes match" message and exits 0
- [ ] empty query string prints an error and exits non-zero
EOF

apm spec "$T5" --no-aggressive --section 'Out of scope' --set \
  'Case-insensitive or fuzzy matching — see the fuzzy search fallback ticket.'

apm spec "$T5" --no-aggressive --section 'Approach' --set \
  'Read notes.txt with BufReader.  Filter lines using str::contains.  Print
matching lines with their original 1-based index.  Exit 0 in all non-error cases.'

apm state "$T5" --no-aggressive --force in_progress
echo "    T5 → in_progress"

# ── Ticket 6: Search result highlighting → ready (epic) ───────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T5" --epic "$EPIC_ID" \
  'Search result highlighting')
T6=$(extract_id "$out")
echo "    T6=$T6  (Search result highlighting)"

apm spec "$T6" --no-aggressive --section 'Problem' --set \
  'Plain search output makes it hard to see why a note matched.  Matching text
in result lines should be highlighted using ANSI bold so users can quickly scan.'

apm spec "$T6" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] matched query text is wrapped in ANSI bold escape sequences in the output
- [ ] when stdout is not a TTY, highlighting is suppressed (raw text only)
- [ ] highlighting does not alter the matched text content itself
EOF

apm spec "$T6" --no-aggressive --section 'Out of scope' --set \
  'Colour theming, configurable highlight style, or regex patterns.'

apm spec "$T6" --no-aggressive --section 'Approach' --set \
  'Check std::io::IsTerminal on stdout.  If a TTY, wrap each match with the ANSI
bold escape sequence.  Use str::split on the query and reassemble to avoid altering
non-matched characters.'

apm state "$T6" --no-aggressive --force ready
echo "    T6 → ready"

# ── Ticket 7: Export notes to markdown → specd ────────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Export notes to markdown')
T7=$(extract_id "$out")
echo "    T7=$T7  (Export notes to markdown)"

apm spec "$T7" --no-aggressive --section 'Problem' --set \
  'Users want to share or archive their notes outside of jot.  An export command
should write all notes as a Markdown list to a file in the current directory.'

apm spec "$T7" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot export writes all notes to ./jot-export.md in the current directory
- [ ] each note is rendered as a Markdown list item (starts with "- ")
- [ ] existing jot-export.md is overwritten without prompting
- [ ] jot export prints "Exported N notes to jot-export.md" on success
- [ ] when notes.txt does not exist, exports an empty file and prints 0 notes
EOF

apm spec "$T7" --no-aggressive --section 'Out of scope' --set \
  'Custom output paths, multiple export formats, or incremental exports.'

apm spec "$T7" --no-aggressive --section 'Approach' --set \
  'Read notes.txt line by line.  Build a String with each line prefixed by "- ".
Write the result to jot-export.md in the current working directory using fs::write
(which overwrites automatically).'

apm state "$T7" --no-aggressive --force specd
echo "    T7 → specd"

# ── Ticket 8: Note tagging support → in_design ────────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Note tagging support')
T8=$(extract_id "$out")
echo "    T8=$T8  (Note tagging support)"

apm spec "$T8" --no-aggressive --section 'Problem' --set \
  'Users want to organise notes with inline hashtag-style tags.  jot add should
parse tags from note text and jot list --tag should filter by them.'

apm spec "$T8" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot add "buy milk #shopping" stores the note with the tag "shopping" recorded
- [ ] jot list --tag shopping shows only notes carrying the #shopping tag
- [ ] tags are case-insensitive when filtering
EOF

apm state "$T8" --no-aggressive --force in_design
echo "    T8 → in_design  (worktree created in $WORKDIR/jot--worktrees/)"

# ── Ticket 9: Configuration file support → groomed ────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Configuration file support')
T9=$(extract_id "$out")
echo "    T9=$T9  (Configuration file support)"

apm spec "$T9" --no-aggressive --section 'Problem' --set \
  'jot hardcodes ~/.jot/notes.txt.  Power users need to configure an alternative
notes-file path and other preferences via a TOML config at ~/.jot/config.toml.'

apm state "$T9" --no-aggressive --force groomed
echo "    T9 → groomed"

# ── Ticket 10: Pagination for long note lists → new (stays new) ───────────────
out=$(apm new --no-edit --no-aggressive 'Pagination for long note lists')
T10=$(extract_id "$out")
echo "    T10=$T10  (Pagination — stays new)"

# ── Ticket 11: Interactive TUI mode → question ────────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Interactive TUI mode')
T11=$(extract_id "$out")
echo "    T11=$T11  (Interactive TUI mode)"

apm spec "$T11" --no-aggressive --section 'Problem' --set \
  'Keyboard-driven users want a curses-style interactive interface for browsing,
adding, and deleting notes without re-invoking the binary for each operation.'

apm spec "$T11" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot tui opens a full-screen interface showing the note list
- [ ] pressing 'a' opens a prompt to append a note
- [ ] pressing 'd' deletes the currently selected note
- [ ] pressing 'q' or Ctrl-C exits cleanly and restores the terminal
EOF

apm spec "$T11" --no-aggressive --section 'Open questions' --set \
  'Which TUI framework should we adopt — ratatui or cursive? ratatui is more actively
maintained but cursive has a simpler widget API. Any licensing concerns with either?'

apm state "$T11" --no-aggressive --force question
echo "    T11 → question"

# ── Ticket 12: Fuzzy search fallback → ammend (epic) ──────────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T5" --epic "$EPIC_ID" \
  'Fuzzy search fallback')
T12=$(extract_id "$out")
echo "    T12=$T12  (Fuzzy search fallback)"

apm spec "$T12" --no-aggressive --section 'Problem' --set \
  'The current full-text search is case-sensitive, which surprises users who type
"TODO" when their note says "todo". A fallback should activate when an exact
match returns no results.'

apm spec "$T12" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] when an exact search returns zero results, a case-insensitive retry runs automatically
- [ ] results from the fallback are prefixed with "(fuzzy)" in the output
- [ ] the fallback can be suppressed with the --exact flag
EOF

# Transition to ammend (apm auto-inserts a placeholder Amendment requests section)
apm state "$T12" --no-aggressive --force ammend

# Replace the auto-inserted placeholder with a real unchecked checkbox
apm spec "$T12" --no-aggressive --section 'Amendment requests' --set - << 'EOF'
- [ ] Reconsider the automatic-fallback UX — supervisor prefers an explicit --fuzzy flag rather than a silent automatic retry. Update Acceptance criteria and Approach to use the flag-based design.
EOF

echo "    T12 → ammend"

# ── Ticket 13: Fix list command index off-by-one → blocked ────────────────────
out=$(apm new --no-edit --no-aggressive 'Fix list command index off-by-one')
T13=$(extract_id "$out")
echo "    T13=$T13  (Fix list index)"

apm spec "$T13" --no-aggressive --section 'Problem' --set \
  'jot list uses 0-based indices in some builds, but users expect 1-based numbering
consistent with cat -n and nl.  This causes confusing behaviour when jot delete 1
removes the wrong note.'

apm spec "$T13" --no-aggressive --section 'Approach' --set \
  'Change the loop in cmd_list to start at 1: use enumerate().map(|(i, l)| (i+1, l)).
Verify cmd_delete uses the same 1-based convention.  Update tests accordingly.'

apm spec "$T13" --no-aggressive --section 'Open questions' --set \
  'Should jot list also print a column header ("  #  Note") or keep the bare numeric
prefix? Supervisor decision needed before implementation can begin.'

apm state "$T13" --no-aggressive --force blocked
echo "    T13 → blocked"

# ── Ticket 14: Add --count flag to jot list → ready ───────────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" 'Add --count flag to jot list')
T14=$(extract_id "$out")
echo "    T14=$T14  (Add --count flag)"
apm set "$T14" priority 3 --no-aggressive

apm spec "$T14" --no-aggressive --section 'Problem' --set \
  'Users want a quick way to see how many notes they have without scrolling through
a long list.  A --count flag should print just the note count and exit.'

apm spec "$T14" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot list --count prints a single line: "N notes" (or "1 note" for N=1)
- [ ] jot list --count exits 0 even when notes.txt does not exist (prints "0 notes")
- [ ] --count is mutually exclusive with other list modifier flags
EOF

apm spec "$T14" --no-aggressive --section 'Out of scope' --set \
  'Counting filtered subsets (e.g. --count --tag work) — that can be combined later.'

apm spec "$T14" --no-aggressive --section 'Approach' --set \
  'Parse argv for --count before iterating notes.  If set, count lines and print the
count string, then exit.  Reuse notes_path() for consistency with cmd_list.'

apm state "$T14" --no-aggressive --force ready
echo "    T14 → ready"

# ── Epic 2: Multi-notebook support — TE1 through TE7 ─────────────────────────

# ── Epic ticket TE1: Create a named notebook → closed ─────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" 'Create a named notebook')
TE1=$(extract_id "$out")
echo "    TE1=$TE1  (Create a named notebook)"

apm spec "$TE1" --no-aggressive --section 'Problem' --set \
  'Users organising notes into projects need dedicated notebooks.
jot notebook create <name> should create a new named notebook stored under
~/.jot/notebooks/<name>/notes.txt.'

apm spec "$TE1" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot notebook create <name> creates ~/.jot/notebooks/<name>/notes.txt
- [x] creating a notebook that already exists prints an error and exits non-zero
- [x] notebook names are restricted to alphanumeric characters, hyphens, and underscores
- [x] jot notebook list shows the new notebook in its output
EOF

apm spec "$TE1" --no-aggressive --section 'Out of scope' --set \
  'Nested notebooks, importing existing note files into a new notebook.'

apm spec "$TE1" --no-aggressive --section 'Approach' --set \
  'Parse the notebook create <name> subcommand.  Validate the name against a
regex ([a-zA-Z0-9_-]+).  Call fs::create_dir_all on the notebook path; if the
directory already exists, return an error.'

apm spec "$TE1" --no-aggressive --section 'Code review' --set - << 'EOF'
- [x] name validation rejects paths with slashes — no directory traversal
- [x] create_dir_all error surfaces cleanly through the main error handler
EOF

apm state "$TE1" --no-aggressive --force closed
echo "    TE1 → closed"

# ── Epic ticket TE2: Switch active notebook → closed ──────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" 'Switch active notebook')
TE2=$(extract_id "$out")
echo "    TE2=$TE2  (Switch active notebook)"

apm spec "$TE2" --no-aggressive --section 'Problem' --set \
  'Once multiple notebooks exist, users need to set which one is active.
The active notebook determines where jot add writes and jot list reads without
requiring an explicit --notebook flag on every command.'

apm spec "$TE2" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot notebook use <name> updates ~/.jot/active to contain <name>
- [x] switching to a non-existent notebook prints an error and exits non-zero
- [x] after switching, jot add and jot list operate on the new notebook
- [x] the active notebook persists across invocations
EOF

apm spec "$TE2" --no-aggressive --section 'Out of scope' --set \
  'A --notebook per-command override flag — that is a separate ticket.'

apm spec "$TE2" --no-aggressive --section 'Approach' --set \
  'Write the notebook name to ~/.jot/active as a plain text file.
Update notes_path() to read this file (falling back to the default notebook
if absent) to resolve the current notes file path.'

apm spec "$TE2" --no-aggressive --section 'Code review' --set - << 'EOF'
- [x] fall-back to default when ~/.jot/active is absent — no error on first run
- [x] non-existent notebook check happens before the write
EOF

apm state "$TE2" --no-aggressive --force closed
echo "    TE2 → closed"

# ── Epic ticket TE3: List all notebooks → implemented ─────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" \
  --depends-on "$TE1" --depends-on "$TE2" 'List all notebooks')
TE3=$(extract_id "$out")
echo "    TE3=$TE3  (List all notebooks)"

apm spec "$TE3" --no-aggressive --section 'Problem' --set \
  'Users need to see all their notebooks at a glance and know which one is
currently active before running notebook-scoped commands.'

apm spec "$TE3" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot notebook list prints each notebook name on its own line
- [x] the active notebook is marked with a * prefix
- [x] when no extra notebooks exist, the default notebook is shown
- [x] output is sent to stdout
EOF

apm spec "$TE3" --no-aggressive --section 'Out of scope' --set \
  'Note counts per notebook, sorting options, or colour output.'

apm spec "$TE3" --no-aggressive --section 'Approach' --set \
  'Read ~/.jot/notebooks/ directory entries.  Read ~/.jot/active for the
current notebook name.  Print each notebook, prepending "* " for the active one.'

apm state "$TE3" --no-aggressive --force implemented
echo "    TE3 → implemented"

# ── Epic ticket TE4: Delete a notebook → ready ────────────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" \
  --depends-on "$TE1" --depends-on "$TE2" 'Delete a notebook')
TE4=$(extract_id "$out")
echo "    TE4=$TE4  (Delete a notebook)"

apm spec "$TE4" --no-aggressive --section 'Problem' --set \
  'Users want to remove notebooks they no longer need, including all notes
inside them.  There is currently no command to do this safely.'

apm spec "$TE4" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot notebook delete <name> removes ~/.jot/notebooks/<name>/ and all its contents
- [ ] deleting the active notebook resets the active pointer to the default
- [ ] attempting to delete a non-existent notebook prints an error and exits non-zero
- [ ] a --confirm flag is required to prevent accidental deletion
EOF

apm spec "$TE4" --no-aggressive --section 'Out of scope' --set \
  'Archiving notes before deletion, soft-delete or trash functionality.'

apm spec "$TE4" --no-aggressive --section 'Approach' --set \
  'Verify the notebook exists.  If it is active, write "default" to
~/.jot/active.  Require --confirm flag; if absent print a warning and exit
non-zero.  Use fs::remove_dir_all to delete the notebook directory.'

apm state "$TE4" --no-aggressive --force ready
echo "    TE4 → ready"

# ── Epic ticket TE5: Rename a notebook → specd ────────────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" \
  --depends-on "$TE1" --depends-on "$TE2" 'Rename a notebook')
TE5=$(extract_id "$out")
echo "    TE5=$TE5  (Rename a notebook)"

apm spec "$TE5" --no-aggressive --section 'Problem' --set \
  'Notebook names are fixed after creation, making it impossible to correct
typos or reflect project renames without recreating the notebook from scratch.'

apm spec "$TE5" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot notebook rename <old> <new> renames the notebook directory
- [ ] if the new name already exists, prints an error and exits non-zero
- [ ] if the renamed notebook was active, the active pointer is updated to the new name
- [ ] the new name must pass the same validation rules as notebook creation
EOF

apm spec "$TE5" --no-aggressive --section 'Out of scope' --set \
  'Merging notebooks during rename, renaming the default notebook.'

apm spec "$TE5" --no-aggressive --section 'Approach' --set \
  'Validate both names.  Check that <old> exists and <new> does not.  Use
fs::rename on the notebook directory.  If the active pointer matches <old>,
rewrite ~/.jot/active with <new>.'

apm state "$TE5" --no-aggressive --force specd
echo "    TE5 → specd"

# ── Epic ticket TE6: Move note between notebooks → in_design ──────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" \
  --depends-on "$TE3" 'Move note between notebooks')
TE6=$(extract_id "$out")
echo "    TE6=$TE6  (Move note between notebooks)"

apm spec "$TE6" --no-aggressive --section 'Problem' --set \
  'Notes end up in the wrong notebook and there is no way to relocate them
without manually editing files.  A move command would let users reorganise
their notes across notebooks.'

apm spec "$TE6" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot note move <index> --to <notebook> moves the note at <index> in the active notebook into <notebook>
- [ ] the moved note is appended to the target notebook
EOF

apm state "$TE6" --no-aggressive --force in_design
echo "    TE6 → in_design"

# ── Epic ticket TE7: Merge two notebooks → new ────────────────────────────────
out=$(apm new --no-edit --no-aggressive --epic "$EPIC2_ID" \
  --depends-on "$TE6" 'Merge two notebooks')
TE7=$(extract_id "$out")
echo "    TE7=$TE7  (Merge two notebooks — stays new)"

# ── Standalone tickets TS1–TS14 ───────────────────────────────────────────────

# ── Standalone TS1: Add --version flag → closed ───────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Add --version flag')
TS1=$(extract_id "$out")
echo "    TS1=$TS1  (Add --version flag)"

apm spec "$TS1" --no-aggressive --section 'Problem' --set \
  'Users cannot quickly check which version of jot is installed.  The binary
should print its version when invoked with --version or -V.'

apm spec "$TS1" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot --version prints "jot <version>" and exits 0
- [x] -V is accepted as a short alias
- [x] the version string matches the version field in Cargo.toml
EOF

apm spec "$TS1" --no-aggressive --section 'Out of scope' --set \
  '--long-version with build metadata or git hash.'

apm spec "$TS1" --no-aggressive --section 'Approach' --set \
  'Add a top-level match arm for "--version" and "-V" before the subcommand
dispatch.  Print format!("jot {}", env!("CARGO_PKG_VERSION")) and return.'

apm spec "$TS1" --no-aggressive --section 'Code review' --set - << 'EOF'
- [x] env!("CARGO_PKG_VERSION") is evaluated at compile time — no runtime overhead
- [x] exits 0 via the normal return path rather than std::process::exit
EOF

apm state "$TS1" --no-aggressive --force closed
echo "    TS1 → closed"

# ── Standalone TS2: Colorize list output → implemented ────────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" 'Colorize list output')
TS2=$(extract_id "$out")
echo "    TS2=$TS2  (Colorize list output)"

apm spec "$TS2" --no-aggressive --section 'Problem' --set \
  'jot list output is plain text, making it hard to scan long note lists.
Applying colour to the index numbers would improve readability at a glance.'

apm spec "$TS2" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] jot list renders index numbers in bold cyan ANSI colour when stdout is a TTY
- [x] colour is suppressed when stdout is not a TTY (piped or redirected)
- [x] colour does not appear in the note text itself
EOF

apm spec "$TS2" --no-aggressive --section 'Out of scope' --set \
  'Configurable colours, highlighting note content, dark/light theme detection.'

apm spec "$TS2" --no-aggressive --section 'Approach' --set \
  'Check std::io::IsTerminal on stdout.  If a TTY, wrap each index with ANSI
escape codes \x1b[1;36m (bold cyan) and \x1b[0m (reset).  Otherwise print
the plain format unchanged.'

apm state "$TS2" --no-aggressive --force implemented
echo "    TS2 → implemented"

# ── Standalone TS3: Record timestamp on note creation → implemented ───────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T2" \
  'Record timestamp on note creation')
TS3=$(extract_id "$out")
echo "    TS3=$TS3  (Record timestamp on note creation)"

apm spec "$TS3" --no-aggressive --section 'Problem' --set \
  'Users cannot tell when a note was written, making it hard to distinguish
old reminders from recent ones.  Each note should carry an ISO-8601 timestamp.'

apm spec "$TS3" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [x] each note stored in notes.txt is prefixed with an ISO-8601 timestamp [YYYY-MM-DD HH:MM]
- [x] jot list displays the timestamp prefix before the note text
- [x] jot add "text" still prints "Note added." — the timestamp is not shown at add time
EOF

apm spec "$TS3" --no-aggressive --section 'Out of scope' --set \
  'Editing or removing timestamps, timezone configuration, timestamp filtering.'

apm spec "$TS3" --no-aggressive --section 'Approach' --set \
  'In cmd_add, format the current UTC time from std::time::SystemTime as
[YYYY-MM-DD HH:MM] and prepend it to the note text before writing to disk.'

apm state "$TS3" --no-aggressive --force implemented
echo "    TS3 → implemented"

# ── Standalone TS4: Edit a note in-place (jot edit N) → specd ────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" \
  'Edit a note in-place (jot edit N)')
TS4=$(extract_id "$out")
echo "    TS4=$TS4  (Edit a note in-place)"

apm spec "$TS4" --no-aggressive --section 'Problem' --set \
  'Fixing a typo in an existing note requires deleting it and re-adding it.
An edit command would let users amend note text in-place by 1-based index.'

apm spec "$TS4" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot edit N <new text> replaces the text of the note at 1-based index N
- [ ] index out of range prints an error and exits non-zero
- [ ] the edited note retains its original position in the list
- [ ] the timestamp prefix (if present) is preserved on the existing note
EOF

apm spec "$TS4" --no-aggressive --section 'Out of scope' --set \
  'Opening an external $EDITOR, multi-line note editing, editing by content match.'

apm spec "$TS4" --no-aggressive --section 'Approach' --set \
  'Read all lines into Vec<String>.  Validate the index.  Replace the text
portion of the element at that index.  Rewrite the file atomically via a
temp file + rename to avoid corruption on crash.'

apm state "$TS4" --no-aggressive --force specd
echo "    TS4 → specd"

# ── Standalone TS5: Clear all notes (jot clear) → ready ──────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" 'Clear all notes (jot clear)')
TS5=$(extract_id "$out")
echo "    TS5=$TS5  (Clear all notes)"

apm spec "$TS5" --no-aggressive --section 'Problem' --set \
  'Starting fresh requires manually deleting ~/.jot/notes.txt.  A jot clear
command would truncate the notes file safely with a confirmation step.'

apm spec "$TS5" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot clear removes all notes from the active notebook
- [ ] the command requires a --confirm flag to proceed, to prevent accidents
- [ ] after clearing, jot list prints the "no notes yet" message
- [ ] jot clear --confirm exits 0 on success
EOF

apm spec "$TS5" --no-aggressive --section 'Out of scope' --set \
  'Selective clearing by tag or age, soft-delete or archive-before-clear.'

apm spec "$TS5" --no-aggressive --section 'Approach' --set \
  'Match the "clear" subcommand.  Require --confirm in argv; if absent, print
a warning and exit non-zero.  Truncate the file with
OpenOptions::create(true).write(true).truncate(true).'

apm state "$TS5" --no-aggressive --force ready
echo "    TS5 → ready"

# ── Standalone TS6: Word count and stats (jot stats) → in_design ─────────────
out=$(apm new --no-edit --no-aggressive 'Word count and stats (jot stats)')
TS6=$(extract_id "$out")
echo "    TS6=$TS6  (Word count and stats)"

apm spec "$TS6" --no-aggressive --section 'Problem' --set \
  'Users managing a large note collection have no way to see aggregate
statistics such as total note count, word count, or average note length.'

apm spec "$TS6" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot stats prints total note count, total word count, and average note length
- [ ] output is human-readable, e.g. "Notes: 42  Words: 387  Avg: 9.2 words/note"
EOF

apm state "$TS6" --no-aggressive --force in_design
echo "    TS6 → in_design"

# ── Standalone TS7: Deduplicate notes → groomed ──────────────────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" 'Deduplicate notes')
TS7=$(extract_id "$out")
echo "    TS7=$TS7  (Deduplicate notes)"

apm spec "$TS7" --no-aggressive --section 'Problem' --set \
  'After extensive use, notes.txt can accumulate identical lines from repeated
jot add calls.  A deduplication command should remove exact duplicates while
preserving note order.'

apm spec "$TS7" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot dedup removes all but the first occurrence of any exact duplicate line
- [ ] relative order of unique notes is preserved
EOF

apm state "$TS7" --no-aggressive --force groomed
echo "    TS7 → groomed"

# ── Standalone TS8: Pin a note to the top of jot list → new ──────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" \
  'Pin a note to the top of jot list')
TS8=$(extract_id "$out")
echo "    TS8=$TS8  (Pin a note — stays new)"

# ── Standalone TS9: Copy note to clipboard (jot copy N) → new ────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" \
  'Copy note to clipboard (jot copy N)')
TS9=$(extract_id "$out")
echo "    TS9=$TS9  (Copy note to clipboard — stays new)"

# ── Standalone TS10: Archive notes older than N days → blocked ───────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T3" \
  'Archive notes older than N days')
TS10=$(extract_id "$out")
echo "    TS10=$TS10  (Archive notes older than N days)"

apm spec "$TS10" --no-aggressive --section 'Problem' --set \
  'Long-running note files grow indefinitely.  An archive command should move
notes older than a given age threshold to a separate archive file, keeping
the main list focused on recent items.'

apm spec "$TS10" --no-aggressive --section 'Approach' --set - << 'EOF'
Read each note's timestamp prefix.  Compare to today's date.  Move notes
older than N days to ~/.jot/archive.txt.  Rewrite notes.txt with the
remaining notes atomically via temp file + rename.
EOF

apm spec "$TS10" --no-aggressive --section 'Open questions' --set \
  'What "age" threshold is appropriate — calendar days since note was written,
or days since last viewed? Waiting on supervisor guidance.'

apm state "$TS10" --no-aggressive --force blocked
echo "    TS10 → blocked"

# ── Standalone TS11: Shell completion scripts (bash/zsh) → specd ─────────────
out=$(apm new --no-edit --no-aggressive 'Shell completion scripts (bash/zsh)')
TS11=$(extract_id "$out")
echo "    TS11=$TS11  (Shell completion scripts)"

apm spec "$TS11" --no-aggressive --section 'Problem' --set \
  'Users who rely on tab completion cannot complete jot subcommand names or
flags from their shell.  Providing generated completion scripts would reduce
friction for power users.'

apm spec "$TS11" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot completions bash prints a bash completion script to stdout
- [ ] jot completions zsh prints a zsh completion script to stdout
- [ ] the scripts complete subcommand names and known flags
- [ ] installation instructions are documented in the README
EOF

apm spec "$TS11" --no-aggressive --section 'Out of scope' --set \
  'Fish shell completions, PowerShell completions, dynamic completion of note text.'

apm spec "$TS11" --no-aggressive --section 'Approach' --set \
  'Hardcode completion scripts as string literals in a cmd_completions function.
Route the "completions" subcommand to it.  Print the appropriate script based
on the shell argument (bash or zsh).'

apm state "$TS11" --no-aggressive --force specd
echo "    TS11 → specd"

# ── Standalone TS12: Man page generation → question ──────────────────────────
out=$(apm new --no-edit --no-aggressive 'Man page generation')
TS12=$(extract_id "$out")
echo "    TS12=$TS12  (Man page generation)"

apm spec "$TS12" --no-aggressive --section 'Problem' --set \
  'Power users expect "man jot" to work.  The project currently has no man
page, which is a gap for users who prefer offline documentation.'

apm spec "$TS12" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] man jot displays the jot man page after installation
- [ ] the man page covers all implemented subcommands with examples
EOF

apm spec "$TS12" --no-aggressive --section 'Open questions' --set - << 'EOF'
Should the man page be generated from a hand-written Markdown file (using
pandoc) or auto-generated from clap's help text? Decision needed before
design can start.
EOF

apm state "$TS12" --no-aggressive --force question
echo "    TS12 → question"

# ── Standalone TS13: Encrypted notes at rest → in_progress ───────────────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T2" \
  'Encrypted notes at rest')
TS13=$(extract_id "$out")
echo "    TS13=$TS13  (Encrypted notes at rest)"

apm spec "$TS13" --no-aggressive --section 'Problem' --set \
  'Notes stored in plaintext at ~/.jot/notes.txt are readable by any process
with filesystem access.  Sensitive notes — passwords, personal reminders —
are exposed to other users and processes on the same machine.'

apm spec "$TS13" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot transparently encrypts notes.txt using a user-supplied passphrase
- [ ] the passphrase is prompted once per session or read from $JOT_PASSPHRASE
EOF

apm state "$TS13" --no-aggressive --force in_progress
echo "    TS13 → in_progress"

# ── Standalone TS14: Import notes from a plain-text file → groomed ───────────
out=$(apm new --no-edit --no-aggressive --depends-on "$T2" \
  'Import notes from a plain-text file')
TS14=$(extract_id "$out")
echo "    TS14=$TS14  (Import notes from a plain-text file)"

apm spec "$TS14" --no-aggressive --section 'Problem' --set \
  'Users migrating from another notes tool want to bulk-import existing note
files into jot without adding them one by one via jot add.'

apm spec "$TS14" --no-aggressive --section 'Acceptance criteria' --set - << 'EOF'
- [ ] jot import <file> appends each line of <file> to the active notebook as a new note
- [ ] lines beginning with # in the source file are treated as comments and skipped
EOF

apm state "$TS14" --no-aggressive --force groomed
echo "    TS14 → groomed"

echo ""
echo "==> All 35 tickets created and transitioned"

# ─── 8. Write README.md ───────────────────────────────────────────────────────

echo ""
echo "==> Writing README.md"

cat > README.md << 'README'
# apm-demo

A self-contained demo repository for exploring [APM](https://github.com/philippepascal/apm)
— a git-native, agent-first project management tool.

The "software project" here is **jot**, a minimal command-line notes tool written in Rust.
It is intentionally simple so the ticket backlog is easy to understand at a glance.
The interesting part is the APM ticket set: **35 tickets** spread across all 11 workflow
states, two epics, cross-ticket dependencies, a pending amendment request, and an open
question waiting for supervisor input.

Clone this repo and you have an instant APM sandbox.

---

## About jot

`jot` appends short notes to `~/.jot/notes.txt` and prints them back.

| Command | Status |
|---------|--------|
| `jot add "<text>"` | ✅ works |
| `jot list` | ✅ works |
| `jot delete <n>` | 🚧 stub — prints "not yet implemented" |
| `jot search <query>` | 🚧 stub — panics with `unimplemented!()` |

The unfinished commands exist because several tickets in the backlog are still
`in_progress`, `ready`, or `specd` — which is exactly what APM is designed to track.

---

## Prerequisites

| Tool | Install |
|------|---------|
| Rust + Cargo | <https://rustup.rs> |
| APM | `cargo install apm` (or see the [APM repo](https://github.com/philippepascal/apm)) |
| apm-server *(optional)* | `cargo install apm-server` |

---

## Getting started

### 1. Clone and fetch all ticket branches

```bash
git clone https://github.com/${GH_USER}/apm-demo.git
cd apm-demo
git fetch --all
```

### 2. Build jot and verify it runs

```bash
cargo build
./target/debug/jot list          # prints "(no notes yet ...)"
./target/debug/jot add "hello"   # adds your first note
./target/debug/jot list          # shows:    1  hello
```

### 3. List all tickets

```bash
apm list
```

You should see 35 tickets in a variety of states:
`new`, `groomed`, `in_design`, `specd`, `question`, `ammend`,
`ready`, `in_progress`, `blocked`, `implemented`, `closed`.

### 4. Inspect individual tickets

```bash
apm show <id>
```

Suggested tickets to explore:

- A **closed** ticket (T1 or T2): fully-populated spec with all four sections
  filled and every acceptance criterion checked `[x]`.
- The **ammend** ticket (T12): a supervisor has left an unchecked amendment
  request in `### Amendment requests`.
- The **question** ticket (T11): an unresolved design question is blocking
  spec completion in `### Open questions`.

### 5. Find the next actionable ticket

```bash
apm next
```

Returns the highest-priority `ready` ticket an agent can claim.  Try
transitioning it and running `apm next` again:

```bash
apm state <id> in_progress   # claim the ticket
apm next                     # now shows the next ready ticket
```

### 6. Browse the epics

```bash
apm epic list                    # shows both epics with ticket counts
apm epic show <epic-id>          # lists T5, T6, T12 and their states (Search feature)
apm epic show <epic2-id>         # lists TE1–TE7 and their states (Multi-notebook support)
```

### 7. Launch the web UI

```bash
apm-server
# then open http://localhost:3000
```

The web UI shows the same ticket data as `apm list` in a kanban-style board
with richer detail views and one-click state transitions.

---

## Next steps

```bash
apm help       # full command reference
apm work       # orchestrate AI worker agents (requires Claude CLI)
apm register   # pair a device with a running apm-server instance
```

---

## Ticket inventory

| Title | State | Epic | Depends on |
|-------|-------|------|-----------|
| Initial CLI scaffold | closed | — | — |
| Add note to file (jot add) | closed | — | T1 |
| List notes command (jot list) | implemented | — | T2 |
| Delete note command (jot delete) | in_progress | — | T3 |
| Add full-text search | in_progress | Search feature | T3 |
| Search result highlighting | ready | Search feature | T5 |
| Export notes to markdown | specd | — | — |
| Note tagging support | in_design | — | — |
| Configuration file support | groomed | — | — |
| Pagination for long note lists | new | — | — |
| Interactive TUI mode | question | — | — |
| Fuzzy search fallback | ammend | Search feature | T5 |
| Fix list command index off-by-one | blocked | — | — |
| Add --count flag to jot list | ready | — | T3 |
| Create a named notebook | closed | Multi-notebook support | — |
| Switch active notebook | closed | Multi-notebook support | — |
| List all notebooks | implemented | Multi-notebook support | TE1, TE2 |
| Delete a notebook | ready | Multi-notebook support | TE1, TE2 |
| Rename a notebook | specd | Multi-notebook support | TE1, TE2 |
| Move note between notebooks | in_design | Multi-notebook support | TE3 |
| Merge two notebooks | new | Multi-notebook support | TE6 |
| Add --version flag | closed | — | — |
| Colorize list output | implemented | — | T3 |
| Record timestamp on note creation | implemented | — | T2 |
| Edit a note in-place (jot edit N) | specd | — | T3 |
| Clear all notes (jot clear) | ready | — | T3 |
| Word count and stats (jot stats) | in_design | — | — |
| Deduplicate notes | groomed | — | T3 |
| Pin a note to the top of jot list | new | — | T3 |
| Copy note to clipboard (jot copy N) | new | — | T3 |
| Archive notes older than N days | blocked | — | T3 |
| Shell completion scripts (bash/zsh) | specd | — | — |
| Man page generation | question | — | — |
| Encrypted notes at rest | in_progress | — | T2 |
| Import notes from a plain-text file | groomed | — | T2 |
README

# ─── 9. Final commit + push all branches ──────────────────────────────────────

echo ""
echo "==> Committing Cargo project and README"
git add Cargo.toml src/ README.md
git add .apm/agents.md .apm/apm.spec-writer.md .apm/apm.worker.md 2>/dev/null || true
git add CLAUDE.md 2>/dev/null || true
git commit -m "Add jot Cargo project, README, and apm agent docs"

echo ""
echo "==> Pushing all branches to GitHub"
git push origin main
git push origin --all

echo ""
echo "Done!  Repository is live at https://github.com/${REPO}"
echo ""
echo "Verify:"
echo "  gh repo view ${REPO} --web"
echo "  git clone https://github.com/${REPO}.git /tmp/apm-demo-verify"
echo "  cd /tmp/apm-demo-verify && git fetch --all && apm list"
