#!/usr/bin/env bash
set -euo pipefail

readonly REPO="12yanogden/bin"

# ----------------------------------------------------------------------------
# Available commands. The interactive picker pre-selects any already
# installed in the target directory.
# ----------------------------------------------------------------------------
COMMANDS=(
    "arr-intersect"
    "arr-subtract"
    "bs"
    "cb"
    "cmt"
    "dot-env"
    "fail"
    "is-dirty"
    "is-on-branch"
    "multiselect"
    "pass"
    "pwb"
    "x"
)

usage() {
    cat <<EOF
Usage: install.sh [OPTIONS]

Download and install selected binaries from the latest release into a
directory already on PATH (default: /usr/local/bin).

An interactive picker is shown; commands already present in the install
directory are pre-selected.

Options:
    --dir <path>    Install directory (default: /usr/local/bin or \$BIN_install_dir)
    -h, --help      Show this help message
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
            --help|-h)
                usage
                ;;
            *)
                echo "Unknown argument: $1" >&2
                exit 1
                ;;
        esac
    done
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}-${arch}" in
        Darwin-arm64|Darwin-aarch64) target="aarch64-apple-darwin" ;;
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
    trap "rm -rf '$tmpdir'" EXIT
}

fetch_binary() {
    # Downloads and extracts a release binary into tmpdir, echoing its path.
    local binary="$1"
    local archive_name="${binary}-${target}.tar.xz"
    local download_url="https://github.com/$REPO/releases/download/$latest_tag/$archive_name"

    if ! curl -sfL -o "$tmpdir/$archive_name" "$download_url"; then
        return 1
    fi
    [[ -s "$tmpdir/$archive_name" ]] || return 1

    tar -xf "$tmpdir/$archive_name" -C "$tmpdir" || return 1

    local extracted="$tmpdir/${binary}-${target}/${binary}"
    [[ -f "$extracted" ]] || return 1

    chmod +x "$extracted"
    printf '%s\n' "$extracted"
}

bootstrap_multiselect() {
    echo "Fetching multiselect for interactive picker..."
    if ! multiselect_bin="$(fetch_binary multiselect)"; then
        echo "Failed to download multiselect from release ${latest_tag}." >&2
        exit 1
    fi
}

