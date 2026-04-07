#!/bin/sh
set -e

# ---------------------------------------------------------------------------
# APM installer
# Supports: macOS Apple Silicon (aarch64-apple-darwin)
#           Linux x86_64 (x86_64-unknown-linux-musl)
# ---------------------------------------------------------------------------

# ---- Platform detection ----------------------------------------------------
OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}/${ARCH}" in
    Darwin/arm64)
        TARGET="aarch64-apple-darwin"
        ;;
    Linux/x86_64)
        TARGET="x86_64-unknown-linux-musl"
        ;;
    *)
        echo "Error: Unsupported platform: ${OS}/${ARCH}" >&2
        echo "Supported platforms:" >&2
        echo "  macOS Apple Silicon — Darwin/arm64" >&2
        echo "  Linux x86_64        — Linux/x86_64" >&2
        exit 1
        ;;
esac

# ---- Prerequisites ---------------------------------------------------------
for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Error: '${cmd}' is required but was not found on PATH." >&2
        exit 1
    fi
done

# ---- Resolve version -------------------------------------------------------
if [ -n "${APM_VERSION:-}" ]; then
    VERSION=$(printf '%s' "$APM_VERSION" | sed 's/^v//')
else
    echo "Fetching latest APM release version..."
    VERSION=$(curl -fsSL https://api.github.com/repos/philippepascal/apm/releases/latest \
        | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine the latest APM version." >&2
        echo "  The GitHub API may be rate-limiting unauthenticated requests." >&2
        echo "  Set APM_VERSION=<version> and re-run the installer, e.g.:" >&2
        echo "    APM_VERSION=0.1.0 sh install.sh" >&2
        exit 1
    fi
fi

echo "Installing APM v${VERSION} for ${TARGET}..."

# ---- Build download URLs ---------------------------------------------------
BASE="https://github.com/philippepascal/apm/releases/download/v${VERSION}"
ARCHIVE_NAME="apm-v${VERSION}-${TARGET}.tar.gz"
ARCHIVE_URL="${BASE}/${ARCHIVE_NAME}"
CHECKSUM_URL="${BASE}/checksums.txt"

# ---- Temp directory (cleaned up on exit) -----------------------------------
APM_TMP=$(mktemp -d)
trap 'rm -rf "$APM_TMP"' EXIT

# ---- Download --------------------------------------------------------------
echo "Downloading ${ARCHIVE_NAME}..."
curl -fsSL -o "${APM_TMP}/${ARCHIVE_NAME}" "$ARCHIVE_URL"

echo "Downloading checksums.txt..."
curl -fsSL -o "${APM_TMP}/checksums.txt" "$CHECKSUM_URL"

# ---- Verify checksum -------------------------------------------------------
echo "Verifying checksum..."

EXPECTED=$(grep " ${ARCHIVE_NAME}$" "${APM_TMP}/checksums.txt" | awk '{print $1}')
if [ -z "$EXPECTED" ]; then
    echo "Error: Could not find a checksum entry for ${ARCHIVE_NAME} in checksums.txt." >&2
    exit 1
fi

if [ "$OS" = "Darwin" ]; then
    ACTUAL=$(shasum -a 256 "${APM_TMP}/${ARCHIVE_NAME}" | awk '{print $1}')
else
    ACTUAL=$(sha256sum "${APM_TMP}/${ARCHIVE_NAME}" | awk '{print $1}')
fi

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Error: Checksum verification failed!" >&2
    echo "  Expected: ${EXPECTED}" >&2
    echo "  Actual:   ${ACTUAL}" >&2
    echo "The downloaded archive may be corrupt or tampered with." >&2
    exit 1
fi

echo "Checksum verified."

# ---- Resolve install directory ---------------------------------------------
INSTALL_DIR="${APM_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

# ---- Extract and install ---------------------------------------------------
echo "Extracting and installing binaries to ${INSTALL_DIR}..."
tar -xzf "${APM_TMP}/${ARCHIVE_NAME}" -C "$APM_TMP"
cp "${APM_TMP}/apm" "${INSTALL_DIR}/apm"
cp "${APM_TMP}/apm-server" "${INSTALL_DIR}/apm-server"
chmod +x "${INSTALL_DIR}/apm" "${INSTALL_DIR}/apm-server"

# ---- Add to PATH -----------------------------------------------------------
SENTINEL="# Added by APM installer"

for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
    if [ ! -f "$rc" ]; then
        continue
    fi
    if grep -qF "$INSTALL_DIR" "$rc"; then
        continue
    fi
    printf '\n%s\nexport PATH="%s:$PATH"\n' "$SENTINEL" "$INSTALL_DIR" >> "$rc"
    echo "Updated ${rc}"
done

# ---- Success ---------------------------------------------------------------
echo ""
echo "APM v${VERSION} installed to ${INSTALL_DIR}"
echo ""
echo "  apm --help        to get started"
echo "  apm-server --help for the web server"
echo ""
echo "Restart your shell or run:  export PATH=\"${INSTALL_DIR}:\$PATH\""
