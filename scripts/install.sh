#!/usr/bin/env bash
set -euo pipefail

REPO="12yanogden/bin"
INSTALL_DIR="${BIN_INSTALL_DIR:-$HOME/.bin}"
NON_INTERACTIVE=false
SELECTED_TAGS=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
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

ENABLED_DIR="$INSTALL_DIR/enabled"
ALL_DIR="$INSTALL_DIR/all"

# Check dependencies
if ! command -v python3 &>/dev/null; then
    echo "Error: python3 is required but not installed" >&2
    exit 1
fi

if [[ "$NON_INTERACTIVE" == "false" ]]; then
    if ! command -v whiptail &>/dev/null; then
        echo "Error: whiptail is required for interactive tag selection" >&2
        echo "Install it or use --tags to specify tags non-interactively" >&2
        exit 1
    fi
fi

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
mkdir -p "$ALL_DIR"
mkdir -p "$ENABLED_DIR"

# Extract binaries into all/
echo "Extracting binaries..."
tar -xf "$TMPDIR/$ARCHIVE_NAME" -C "$TMPDIR"

# Find and copy binaries (cargo-dist places them in a subdirectory)
EXTRACTED_DIR=$(find "$TMPDIR" -mindepth 1 -maxdepth 1 -type d ! -name ".*" | head -1)
if [[ -n "$EXTRACTED_DIR" ]]; then
    cp "$EXTRACTED_DIR"/* "$ALL_DIR/" 2>/dev/null || true
else
    # Binaries may be directly in tmpdir
    cp "$TMPDIR"/bin-*/* "$ALL_DIR/" 2>/dev/null || true
fi

chmod +x "$ALL_DIR"/*

# Copy tags.json to project root
cp "$TMPDIR/tags.json" "$INSTALL_DIR/tags.json"

# Update PATH in ~/.zshrc
SHELL_RC="$HOME/.zshrc"
MARKER="# bin-tools"
PATH_LINE="export PATH=\"\$PATH:$INSTALL_DIR/enabled\" $MARKER"

if [[ -f "$SHELL_RC" ]]; then
    TMP_RC="$(mktemp)"
    grep -v "$MARKER" "$SHELL_RC" > "$TMP_RC" || true
    mv "$TMP_RC" "$SHELL_RC"
fi

printf '\n%s\n' "$PATH_LINE" >> "$SHELL_RC"
echo "Added $INSTALL_DIR/enabled to PATH in $SHELL_RC"

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
fi

# Create symlinks in enabled/ for selected tag commands
while IFS= read -r tag; do
    IS_SELECTED=false
    for selected in "${SELECTED_TAGS[@]}"; do
        if [[ "$tag" == "$selected" ]]; then
            IS_SELECTED=true
            break
        fi
    done

    if [[ "$IS_SELECTED" == "true" ]]; then
        CMDS=$(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
for cmd in tags.get('$tag', []):
    print(cmd)
")
        while IFS= read -r cmd; do
            if [[ -n "$cmd" && -f "$ALL_DIR/$cmd" && ! -L "$ENABLED_DIR/$cmd" ]]; then
                ln -s "$ALL_DIR/$cmd" "$ENABLED_DIR/$cmd"
                echo "  Enabled: $cmd (tag: $tag)"
            fi
        done <<< "$CMDS"
    fi
done <<< "$ALL_TAGS"

echo ""
echo "Installation complete!"
echo "  Binary directory:   $ALL_DIR"
echo "  Enabled directory:  $ENABLED_DIR"
echo ""
echo "Restart your shell or run: source ~/.zshrc"
