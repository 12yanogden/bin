#!/usr/bin/env bash
set -euo pipefail

REPO="12yanogden/bin"
INSTALL_DIR="${BIN_INSTALL_DIR:-$HOME/.bin}"
NON_INTERACTIVE=false
SELECTED_TAGS=()

usage() {
    cat <<EOF
Usage: install.sh [OPTIONS]

Download and install binaries from the latest release.

Options:
    --dir <path>            Set install directory (default: \$HOME/.bin or \$BIN_INSTALL_DIR)
    --tags <tag1,tag2,...>   Install specific tags non-interactively
    -h, --help              Show this help message

Example:
    install.sh --dir ~/.local/bin --tags git,bin-admin
EOF
    exit 0
}

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
        --help|-h)
            usage
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

# Set up temp directory
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

TAGS_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/tags.json"

# Download tags.json
echo "Downloading tags.json..."
curl -sL -o "$TMPDIR/tags.json" "$TAGS_URL"
if [[ ! -s "$TMPDIR/tags.json" ]]; then
    echo "Failed to download tags.json" >&2
    exit 1
fi

# Create directories
mkdir -p "$ALL_DIR"
mkdir -p "$ENABLED_DIR"

# Get list of all binaries from tags.json
BINARIES=$(python3 -c "
import json
with open('$TMPDIR/tags.json') as f:
    tags = json.load(f)
seen = set()
for cmds in tags.values():
    for cmd in cmds:
        if cmd not in seen:
            seen.add(cmd)
            print(cmd)
")

# Download and extract each binary
FAILED_BINARIES=()

while IFS= read -r binary; do
    ARCHIVE_NAME="${binary}-${TARGET}.tar.xz"
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ARCHIVE_NAME"

    echo "Downloading ${ARCHIVE_NAME}..."
    if ! curl -sfL -o "$TMPDIR/$ARCHIVE_NAME" "$DOWNLOAD_URL"; then
        echo "  Warning: Failed to download ${ARCHIVE_NAME}, skipping" >&2
        FAILED_BINARIES+=("$binary")
        continue
    fi

    if [[ ! -s "$TMPDIR/$ARCHIVE_NAME" ]]; then
        echo "  Warning: Downloaded empty archive for ${binary}, skipping" >&2
        FAILED_BINARIES+=("$binary")
        continue
    fi

    tar -xf "$TMPDIR/$ARCHIVE_NAME" -C "$TMPDIR"

    # cargo-dist extracts into {binary}-{target}/
    EXTRACTED_BIN="$TMPDIR/${binary}-${TARGET}/${binary}"
    if [[ -f "$EXTRACTED_BIN" ]]; then
        cp "$EXTRACTED_BIN" "$ALL_DIR/"
    else
        echo "  Warning: Binary '${binary}' not found after extraction, skipping" >&2
        FAILED_BINARIES+=("$binary")
        continue
    fi
done <<< "$BINARIES"

chmod +x "$ALL_DIR"/*

if [[ "$NON_INTERACTIVE" == "false" ]]; then
    for failed in "${FAILED_BINARIES[@]:-}"; do
        if [[ "$failed" == "multiselect" ]]; then
            echo "Error: multiselect binary failed to download — required for interactive tag selection" >&2
            echo "Use --tags to specify tags non-interactively" >&2
            exit 1
        fi
    done
fi

# Copy tags.json to install directory
cp "$TMPDIR/tags.json" "$INSTALL_DIR/tags.json"

# Detect shell rc file and PATH syntax
MARKER="# bin-tools"

case "$SHELL" in
    */zsh)
        SHELL_RC="$HOME/.zshrc"
        PATH_LINE="export PATH=\"$INSTALL_DIR/enabled:\$PATH\" $MARKER"
        ;;
    */bash)
        SHELL_RC="$HOME/.bashrc"
        PATH_LINE="export PATH=\"$INSTALL_DIR/enabled:\$PATH\" $MARKER"
        ;;
    */fish)
        SHELL_RC="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
        PATH_LINE="set -gx PATH $INSTALL_DIR/enabled \$PATH $MARKER"
        ;;
    *)
        SHELL_RC="$HOME/.profile"
        PATH_LINE="export PATH=\"$INSTALL_DIR/enabled:\$PATH\" $MARKER"
        echo "Warning: Unrecognized shell '$SHELL', falling back to $SHELL_RC" >&2
        ;;
