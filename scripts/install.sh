#!/usr/bin/env bash
set -euo pipefail

REPO="12yanogden/bin"
INSTALL_DIR="$HOME/Projects/rust/bin"
ENABLED_DIR="$INSTALL_DIR/enabled"
DISABLED_DIR="$INSTALL_DIR/disabled"
NON_INTERACTIVE=false
SELECTED_TAGS=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --tags)
            NON_INTERACTIVE=true
            IFS=',' read -ra SELECTED_TAGS <<< "$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

# Detect platform and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Darwin-arm64|Darwin-aarch64) TARGET="aarch64-apple-darwin" ;;
    Darwin-x86_64)               TARGET="x86_64-apple-darwin" ;;
    Linux-x86_64)                TARGET="x86_64-unknown-linux-gnu" ;;
    Linux-aarch64)               TARGET="aarch64-unknown-linux-gnu" ;;
    *)
        echo "Unsupported platform: ${OS}-${ARCH}" >&2
        exit 1
        ;;
esac

echo "Detected platform: ${TARGET}"

# Get latest release tag
echo "Fetching latest release..."
LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST_TAG" ]]; then
    echo "Failed to determine latest release tag" >&2
    exit 1
fi

echo "Latest release: ${LATEST_TAG}"

# Download archive and tags.json to temp directory
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

ARCHIVE_NAME="bin-${TARGET}.tar.xz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ARCHIVE_NAME"
TAGS_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/tags.json"

echo "Downloading ${ARCHIVE_NAME}..."
curl -sL -o "$TMPDIR/$ARCHIVE_NAME" "$DOWNLOAD_URL"
if [[ ! -s "$TMPDIR/$ARCHIVE_NAME" ]]; then
    echo "Failed to download archive" >&2
    exit 1
fi

echo "Downloading tags.json..."
curl -sL -o "$TMPDIR/tags.json" "$TAGS_URL"
if [[ ! -s "$TMPDIR/tags.json" ]]; then
    echo "Failed to download tags.json" >&2
    exit 1
fi

# Create directories
mkdir -p "$ENABLED_DIR"
mkdir -p "$DISABLED_DIR"

# Extract binaries into enabled/
echo "Extracting binaries..."
tar -xf "$TMPDIR/$ARCHIVE_NAME" -C "$TMPDIR"

# Find and copy binaries (cargo-dist places them in a subdirectory)
EXTRACTED_DIR=$(find "$TMPDIR" -mindepth 1 -maxdepth 1 -type d ! -name ".*" | head -1)
if [[ -n "$EXTRACTED_DIR" ]]; then
    cp "$EXTRACTED_DIR"/* "$ENABLED_DIR/" 2>/dev/null || true
else
    # Binaries may be directly in tmpdir
    cp "$TMPDIR"/bin-*/* "$ENABLED_DIR/" 2>/dev/null || true
fi

chmod +x "$ENABLED_DIR"/*

# Copy tags.json to project root
cp "$TMPDIR/tags.json" "$INSTALL_DIR/tags.json"

# Update PATH in ~/.zshrc
PATH_LINE='export PATH="$PATH:~/Projects/rust/bin/enabled"'
MARKER="# Personal binaries"
if ! grep -qF "$MARKER" "$HOME/.zshrc" 2>/dev/null; then
    printf '\n%s\n%s\n' "$MARKER" "$PATH_LINE" >> "$HOME/.zshrc"
    echo "Added enabled/ to PATH in ~/.zshrc"
fi

# Tag selection
ALL_TAGS=$(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
for tag in sorted(tags.keys()):
    print(tag)
")

if [[ "$NON_INTERACTIVE" == "true" ]]; then
    # Ensure bin-admin is always included
    HAS_BIN_ADMIN=false
    for tag in "${SELECTED_TAGS[@]}"; do
        if [[ "$tag" == "bin-admin" ]]; then
            HAS_BIN_ADMIN=true
            break
        fi
    done
    if [[ "$HAS_BIN_ADMIN" == "false" ]]; then
        SELECTED_TAGS+=("bin-admin")
    fi
else
    # Interactive mode with whiptail
    if command -v whiptail &>/dev/null; then
        CHECKLIST_ITEMS=()
        while IFS= read -r tag; do
            CMD_COUNT=$(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
print(len(tags.get('$tag', [])))
")
            CHECKLIST_ITEMS+=("$tag" "${CMD_COUNT} commands" "ON")
        done <<< "$ALL_TAGS"

        SELECTED=$(whiptail --checklist "Select tags to enable:" 20 60 10 "${CHECKLIST_ITEMS[@]}" 3>&1 1>&2 2>&3) || true

        # Parse whiptail output (quoted, space-separated)
        SELECTED_TAGS=()
        for tag in $SELECTED; do
            tag="${tag//\"/}"
            SELECTED_TAGS+=("$tag")
        done

        # Ensure bin-admin is always selected
        HAS_BIN_ADMIN=false
        for tag in "${SELECTED_TAGS[@]}"; do
            if [[ "$tag" == "bin-admin" ]]; then
                HAS_BIN_ADMIN=true
                break
            fi
        done
        if [[ "$HAS_BIN_ADMIN" == "false" ]]; then
            SELECTED_TAGS+=("bin-admin")
        fi
    else
        echo "whiptail not found, enabling all tags"
        SELECTED_TAGS=()
        while IFS= read -r tag; do
            SELECTED_TAGS+=("$tag")
        done <<< "$ALL_TAGS"
    fi
fi

# Move unselected tag commands to disabled/
while IFS= read -r tag; do
    IS_SELECTED=false
    for selected in "${SELECTED_TAGS[@]}"; do
        if [[ "$tag" == "$selected" ]]; then
            IS_SELECTED=true
            break
        fi
    done

    if [[ "$IS_SELECTED" == "false" ]]; then
        CMDS=$(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
for cmd in tags.get('$tag', []):
    print(cmd)
")
        while IFS= read -r cmd; do
            if [[ -f "$ENABLED_DIR/$cmd" ]]; then
                mv "$ENABLED_DIR/$cmd" "$DISABLED_DIR/$cmd"
                echo "  Disabled: $cmd (tag: $tag)"
            fi
        done <<< "$CMDS"
    fi
done <<< "$ALL_TAGS"

echo ""
echo "Installation complete!"
echo "  Enabled directory:  $ENABLED_DIR"
echo "  Disabled directory: $DISABLED_DIR"
echo ""
echo "Restart your shell or run: source ~/.zshrc"
