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

TAP_REPO="philippepascal/homebrew-tap"
APM_REPO="philippepascal/apm"

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------

command -v gh    >/dev/null || abort "gh CLI not found"
command -v brew  >/dev/null || abort "brew not found"
command -v cargo >/dev/null || abort "cargo not found"
command -v curl  >/dev/null || abort "curl not found"

# ---------------------------------------------------------------------------
# Find the latest release
# ---------------------------------------------------------------------------

LATEST_TAG=$(gh release view --repo "$APM_REPO" --json tagName -q .tagName 2>/dev/null) \
    || abort "No releases found on $APM_REPO"
VERSION="${LATEST_TAG#v}"

bold "Latest release: $LATEST_TAG (version $VERSION)"
echo

# ---------------------------------------------------------------------------
# Publish crates to crates.io
# ---------------------------------------------------------------------------

is_published() {
    local crate="$1"
    local version="$2"
    local status
    status=$(curl -sIo /dev/null -w "%{http_code}" \
        "https://crates.io/api/v1/crates/$crate/$version" 2>/dev/null || echo "000")
    [[ "$status" == "200" ]]
}

wait_for_published() {
    local crate="$1"
    local version="$2"
    local max_wait=90
    local elapsed=0
    printf 'Waiting for %s %s to appear on crates.io' "$crate" "$version"
    while ! is_published "$crate" "$version"; do
        if (( elapsed >= max_wait )); then
            echo
            abort "Timed out waiting for $crate $version on crates.io"
        fi
        printf '.'
        sleep 3
        elapsed=$((elapsed + 3))
    done
    echo ' ok'
}

publish_crate() {
    local crate="$1"
    if is_published "$crate" "$VERSION"; then
        green "✓ $crate $VERSION already on crates.io — skipping"
        return
    fi
    bold "Publishing $crate $VERSION..."
    cargo publish -p "$crate" || abort "$crate publish failed"
    green "✓ $crate $VERSION published"
}

printf 'Publish crates to crates.io? [y/N] '
read -r answer
if [[ "$answer" =~ ^[Yy]$ ]]; then
    # cargo publish uses the local working tree — warn if HEAD isn't at the tag
    HEAD_TAG=$(git describe --tags --exact-match HEAD 2>/dev/null || echo "")
    if [[ "$HEAD_TAG" != "$LATEST_TAG" ]]; then
        red "Warning: HEAD is not at $LATEST_TAG (currently at: ${HEAD_TAG:-<no tag>})"
        confirm "Continue anyway? cargo publish will use the current working tree."
    fi

    # apm-core must be live on crates.io before apm-cli and apm-server publish
    publish_crate apm-core
    if ! is_published apm-cli "$VERSION" || ! is_published apm-server "$VERSION"; then
        wait_for_published apm-core "$VERSION"
    fi
    publish_crate apm-cli
    publish_crate apm-server

    echo
    green "All crates published or already live on crates.io"
    echo
else
    bold "Skipping crates.io publish"
    echo
fi

# Check release assets exist
ASSETS=$(gh release view "$LATEST_TAG" --repo "$APM_REPO" --json assets -q '.assets[].name')
echo "$ASSETS" | grep -q "checksums.txt" || abort "Release $LATEST_TAG has no checksums.txt — CI may still be running"

EXPECTED_ASSETS=(
    "apm-${LATEST_TAG}-aarch64-apple-darwin.tar.gz"
    "checksums.txt"
)

for asset in "${EXPECTED_ASSETS[@]}"; do
    if ! echo "$ASSETS" | grep -q "^${asset}$"; then
        abort "Missing release asset: $asset"
    fi
done

green "All expected assets present"
echo

# ---------------------------------------------------------------------------
# Download checksums
# ---------------------------------------------------------------------------

