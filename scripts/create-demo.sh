#!/usr/bin/env bash
# scripts/create-demo.sh — create the philippepascal/apm-demo GitHub repository.
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

REPO="philippepascal/apm-demo"
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
git clone "${REPO_URL}" "$WORKDIR/apm-demo"
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

# ─── 7. Create tickets ────────────────────────────────────────────────────────
# Helper: extract ticket ID from "apm new" output.
# Output format: "Created ticket <id>: <filename> (branch: <branch>)"
extract_id() { echo "$1" | awk '{print $3}' | tr -d ':'; }

echo ""
echo "==> Creating 14 tickets"

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

echo ""
echo "==> All 14 tickets created and transitioned"

# ─── 8. Write README.md ───────────────────────────────────────────────────────

echo ""
echo "==> Writing README.md"

cat > README.md << 'README'
# apm-demo

A self-contained demo repository for exploring [APM](https://github.com/philippepascal/apm)
— a git-native, agent-first project management tool.

The "software project" here is **jot**, a minimal command-line notes tool written in Rust.
It is intentionally simple so the ticket backlog is easy to understand at a glance.
The interesting part is the APM ticket set: **14 tickets** spread across all 11 workflow
states, one epic, cross-ticket dependencies, a pending amendment request, and an open
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
git clone https://github.com/philippepascal/apm-demo.git
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

You should see 14 tickets in a variety of states:
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

### 6. Browse the Search feature epic

```bash
apm epic list               # shows the "Search feature" epic with ticket counts
apm epic show <epic-id>     # lists T5, T6, T12 and their states
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
