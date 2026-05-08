#!/usr/bin/env bash
set -euo pipefail

readonly REPO="12yanogden/bin"
readonly MARKER="# bin-tools"

usage() {
    cat <<EOF
Usage: install.sh [OPTIONS]

Download and install binaries from the latest release.

Options:
    --dir <path>            Set install directory (default: \$HOME/.bin or \$BIN_install_dir)
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
                install_dir="$2"
                shift 2
                ;;
            --tags)
                non_interactive=true
                IFS=',' read -ra selected_tags <<< "$2"
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
    enabled_dir="$install_dir/enabled"
    all_dir="$install_dir/all"
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
        Darwin-arm64|Darwin-aarch64) target="aarch64-apple-darwin" ;;
        Darwin-x86_64)               target="x86_64-apple-darwin" ;;
        Linux-x86_64)                target="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64)               target="aarch64-unknown-linux-gnu" ;;
        *)
            echo "Unsupported platform: ${os}-${arch}" >&2
            exit 1
            ;;
    esac

    echo "Detected platform: ${target}"
}

fetch_latest_tag() {
    echo "Fetching latest release..."
    latest_tag=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [[ -z "$latest_tag" ]]; then
        echo "Failed to determine latest release tag" >&2
        exit 1
    fi

    echo "Latest release: ${latest_tag}"
}

setup_tmpdir() {
    tmpdir="$(mktemp -d)"
    # Expand tmpdir at trap-set time so cleanup works after main() returns
    trap "rm -rf '$tmpdir'" EXIT
}

download_tags_json() {
    local url="https://github.com/$REPO/releases/download/$latest_tag/tags.json"

    echo "Downloading tags.json..."
    curl -sL -o "$tmpdir/tags.json" "$url"
    if [[ ! -s "$tmpdir/tags.json" ]]; then
        echo "Failed to download tags.json" >&2
        exit 1
    fi
}

download_binaries() {
    mkdir -p "$all_dir"
    mkdir -p "$enabled_dir"

    local binaries
    binaries=$(python3 -c "
import json
with open('$tmpdir/tags.json') as f:
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
        archive_name="${binary}-${target}.tar.xz"
        download_url="https://github.com/$REPO/releases/download/$latest_tag/$archive_name"

        echo "Downloading ${archive_name}..."
        if ! curl -sfL -o "$tmpdir/$archive_name" "$download_url"; then
            echo "  Warning: Failed to download ${archive_name}, skipping" >&2
            failed_binaries+=("$binary")
            continue
        fi

        if [[ ! -s "$tmpdir/$archive_name" ]]; then
            echo "  Warning: Downloaded empty archive for ${binary}, skipping" >&2
            failed_binaries+=("$binary")
            continue
        fi

        tar -xf "$tmpdir/$archive_name" -C "$tmpdir"

        # cargo-dist extracts into {binary}-{target}/
        extracted_bin="$tmpdir/${binary}-${target}/${binary}"
        if [[ -f "$extracted_bin" ]]; then
            cp "$extracted_bin" "$all_dir/"
        else
            echo "  Warning: Binary '${binary}' not found after extraction, skipping" >&2
            failed_binaries+=("$binary")
            continue
        fi
    done <<< "$binaries"

    chmod +x "$all_dir"/*
}

ensure_multiselect_available() {
    if [[ "$non_interactive" == "true" ]]; then
        return
    fi

    local failed
    for failed in "${failed_binaries[@]:-}"; do
        if [[ "$failed" == "multiselect" ]]; then
            echo "Error: multiselect binary failed to download — required for interactive tag selection" >&2
            echo "Use --tags to specify tags non-interactively" >&2
            exit 1
        fi
    done
}

install_tags_json() {
    cp "$tmpdir/tags.json" "$install_dir/tags.json"
}

setup_shell_path() {
    case "$SHELL" in
        */zsh)
            shell_rc="$HOME/.zshrc"
            path_line="export PATH=\"$install_dir/enabled:\$PATH\" $MARKER"
            ;;
        */bash)
            shell_rc="$HOME/.bashrc"
            path_line="export PATH=\"$install_dir/enabled:\$PATH\" $MARKER"
            ;;
        */fish)
            shell_rc="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            path_line="set -gx PATH $install_dir/enabled \$PATH $MARKER"
            ;;
        *)
            shell_rc="$HOME/.profile"
            path_line="export PATH=\"$install_dir/enabled:\$PATH\" $MARKER"
            echo "Warning: Unrecognized shell '$SHELL', falling back to $shell_rc" >&2
            ;;
    esac

    if [[ -f "$shell_rc" ]]; then
        local tmp_rc
        tmp_rc="$(mktemp)"
        grep -v "$MARKER" "$shell_rc" > "$tmp_rc" || true
        mv "$tmp_rc" "$shell_rc"
    fi

    # Ensure parent directory exists (needed for fish config)
    mkdir -p "$(dirname "$shell_rc")"

    printf '\n%s\n' "$path_line" >> "$shell_rc"
    echo "Added $install_dir/enabled to PATH in $shell_rc"
}

