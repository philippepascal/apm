#!/usr/bin/env bash
# migrate-ticket-ids.sh
#
# Migrate tickets from integer IDs to zero-padded 4-char string IDs.
# For each ticket/NNNN-* branch, rewrites `id = N` → `id = "NNNN"` in
# the frontmatter so existing tickets are compatible with the new hex-ID system.
#
# Usage: bash scripts/migrate-ticket-ids.sh [--dry-run]
#
# After migration:
#   apm show 0035  and  apm show 35  both resolve correctly.

set -euo pipefail

DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --dry-run|-n) DRY_RUN=1 ;;
    *) echo "Unknown argument: $arg" >&2; exit 1 ;;
  esac
done

GIT_ENV=(
  -c commit.gpgsign=false
  -c user.email="${GIT_AUTHOR_EMAIL:-apm-migrate@localhost}"
  -c user.name="${GIT_AUTHOR_NAME:-apm-migrate}"
)

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
CHANGED=0

for branch in $(git branch --list 'ticket/*' | sed 's/^[* ] //'); do
  suffix="${branch#ticket/}"
  # Extract the numeric prefix (the part before the first hyphen).
  id_part="${suffix%%-*}"

  # Only process branches whose prefix is purely numeric (old-style).
  if ! echo "$id_part" | grep -qE '^[0-9]+$'; then
    continue
  fi

  # Zero-pad to 4 digits.
  padded=$(printf '%04d' "$id_part")
  filename="${padded}-${suffix#*-}.md"
  tickets_dir=$(git config --file apm.toml tickets.dir 2>/dev/null || echo "tickets")
  rel_path="${tickets_dir}/${filename}"

  # Check that the file exists on this branch.
  if ! git show "${branch}:${rel_path}" >/dev/null 2>&1; then
    echo "skip $branch — $rel_path not found on branch"
    continue
  fi

  content=$(git show "${branch}:${rel_path}")

  # Skip if already a string id.
  if echo "$content" | grep -qE '^id = "[0-9a-f]+"'; then
    echo "skip $branch — id already a string"
    continue
  fi

  # Find bare integer id line, e.g. `id = 35`.
  if ! echo "$content" | grep -qE "^id = ${id_part}$"; then
    echo "skip $branch — id = ${id_part} not found in frontmatter"
    continue
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    echo "would migrate $branch: id = ${id_part} → id = \"${padded}\""
    continue
  fi

  # Apply the substitution on the branch.
  tmpfile=$(mktemp)
  git show "${branch}:${rel_path}" | sed "s/^id = ${id_part}$/id = \"${padded}\"/" > "$tmpfile"

  git checkout -q "$branch"
  cp "$tmpfile" "$rel_path"
  rm "$tmpfile"
  git "${GIT_ENV[@]}" add "$rel_path"
  git "${GIT_ENV[@]}" commit -m "ticket(${padded}): migrate id to string format"
  echo "migrated $branch"
  CHANGED=$((CHANGED + 1))
done

git checkout -q "$CURRENT_BRANCH"

if [ "$DRY_RUN" -eq 0 ] && [ "$CHANGED" -gt 0 ]; then
  echo ""
  echo "Migrated $CHANGED ticket(s). Push with:"
  echo "  git push origin 'refs/heads/ticket/*'"
fi