bold "Fetching checksums..."
CHECKSUMS=$(gh release download "$LATEST_TAG" --repo "$APM_REPO" --pattern "checksums.txt" --output - 2>/dev/null) \
    || abort "Failed to download checksums.txt"

SHA_ARM64=$(echo "$CHECKSUMS" | grep "aarch64-apple-darwin" | awk '{print $1}')

[[ -n "$SHA_ARM64" ]] || abort "No arm64 checksum found in checksums.txt"

echo "  arm64:  $SHA_ARM64"
echo

# ---------------------------------------------------------------------------
# Generate updated formula
# ---------------------------------------------------------------------------

FORMULA=$(cat <<RUBY
class Apm < Formula
  desc "Agentic project manager — CLI and server"
  homepage "https://github.com/$APM_REPO"
  version "$VERSION"
  license "BSL-1.1"

  on_macos do
    on_arm do
      url "https://github.com/$APM_REPO/releases/download/v#{version}/apm-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "$SHA_ARM64"
    end
  end

  def install
    bin.install "apm"
    bin.install "apm-server"
  end

  test do
    assert_match "apm", shell_output("#{bin}/apm --help")
    assert_match "apm-server", shell_output("#{bin}/apm-server --help")
  end
end
RUBY
)

bold "Generated formula:"
echo "$FORMULA"
echo

# ---------------------------------------------------------------------------
# Push formula to tap repo
# ---------------------------------------------------------------------------

confirm "Push updated formula to $TAP_REPO?"

# Get current file SHA for the GitHub API update
FILE_SHA=$(gh api "repos/$TAP_REPO/contents/Formula/apm.rb" -q .sha 2>/dev/null) || true

ENCODED=$(echo "$FORMULA" | base64)

if [[ -n "$FILE_SHA" ]]; then
    gh api "repos/$TAP_REPO/contents/Formula/apm.rb" \
        -X PUT \
        -f message="Update apm formula to $VERSION" \
        -f content="$ENCODED" \
        -f sha="$FILE_SHA" \
        --silent
else
    gh api "repos/$TAP_REPO/contents/Formula/apm.rb" \
        -X PUT \
        -f message="Add apm formula $VERSION" \
        -f content="$ENCODED" \
        --silent
fi

green "Formula pushed to $TAP_REPO"
echo

# ---------------------------------------------------------------------------
# Verify installation
# ---------------------------------------------------------------------------

confirm "Run brew install to verify?"

bold "Updating tap..."
brew tap "$TAP_REPO" 2>/dev/null || brew tap --force-auto-update philippepascal/tap

# Reinstall if already installed
if brew list philippepascal/tap/apm &>/dev/null; then
    bold "Upgrading existing installation..."
    brew upgrade philippepascal/tap/apm || brew reinstall philippepascal/tap/apm
else
    bold "Installing..."
    brew install philippepascal/tap/apm
fi

echo
bold "Verifying binaries..."

if apm --help >/dev/null 2>&1; then
    green "✓ apm --help works"
else
    red "✗ apm --help failed"
fi

if apm-server --help >/dev/null 2>&1; then
    green "✓ apm-server --help works"
else
    red "✗ apm-server --help failed"
fi

echo
bold "Running brew test..."
brew test philippepascal/tap/apm && green "✓ brew test passed" || red "✗ brew test failed"

echo
green "Done. Release $LATEST_TAG is live."
echo
bold "Links:"
echo "  GitHub release: https://github.com/$APM_REPO/releases/tag/$LATEST_TAG"
echo "  Homebrew:       brew install philippepascal/tap/apm"
if is_published apm-core "$VERSION"; then
    echo "  apm-core:       https://crates.io/crates/apm-core/$VERSION"
fi
if is_published apm-cli "$VERSION"; then
    echo "  apm-cli:        https://crates.io/crates/apm-cli/$VERSION"
fi
if is_published apm-server "$VERSION"; then
    echo "  apm-server:     https://crates.io/crates/apm-server/$VERSION"
fi
