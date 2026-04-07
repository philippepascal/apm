#!/bin/sh
set -e

# ---------------------------------------------------------------------------
# APM uninstaller
# Removes APM binaries and PATH entries added by install.sh
# ---------------------------------------------------------------------------

OS=$(uname -s)
INSTALL_DIR="${APM_INSTALL_DIR:-$HOME/.local/bin}"
REMOVED=0

# ---- Remove binaries -------------------------------------------------------
for bin in apm apm-server; do
    if [ -f "${INSTALL_DIR}/${bin}" ]; then
        rm "${INSTALL_DIR}/${bin}"
        echo "Removed ${INSTALL_DIR}/${bin}"
        REMOVED=1
    else
        echo "Not found: ${INSTALL_DIR}/${bin} (skipping)"
    fi
done

# ---- Remove PATH entries ---------------------------------------------------
SENTINEL="# Added by APM installer"

for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
    if [ ! -f "$rc" ]; then
        continue
    fi
    if ! grep -qF "$SENTINEL" "$rc"; then
        continue
    fi
    if [ "$OS" = "Darwin" ]; then
        sed -i '' '/^# Added by APM installer$/{N;d;}' "$rc"
    else
        sed -i '/^# Added by APM installer$/{N;d;}' "$rc"
    fi
    echo "Removed APM PATH entry from ${rc}"
    REMOVED=1
done

# ---- Summary ---------------------------------------------------------------
if [ "$REMOVED" = "0" ]; then
    echo "APM does not appear to be installed — nothing to do."
fi
