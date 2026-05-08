#!/usr/bin/env bash
set -euo pipefail

readonly REPO="12yanogden/bin"
readonly MARKER="# bin-tools"

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

parse_args() {
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
}

check_dependencies() {
    if ! command -v python3 &>/dev/null; then
        echo "Error: python3 is required but not installed" >&2
        exit 1
    fi
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}-${arch}" in
        Darwin-arm64|Darwin-aarch64) TARGET="aarch64-apple-darwin" ;;
        Darwin-x86_64)               TARGET="x86_64-apple-darwin" ;;
        Linux-x86_64)                TARGET="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64)               TARGET="aarch64-unknown-linux-gnu" ;;
        *)
            echo "Unsupported platform: ${os}-${arch}" >&2
            exit 1
            ;;
    esac

    echo "Detected platform: ${TARGET}"
}

fetch_latest_tag() {
    echo "Fetching latest release..."
    LATEST_TAG=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [[ -z "$LATEST_TAG" ]]; then
        echo "Failed to determine latest release tag" >&2
        exit 1
    fi

    echo "Latest release: ${LATEST_TAG}"
}

setup_tmpdir() {
    TMPDIR="$(mktemp -d)"
    # Expand TMPDIR at trap-set time so cleanup works after main() returns
    trap "rm -rf '$TMPDIR'" EXIT
}

download_tags_json() {
    local url="https://github.com/$REPO/releases/download/$LATEST_TAG/tags.json"

    echo "Downloading tags.json..."
    curl -sL -o "$TMPDIR/tags.json" "$url"
    if [[ ! -s "$TMPDIR/tags.json" ]]; then
        echo "Failed to download tags.json" >&2
        exit 1
    fi
}