pick_commands() {
    local name selected tsv=""
    for name in "${COMMANDS[@]}"; do
        if [[ -f "$install_dir/$name" ]]; then
            selected=1
            pre_installed_cmds+=("$name")
        else
            selected=0
        fi
        tsv+="${name}\t\t\t${selected}"$'\n'
    done

    local picked
    if ! picked=$(printf '%b' "$tsv" | "$multiselect_bin" --prompt "Select commands to install (unchecking an installed command will uninstall it):"); then
        echo "Cancelled." >&2
        exit 1
    fi

    while IFS= read -r name; do
        [[ -n "$name" ]] && enabled_cmds+=("$name")
    done <<<"$picked"

    # Anything previously installed but not picked this time → uninstall.
    local prev picked_name found
    if (( ${#pre_installed_cmds[@]} > 0 )); then
        for prev in "${pre_installed_cmds[@]}"; do
            found=0
            if (( ${#enabled_cmds[@]} > 0 )); then
                for picked_name in "${enabled_cmds[@]}"; do
                    if [[ "$prev" == "$picked_name" ]]; then
                        found=1
                        break
                    fi
                done
            fi
            [[ $found -eq 0 ]] && cmds_to_remove+=("$prev")
        done
    fi

    if [[ ${#enabled_cmds[@]} -eq 0 && ${#cmds_to_remove[@]} -eq 0 ]]; then
        echo "Nothing to do." >&2
        exit 1
    fi
}

prepare_install_dir() {
    if [[ -d "$install_dir" && -w "$install_dir" ]]; then
        sudo_cmd=""
    elif [[ -d "$install_dir" ]]; then
        sudo_cmd="sudo"
        echo "Note: $install_dir is not writable by current user; sudo will be used."
    else
        if mkdir -p "$install_dir" 2>/dev/null; then
            sudo_cmd=""
        else
            sudo_cmd="sudo"
            echo "Note: creating $install_dir requires sudo."
            sudo mkdir -p "$install_dir"
        fi
    fi
}

install_one() {
    local binary="$1"
    local extracted_bin

    echo "Downloading ${binary}-${target}.tar.xz..."
    if ! extracted_bin="$(fetch_binary "$binary")"; then
        echo "  Warning: Failed to fetch ${binary}, skipping" >&2
        failed_binaries+=("$binary")
        return
    fi

    $sudo_cmd install -m 0755 "$extracted_bin" "$install_dir/$binary"
    installed_binaries+=("$binary")
}

install_binaries() {
    local binary prev already
    (( ${#enabled_cmds[@]} > 0 )) || return 0
    for binary in "${enabled_cmds[@]}"; do
        already=0
        if (( ${#pre_installed_cmds[@]} > 0 )); then
            for prev in "${pre_installed_cmds[@]}"; do
                if [[ "$prev" == "$binary" ]]; then
                    already=1
                    break
                fi
            done
        fi
        if [[ $already -eq 1 ]]; then
            skipped_binaries+=("$binary")
            continue
        fi
        install_one "$binary"
    done
}

uninstall_one() {
    local binary="$1"
    local path="$install_dir/$binary"

    if [[ ! -e "$path" ]]; then
        return
    fi

    if $sudo_cmd rm -f "$path"; then
        removed_binaries+=("$binary")
    else
        echo "  Warning: Failed to remove $path" >&2
        failed_removals+=("$binary")
    fi
}

uninstall_binaries() {
    local binary
    (( ${#cmds_to_remove[@]} > 0 )) || return 0
    for binary in "${cmds_to_remove[@]}"; do
        echo "Removing $binary..."
        uninstall_one "$binary"
    done
}

install_bin_alias() {
    local bashrc="$HOME/.bashrc"
    local marker="# bin installer alias (12yanogden/bin)"

    if [[ -f "$bashrc" ]] && grep -Fq "$marker" "$bashrc"; then
        return 0
    fi

    cat >> "$bashrc" <<'EOF'

# bin installer alias (12yanogden/bin)
alias bin="bash -c \"\$(curl --proto '=https' --tlsv1.2 -fsSL https://github.com/12yanogden/bin/releases/latest/download/install.sh)\""
EOF

    bashrc_updated=1
}

print_summary() {
    echo ""
    if [[ ${#installed_binaries[@]} -gt 0 ]]; then
        echo "Installed to $install_dir:"
        local b
        for b in "${installed_binaries[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ ${#skipped_binaries[@]} -gt 0 ]]; then
        echo ""
        echo "Already installed (skipped):"
        local b
        for b in "${skipped_binaries[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ ${#removed_binaries[@]} -gt 0 ]]; then
        echo ""
        echo "Removed from $install_dir:"
        local b
        for b in "${removed_binaries[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ ${#failed_binaries[@]} -gt 0 ]]; then
        echo ""
        echo "Failed to install:"
        local b
        for b in "${failed_binaries[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ ${#failed_removals[@]} -gt 0 ]]; then
        echo ""
        echo "Failed to remove:"
        local b
        for b in "${failed_removals[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ ${bashrc_updated:-0} -eq 1 ]]; then
        echo ""
        echo "Added 'bin' alias to ~/.bashrc. Run 'source ~/.bashrc' or open a new shell to use it."
    fi

    echo ""
    echo "Done."

    case ":$PATH:" in
        *":$install_dir:"*)
            ;;
        *)
            echo ""
            echo "Note: $install_dir is not on your PATH. Add it to your shell config to use the installed commands."
            ;;
    esac
}

main() {
    # Shared state — locals in main() are visible to helpers via dynamic scope
    local install_dir="${BIN_install_dir:-/usr/local/bin}"
    local enabled_cmds=()
    local pre_installed_cmds=()
    local cmds_to_remove=()
    local installed_binaries=()
    local skipped_binaries=()
    local removed_binaries=()
    local failed_binaries=()
    local failed_removals=()
    local tmpdir=""
    local target=""
    local latest_tag=""
    local sudo_cmd=""
    local multiselect_bin=""
    local bashrc_updated=0

    parse_args "$@"
    detect_target
    fetch_latest_tag
    setup_tmpdir
    bootstrap_multiselect
    pick_commands
    prepare_install_dir
    install_binaries
    uninstall_binaries
    install_bin_alias
    print_summary
}

main "$@"
