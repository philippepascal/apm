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

command -v cargo  >/dev/null || abort "cargo not found"
command -v git    >/dev/null || abort "git not found"
command -v gh     >/dev/null || abort "gh CLI not found"
command -v rustup >/dev/null || abort "rustup not found (needed to install aarch64-apple-darwin target)"
command -v npm    >/dev/null || abort "npm not found (needed to build apm-ui)"
command -v shasum >/dev/null || abort "shasum not found"

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
else
    bold "Cargo.toml already at $NEW_VERSION — skipping version bump"
    echo
fi

# Verify the build and tests. cargo check regenerates Cargo.lock so its crate
# versions match Cargo.toml — this MUST run before the release commit so the
# synced lockfile is captured in the tag. Otherwise the first `cargo build` in
# CI rewrites Cargo.lock and the release binary reports a misleading `-dirty`.
bold "Running cargo check..."
cargo check --workspace --quiet
green "cargo check passed"
echo

bold "Running cargo test..."
cargo test --workspace --quiet
green "All tests passed"
echo

# Commit any version/lockfile drift produced above.
if [[ -n "$(git status --porcelain Cargo.toml apm/Cargo.toml apm-server/Cargo.toml Cargo.lock)" ]]; then
    confirm "Commit version bump and lockfile?"
    git add Cargo.toml apm/Cargo.toml apm-server/Cargo.toml Cargo.lock
    git commit -m "Release $TAG"
    green "Committed version bump"
    echo
fi

# ---------------------------------------------------------------------------
# Tag, build macOS artifact locally, push, create GitHub Release
# ---------------------------------------------------------------------------

# Refuse to tag a dirty tree: a stale Cargo.lock or stray change would be baked
# into the release binary's `git describe --dirty` version string.
[[ -z "$(git status --porcelain)" ]] || abort "Working tree not clean before tagging — inspect: git status"

confirm "Create tag $TAG, build macOS artifact, and push to origin?"

# Print a recovery hint if anything below this point fails after the tag is created.
trap 'rc=$?; if [[ $rc -ne 0 ]] && git rev-parse "$TAG" >/dev/null 2>&1; then echo; red "Release failed after the local tag $TAG was created."; red "Remove the local tag and retry: git tag -d $TAG"; fi; exit $rc' EXIT

# Create the tag locally BEFORE building so the binary's `git describe --tags`
# resolves to a clean $TAG (no -N-gSHA suffix, no -dirty).
git tag -a "$TAG" -m "Release $TAG"
green "Created local tag $TAG"
echo

# Build macOS arm64 release artifact locally (replaces the dropped macos-14 CI job).
ARTIFACTS_DIR="target/release-artifacts"
MAC_TARBALL_NAME="apm-${TAG}-aarch64-apple-darwin.tar.gz"
MAC_TARBALL="$ARTIFACTS_DIR/$MAC_TARBALL_NAME"
CHECKSUMS="$ARTIFACTS_DIR/checksums.txt"
mkdir -p "$ARTIFACTS_DIR"

bold "Ensuring aarch64-apple-darwin Rust target is installed..."
rustup target add aarch64-apple-darwin >/dev/null
echo

bold "Building apm-ui assets..."
( cd apm-ui && npm ci --silent && npm run build --silent )
green "apm-ui built"
echo

bold "Building macOS arm64 release binaries..."
cargo build --release --target aarch64-apple-darwin -p apm-cli -p apm-server --quiet
strip target/aarch64-apple-darwin/release/apm
strip target/aarch64-apple-darwin/release/apm-server
green "Binaries built and stripped"
echo

bold "Packaging $MAC_TARBALL_NAME..."
tar -czf "$MAC_TARBALL" -C target/aarch64-apple-darwin/release apm apm-server
( cd "$ARTIFACTS_DIR" && shasum -a 256 "$MAC_TARBALL_NAME" > "checksums.txt" )
green "Packaged: $MAC_TARBALL"
echo

git push origin main
git push origin "$TAG"
green "Pushed main and $TAG to origin"
echo

bold "Creating GitHub Release $TAG..."
if gh release view "$TAG" >/dev/null 2>&1; then
    gh release upload "$TAG" --clobber "$MAC_TARBALL" "$CHECKSUMS"
    green "Uploaded macOS asset to existing release $TAG"
else
    gh release create "$TAG" --generate-notes "$MAC_TARBALL" "$CHECKSUMS"
    green "Created release $TAG with macOS asset and checksums"
fi
echo

bold "Linux musl build is running in CI. Monitor at:"
echo "  https://github.com/philippepascal/apm/actions"
echo "  (CI will append apm-${TAG}-x86_64-unknown-linux-musl.tar.gz to the release.)"
echo
bold "Next steps:"
echo "  1. Verify the release: https://github.com/philippepascal/apm/releases/tag/$TAG"
echo "  2. Run ./scripts/post-release.sh (Homebrew formula + crates.io publish)"
