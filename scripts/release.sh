#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

red()   { printf '\033[0;31m%s\033[0m\n' "$*"; }
green() { printf '\033[0;32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

abort() { red "ERROR: $*" >&2; exit 1; }

confirm() {
    local prompt="$1"
    printf '%s [y/N] ' "$prompt"
    read -r answer
    [[ "$answer" =~ ^[Yy]$ ]] || abort "Aborted."
}

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------

command -v cargo >/dev/null || abort "cargo not found"
command -v git   >/dev/null || abort "git not found"
command -v gh    >/dev/null || abort "gh CLI not found"

[[ "$(git rev-parse --abbrev-ref HEAD)" == "main" ]] || abort "Not on main branch"
[[ -z "$(git status --porcelain)" ]] || abort "Working tree is not clean — commit or stash first"

git fetch --tags --quiet

# ---------------------------------------------------------------------------
# Current version
# ---------------------------------------------------------------------------

CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
LATEST_TAG=$(git tag -l 'v*' --sort=-v:refname | head -1)

bold "Current Cargo.toml version: $CARGO_VERSION"
if [[ -n "$LATEST_TAG" ]]; then
    bold "Latest git tag:             $LATEST_TAG"
else
    bold "Latest git tag:             (none)"
fi
echo

# ---------------------------------------------------------------------------
# Ask for new version
# ---------------------------------------------------------------------------

printf 'New version (without v prefix): '
read -r NEW_VERSION

[[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || abort "Invalid semver: $NEW_VERSION"

TAG="v$NEW_VERSION"

git rev-parse "$TAG" >/dev/null 2>&1 && abort "Tag $TAG already exists"

echo
bold "Will release: $TAG"
echo "  Cargo.toml versions: $CARGO_VERSION → $NEW_VERSION"
echo "  Tag:                 $TAG on $(git rev-parse --short HEAD)"
echo

# ---------------------------------------------------------------------------
# Update Cargo.toml versions
# ---------------------------------------------------------------------------

if [[ "$CARGO_VERSION" != "$NEW_VERSION" ]]; then
    confirm "Update Cargo.toml versions to $NEW_VERSION?"

    # Workspace version (inherited by each crate via `version.workspace = true`)
    sed -i '' "s/^version = \"$CARGO_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

    # apm-core path-dep version in the dependent crates
    for toml in apm/Cargo.toml apm-server/Cargo.toml; do
        sed -i '' "s/apm-core\(.*\)version = \"$CARGO_VERSION\"/apm-core\1version = \"$NEW_VERSION\"/" "$toml"
    done

    echo
    bold "Updated version fields:"
    grep -n '^version' Cargo.toml
    grep -n 'apm-core.*version' apm/Cargo.toml apm-server/Cargo.toml
    echo

    # Verify it still builds
    bold "Running cargo check..."
    cargo check --workspace --quiet
    green "cargo check passed"
    echo

    bold "Running cargo test..."
    cargo test --workspace --quiet
    green "All tests passed"
    echo

    confirm "Commit version bump?"
    git add Cargo.toml apm/Cargo.toml apm-server/Cargo.toml Cargo.lock
    git commit -m "Release $TAG"
    green "Committed version bump"
    echo
else
    bold "Cargo.toml already at $NEW_VERSION — skipping version bump"
    echo
fi

# ---------------------------------------------------------------------------
# Tag and push
# ---------------------------------------------------------------------------

confirm "Create tag $TAG and push to origin?"

git tag -a "$TAG" -m "Release $TAG"
green "Created tag $TAG"

git push origin main
git push origin "$TAG"
green "Pushed main and $TAG to origin"

echo
bold "Release CI triggered. Monitor at:"
echo "  https://github.com/philippepascal/apm/actions"
echo
bold "After CI completes:"
echo "  1. Check the release at https://github.com/philippepascal/apm/releases/tag/$TAG"
echo "  2. Update homebrew-tap Formula/apm.rb with SHA-256 values from checksums.txt"
echo "  3. Run: brew tap philippepascal/tap && brew install philippepascal/tap/apm"