esac

if [[ -f "$SHELL_RC" ]]; then
    TMP_RC="$(mktemp)"
    grep -v "$MARKER" "$SHELL_RC" > "$TMP_RC" || true
    mv "$TMP_RC" "$SHELL_RC"
fi

# Ensure parent directory exists (needed for fish config)
mkdir -p "$(dirname "$SHELL_RC")"

printf '\n%s\n' "$PATH_LINE" >> "$SHELL_RC"
echo "Added $INSTALL_DIR/enabled to PATH in $SHELL_RC"

# Command selection
SELECTED_CMDS=()

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

    # Resolve selected tags to a deduped list of commands
    TAGS_INPUT=$(printf '%s\n' "${SELECTED_TAGS[@]}")
    while IFS= read -r cmd; do
        [[ -n "$cmd" ]] && SELECTED_CMDS+=("$cmd")
    done < <(python3 -c "
import json, sys
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
selected = [t for t in sys.stdin.read().splitlines() if t]
seen = set()
for tag in selected:
    for cmd in tags.get(tag, []):
        if cmd not in seen:
            seen.add(cmd)
            print(cmd)
" <<< "$TAGS_INPUT")
else
    ITEMS_TSV="$TMPDIR/items.tsv"
    python3 - "$INSTALL_DIR/tags.json" > "$ITEMS_TSV" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    tags = json.load(f)
seen = set()
for tag in sorted(tags.keys()):
    cmds = [c for c in tags[tag] if c not in seen]
    if not cmds:
        continue
    print(f"{tag}\t{tag}\t\t1")
    for cmd in cmds:
        seen.add(cmd)
        print(f"{cmd}\t{cmd}\t{tag}\t1")
PY

    SELECTED=$("$ALL_DIR/multiselect" --prompt "Select commands to enable:" < "$ITEMS_TSV") || true

    while IFS= read -r cmd; do
        [[ -n "$cmd" ]] && SELECTED_CMDS+=("$cmd")
    done <<< "$SELECTED"

    # Ensure bin-admin commands are always included
    while IFS= read -r cmd; do
        [[ -z "$cmd" ]] && continue
        FOUND=false
        for s in "${SELECTED_CMDS[@]:-}"; do
            if [[ "$s" == "$cmd" ]]; then
                FOUND=true
                break
            fi
        done
        [[ "$FOUND" == "false" ]] && SELECTED_CMDS+=("$cmd")
    done < <(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
for cmd in tags.get('bin-admin', []):
    print(cmd)
")
fi

# Remove existing symlinks from enabled/ (preserves regular files)
STALE_COUNT=0
for f in "$ENABLED_DIR"/*; do
    if [[ -L "$f" ]]; then
        rm "$f"
        STALE_COUNT=$((STALE_COUNT + 1))
    fi
done

if [[ "$STALE_COUNT" -gt 0 ]]; then
    echo "Cleared $STALE_COUNT existing symlinks from enabled/"
fi

# Create symlinks in enabled/ for selected commands
for cmd in "${SELECTED_CMDS[@]:-}"; do
    if [[ -n "$cmd" && -f "$ALL_DIR/$cmd" && ! -L "$ENABLED_DIR/$cmd" ]]; then
        ln -s "$ALL_DIR/$cmd" "$ENABLED_DIR/$cmd"
        echo "  Enabled: $cmd"
    fi
done

# Print failure summary if any binaries failed
if [[ ${#FAILED_BINARIES[@]} -gt 0 ]]; then
    echo ""
    echo "Warning: The following binaries failed to install:"
    for binary in "${FAILED_BINARIES[@]}"; do
        echo "  - $binary"
    done
fi

echo ""
echo "Installation complete!"
echo "  Binary directory:   $ALL_DIR"
echo "  Enabled directory:  $ENABLED_DIR"
echo ""
echo "Restart your shell or run: source $SHELL_RC"