select_commands() {
    if [[ "$non_interactive" == "true" ]]; then
        select_commands_from_tags
    else
        select_commands_interactive
    fi
}

select_commands_from_tags() {
    # Ensure bin-admin is always included
    local has_bin_admin=false tag
    for tag in "${selected_tags[@]}"; do
        if [[ "$tag" == "bin-admin" ]]; then
            has_bin_admin=true
            break
        fi
    done
    if [[ "$has_bin_admin" == "false" ]]; then
        selected_tags+=("bin-admin")
    fi

    # Resolve selected tags to a deduped list of commands
    local tags_input cmd
    tags_input=$(printf '%s\n' "${selected_tags[@]}")
    while IFS= read -r cmd; do
        [[ -n "$cmd" ]] && selected_cmds+=("$cmd")
    done < <(python3 -c "
import json, sys
with open('$install_dir/tags.json') as f:
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
    local items_tsv="$tmpdir/items.tsv"
    python3 - "$install_dir/tags.json" > "$items_tsv" <<'PY'
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
    selected=$("$all_dir/multiselect" --prompt "Select commands to enable:" < "$items_tsv") || true

    local cmd
    while IFS= read -r cmd; do
        [[ -n "$cmd" ]] && selected_cmds+=("$cmd")
    done <<< "$selected"

    # Ensure bin-admin commands are always included
    local found s
    while IFS= read -r cmd; do
        [[ -z "$cmd" ]] && continue
        found=false
        for s in "${selected_cmds[@]:-}"; do
            if [[ "$s" == "$cmd" ]]; then
                found=true
                break
            fi
        done
        [[ "$found" == "false" ]] && selected_cmds+=("$cmd")
    done < <(python3 -c "
import json
with open('$install_dir/tags.json') as f:
    tags = json.load(f)
for cmd in tags.get('bin-admin', []):
    print(cmd)
")
}

refresh_enabled_symlinks() {
    # Remove existing symlinks from enabled/ (preserves regular files)
    local stale_count=0 f
    for f in "$enabled_dir"/*; do
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
    for cmd in "${selected_cmds[@]:-}"; do
        if [[ -n "$cmd" && -f "$all_dir/$cmd" && ! -L "$enabled_dir/$cmd" ]]; then
            ln -s "$all_dir/$cmd" "$enabled_dir/$cmd"
            echo "  Enabled: $cmd"
        fi
    done
}

print_summary() {
    if [[ ${#failed_binaries[@]} -gt 0 ]]; then
        echo ""
        echo "Warning: The following binaries failed to install:"
        local binary
        for binary in "${failed_binaries[@]}"; do
            echo "  - $binary"
        done
    fi

    echo ""
    echo "Installation complete!"
    echo "  Binary directory:   $all_dir"
    echo "  Enabled directory:  $enabled_dir"
    echo ""
    echo "Restart your shell or run: source $shell_rc"
}

main() {
    # Shared state — locals in main() are visible to helpers via dynamic scope
    local install_dir="${BIN_install_dir:-$HOME/.bin}"
    local enabled_dir=""
    local all_dir=""
    local non_interactive=false
    local selected_tags=()
    local selected_cmds=()
    local failed_binaries=()
    local tmpdir=""
    local target=""
    local latest_tag=""
    local shell_rc=""
    local path_line=""

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
