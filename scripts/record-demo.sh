#!/usr/bin/env bash
# scripts/record-demo.sh — drive a self-contained apm demo session suitable
# for screen recording.
#
# Usage:
#   bash scripts/record-demo.sh [--keep-dir]
#
# Requirements: apm in PATH. No GitHub account, no Claude CLI, no API keys.

set -euo pipefail

# ─── 1. Flag parsing and cleanup trap ────────────────────────────────────────

KEEP_DIR=false
for arg in "$@"; do
  case "$arg" in
    --keep-dir) KEEP_DIR=true ;;
    *) echo "ERROR: unknown flag: $arg"; exit 1 ;;
  esac
done

WORKDIR=$(mktemp -d)
echo "Working directory: $WORKDIR"

cleanup() {
  if ! "$KEEP_DIR"; then
    echo "Cleaning up $WORKDIR"
    rm -rf "$WORKDIR"
  else
    echo "Keeping $WORKDIR (--keep-dir)"
  fi
}
trap cleanup EXIT

# ─── 2. Local bare-repo remote ────────────────────────────────────────────────

echo ""
echo "==> Setting up local bare-repo remote"
git init --bare "$WORKDIR/jot.git"
git -C "$WORKDIR/jot.git" symbolic-ref HEAD refs/heads/main
git clone "$WORKDIR/jot.git" "$WORKDIR/jot"
cd "$WORKDIR/jot"
git config user.email "demo@example.com"
git config user.name "APM Demo"

# ─── 3. Minimal project setup ─────────────────────────────────────────────────

echo ""
echo "==> Writing project files"
mkdir -p src .apm

cat > Cargo.toml << 'CARGO_TOML'
[package]
name = "jot"
version = "0.1.0"
edition = "2021"
CARGO_TOML

cat > src/main.rs << 'MAIN_RS'
fn main() {
    println!("jot");
}
MAIN_RS

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
command = "mock-happy"

[logging]
enabled = false
file = "~/.local/state/apm/jot.log"
APM_CONFIG

echo ""
echo "==> Initialising APM"
apm init --no-claude

# Commit any project files not included in apm init's initial commit
git add Cargo.toml src/ .apm/config.toml 2>/dev/null || true
git diff --cached --quiet || git commit -m "Add jot Cargo project"
git push -u origin main

# ─── 4. Ticket creation ───────────────────────────────────────────────────────

echo ""
echo "==> Creating tickets"
extract_id() { echo "$1" | awk '{print $3}' | tr -d ':'; }

# ── Ticket 1: Implement jot add command → ready ───────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Implement jot add command')
T1=$(extract_id "$out")
echo "    T1=$T1  (Implement jot add command)"

apm spec "$T1" --no-aggressive --section 'Problem' --set \
  'Users need to append notes from the command line. jot add "<text>" should write a line to ~/.jot/notes.txt.'
apm spec "$T1" --no-aggressive --section 'Acceptance criteria' --set \
  '- [ ] jot add "<text>" appends the text to ~/.jot/notes.txt
- [ ] the file and directory are created on first use
- [ ] success prints "Note added." to stdout'
apm spec "$T1" --no-aggressive --section 'Out of scope' --set \
  'Tags, timestamps, or any metadata beyond the raw text line.'
apm spec "$T1" --no-aggressive --section 'Approach' --set \
  'Use OpenOptions::create(true).append(true) and fs::create_dir_all on the parent directory.'
apm set "$T1" effort 2 --no-aggressive
apm set "$T1" risk 1 --no-aggressive
apm state "$T1" --no-aggressive --force ready
echo "    T1 → ready"

# ── Ticket 2: Implement jot list command → ready ──────────────────────────────
out=$(apm new --no-edit --no-aggressive 'Implement jot list command')
T2=$(extract_id "$out")
echo "    T2=$T2  (Implement jot list command)"

apm spec "$T2" --no-aggressive --section 'Problem' --set \
  'Users need to view all notes. jot list should print each line from notes.txt with a 1-based index.'
apm spec "$T2" --no-aggressive --section 'Acceptance criteria' --set \
  '- [ ] jot list prints each note prefixed by a right-aligned 1-based index
- [ ] when notes.txt is absent, prints a friendly "no notes yet" message'
apm spec "$T2" --no-aggressive --section 'Out of scope' --set \
  'Sorting, filtering, pagination.'
apm spec "$T2" --no-aggressive --section 'Approach' --set \
  'Open notes.txt with BufReader. Enumerate from 1. Print with format!("{:>4}  {}", i, line).'
apm set "$T2" effort 2 --no-aggressive
apm set "$T2" risk 1 --no-aggressive
apm state "$T2" --no-aggressive --force ready
echo "    T2 → ready"

# ── Ticket 3: Implement jot delete command → ready ───────────────────────────
out=$(apm new --no-edit --no-aggressive 'Implement jot delete command')
T3=$(extract_id "$out")
echo "    T3=$T3  (Implement jot delete command)"

apm spec "$T3" --no-aggressive --section 'Problem' --set \
  'Users need to remove individual notes by index. jot delete N removes the Nth note.'
apm spec "$T3" --no-aggressive --section 'Acceptance criteria' --set \
  '- [ ] jot delete N removes the note at 1-based index N
- [ ] out-of-range index prints an error and exits non-zero'
apm spec "$T3" --no-aggressive --section 'Out of scope' --set \
  'Bulk delete or undo.'
apm spec "$T3" --no-aggressive --section 'Approach' --set \
  'Read all lines into Vec<String>. Validate index. Remove element. Rewrite atomically via temp file + rename.'
apm set "$T3" effort 3 --no-aggressive
apm set "$T3" risk 2 --no-aggressive
apm state "$T3" --no-aggressive --force ready
echo "    T3 → ready"

# ── Ticket 4: Add --version flag → groomed (stays groomed) ───────────────────
out=$(apm new --no-edit --no-aggressive 'Add --version flag')
T4=$(extract_id "$out")
echo "    T4=$T4  (Add --version flag)"
apm state "$T4" --no-aggressive --force groomed
echo "    T4 → groomed"

echo ""
echo "==> Pushing all branches"
git push origin --all

# ─── 5. Demo sequence ─────────────────────────────────────────────────────────

run() {
  printf "\n$ %s\n" "$*"
  sleep 0.5
  "$@"
}

run apm list
sleep 1
run apm work
sleep 1
run apm list