download_binaries() {
    mkdir -p "$ALL_DIR"
    mkdir -p "$ENABLED_DIR"

    local binaries
    binaries=$(python3 -c "
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

    local binary archive_name download_url extracted_bin
    while IFS= read -r binary; do
        archive_name="${binary}-${TARGET}.tar.xz"
        download_url="https://github.com/$REPO/releases/download/$LATEST_TAG/$archive_name"

        echo "Downloading ${archive_name}..."
        if ! curl -sfL -o "$TMPDIR/$archive_name" "$download_url"; then
            echo "  Warning: Failed to download ${archive_name}, skipping" >&2
            FAILED_BINARIES+=("$binary")
            continue
        fi

        if [[ ! -s "$TMPDIR/$archive_name" ]]; then
            echo "  Warning: Downloaded empty archive for ${binary}, skipping" >&2
            FAILED_BINARIES+=("$binary")
            continue
        fi

        tar -xf "$TMPDIR/$archive_name" -C "$TMPDIR"

        # cargo-dist extracts into {binary}-{target}/
        extracted_bin="$TMPDIR/${binary}-${TARGET}/${binary}"
        if [[ -f "$extracted_bin" ]]; then
            cp "$extracted_bin" "$ALL_DIR/"
        else
            echo "  Warning: Binary '${binary}' not found after extraction, skipping" >&2
            FAILED_BINARIES+=("$binary")
            continue
        fi
    done <<< "$binaries"

    chmod +x "$ALL_DIR"/*
}

ensure_multiselect_available() {
    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        return
    fi

    local failed
    for failed in "${FAILED_BINARIES[@]:-}"; do
        if [[ "$failed" == "multiselect" ]]; then
            echo "Error: multiselect binary failed to download — required for interactive tag selection" >&2
            echo "Use --tags to specify tags non-interactively" >&2
            exit 1
        fi
    done
}

install_tags_json() {
    cp "$TMPDIR/tags.json" "$INSTALL_DIR/tags.json"
}

setup_shell_path() {
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
        local tmp_rc
        tmp_rc="$(mktemp)"
        grep -v "$MARKER" "$SHELL_RC" > "$tmp_rc" || true
        mv "$tmp_rc" "$SHELL_RC"
    fi

    # Ensure parent directory exists (needed for fish config)
    mkdir -p "$(dirname "$SHELL_RC")"

    printf '\n%s\n' "$PATH_LINE" >> "$SHELL_RC"
    echo "Added $INSTALL_DIR/enabled to PATH in $SHELL_RC"
}

select_commands() {
    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        select_commands_from_tags
    else
        select_commands_interactive
    fi
}

select_commands_from_tags() {
    # Ensure bin-admin is always included
    local has_bin_admin=false tag
    for tag in "${SELECTED_TAGS[@]}"; do
        if [[ "$tag" == "bin-admin" ]]; then
            has_bin_admin=true
            break
        fi
    done
    if [[ "$has_bin_admin" == "false" ]]; then
        SELECTED_TAGS+=("bin-admin")
    fi

    # Resolve selected tags to a deduped list of commands
    local tags_input cmd
    tags_input=$(printf '%s\n' "${SELECTED_TAGS[@]}")
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
" <<< "$tags_input")
}

select_commands_interactive() {
    local items_tsv="$TMPDIR/items.tsv"
    python3 - "$INSTALL_DIR/tags.json" > "$items_tsv" <<'PY'
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

    local selected
    selected=$("$ALL_DIR/multiselect" --prompt "Select commands to enable:" < "$items_tsv") || true

    local cmd
    while IFS= read -r cmd; do
        [[ -n "$cmd" ]] && SELECTED_CMDS+=("$cmd")
    done <<< "$selected"

    # Ensure bin-admin commands are always included
    local found s
    while IFS= read -r cmd; do
        [[ -z "$cmd" ]] && continue
        found=false
        for s in "${SELECTED_CMDS[@]:-}"; do
            if [[ "$s" == "$cmd" ]]; then
                found=true
                break
            fi
        done
        [[ "$found" == "false" ]] && SELECTED_CMDS+=("$cmd")
    done < <(python3 -c "
import json
with open('$INSTALL_DIR/tags.json') as f:
    tags = json.load(f)
for cmd in tags.get('bin-admin', []):
    print(cmd)
")
}

refresh_enabled_symlinks() {
    # Remove existing symlinks from enabled/ (preserves regular files)
    local stale_count=0 f
    for f in "$ENABLED_DIR"/*; do
        if [[ -L "$f" ]]; then
            rm "$f"
            stale_count=$((stale_count + 1))
        fi
    done

    if [[ "$stale_count" -gt 0 ]]; then
        echo "Cleared $stale_count existing symlinks from enabled/"
    fi

    # Create symlinks in enabled/ for selected commands
    local cmd
    for cmd in "${SELECTED_CMDS[@]:-}"; do
        if [[ -n "$cmd" && -f "$ALL_DIR/$cmd" && ! -L "$ENABLED_DIR/$cmd" ]]; then
            ln -s "$ALL_DIR/$cmd" "$ENABLED_DIR/$cmd"
            echo "  Enabled: $cmd"
        fi
    done
}

print_summary() {
    if [[ ${#FAILED_BINARIES[@]} -gt 0 ]]; then
        echo ""
        echo "Warning: The following binaries failed to install:"
        local binary
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
}

main() {
    # Shared state — locals in main() are visible to helpers via dynamic scope
    local INSTALL_DIR="${BIN_INSTALL_DIR:-$HOME/.bin}"
    local ENABLED_DIR=""
    local ALL_DIR=""
    local NON_INTERACTIVE=false
    local SELECTED_TAGS=()
    local SELECTED_CMDS=()
    local FAILED_BINARIES=()
    local TMPDIR=""
    local TARGET=""
    local LATEST_TAG=""
    local SHELL_RC=""
    local PATH_LINE=""

    parse_args "$@"
    check_dependencies
    detect_target
    fetch_latest_tag
    setup_tmpdir
    download_tags_json
    download_binaries
    ensure_multiselect_available
    install_tags_json
    setup_shell_path
    select_commands
    refresh_enabled_symlinks
    print_summary
}

main "$@"
