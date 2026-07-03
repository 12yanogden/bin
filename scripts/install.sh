#!/usr/bin/env bash
set -euo pipefail

readonly DEFAULT_REPO="12yanogden/bin"

# ----------------------------------------------------------------------------
# Available commands. The interactive picker pre-selects any already
# installed in the target directory.
# ----------------------------------------------------------------------------
COMMANDS=(
    "arr-intersect"
    "arr-subtract"
    "auto-archive"
    "bs"
    "cb"
    "cmt"
    "cronx"
    "dot-env"
    "fail"
    "is-dirty"
    "is-on-branch"
    "multiselect"
    "pass"
    "pwb"
    "umoria"
    "x"
)

# Source-of-truth repository for each binary. Binaries owned by other
# projects ship their own releases; everything else falls through to bin.
# Implemented as a case statement because macOS ships bash 3.2, which has
# no associative arrays.
repo_for() {
    case "$1" in
        multiselect)   printf '%s\n' "Artemis-Cooperative/multiselect-cli" ;;
        x)             printf '%s\n' "Artemis-Cooperative/shell-executor" ;;
        auto-archive)  printf '%s\n' "12yanogden/auto-archive" ;;
        cronx)         printf '%s\n' "12yanogden/cronx" ;;
        umoria)        printf '%s\n' "12yanogden/umoria" ;;
        *)             printf '%s\n' "$DEFAULT_REPO" ;;
    esac
}

# cargo-dist names archives by Cargo package name, not binary name. Defaults
# to the binary name; override here when the source package name differs.
archive_prefix_for() {
    case "$1" in
        multiselect) printf '%s\n' "multiselect-cli" ;;
        x)           printf '%s\n' "shell-executor" ;;
        *)           printf '%s\n' "$1" ;;
    esac
}

# Per-repo cache of latest release tags. Different source repos are on
# different version tracks, so there is no single global "latest".
# Parallel indexed arrays — also a concession to bash 3.2.
TAG_CACHE_REPOS=()
TAG_CACHE_TAGS=()

tag_for() {
    local repo="$1"
    local i tag
    if [[ ${#TAG_CACHE_REPOS[@]} -gt 0 ]]; then
        for i in "${!TAG_CACHE_REPOS[@]}"; do
            if [[ "${TAG_CACHE_REPOS[$i]}" == "$repo" ]]; then
                printf '%s\n' "${TAG_CACHE_TAGS[$i]}"
                return 0
            fi
        done
    fi

    echo "Fetching latest release for ${repo}..." >&2
    tag=$(curl -sL "https://api.github.com/repos/$repo/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"([^"]+)".*/\1/')

    if [[ -z "$tag" ]]; then
        echo "Failed to determine latest release tag for ${repo}" >&2
        return 1
    fi

    echo "  Latest release for ${repo}: ${tag}" >&2
    TAG_CACHE_REPOS+=("$repo")
    TAG_CACHE_TAGS+=("$tag")
    printf '%s\n' "$tag"
}

usage() {
    cat <<EOF
Usage: install.sh [COMMAND] [OPTIONS]

Download and install selected binaries from the latest release into a
directory already on PATH (default: /usr/local/bin).

Commands:
    update          Re-download every installed binary to its latest release
                    (no interactive picker)

With no command, an interactive picker is shown; commands already present
in the install directory are pre-selected.

Options:
    --dir <path>    Install directory (default: /usr/local/bin or \$BIN_install_dir)
    -h, --help      Show this help message
EOF
    exit 0
}

parse_args() {
    UPDATE_MODE=0
    while [[ $# -gt 0 ]]; do
        case "$1" in
            update)
                UPDATE_MODE=1
                shift
                ;;
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

setup_tmpdir() {
    tmpdir="$(mktemp -d)"
    trap "rm -rf '$tmpdir'" EXIT
}

fetch_binary() {
    # Downloads and extracts a release binary into tmpdir, echoing its path.
    # Routes through repo_for() so each binary is pulled from its own
    # source repo's latest release.
    local binary="$1"
    local repo tag prefix
    repo="$(repo_for "$binary")"
    if ! tag="$(tag_for "$repo")"; then
        return 1
    fi
    prefix="$(archive_prefix_for "$binary")"
    local archive_name="${prefix}-${target}.tar.xz"
    local download_url="https://github.com/$repo/releases/download/$tag/$archive_name"

    if ! curl -sfL -o "$tmpdir/$archive_name" "$download_url"; then
        return 1
    fi
    [[ -s "$tmpdir/$archive_name" ]] || return 1

    tar -xf "$tmpdir/$archive_name" -C "$tmpdir" || return 1

    local extracted="$tmpdir/${prefix}-${target}/${binary}"
    [[ -f "$extracted" ]] || return 1

    chmod +x "$extracted"
    printf '%s\n' "$extracted"
}

bootstrap_multiselect() {
    echo "Fetching multiselect for interactive picker..."
    if ! multiselect_bin="$(fetch_binary multiselect)"; then
        echo "Failed to download multiselect for the interactive picker." >&2
        exit 1
    fi
}

bootstrap_x() {
    # x (shell-executor) is the wrapper used to keep subsequent commands'
    # output out of stdout on success. Has to be fetched raw — it cannot
    # wrap its own download.
    echo "Fetching x for command wrapping..."
    if ! x_bin="$(fetch_binary x)"; then
        echo "Failed to download x for command wrapping." >&2
        exit 1
    fi
}

discover_installed_commands() {
    local name
    for name in "${COMMANDS[@]}" bin; do
        if [[ -f "$install_dir/$name" ]]; then
            enabled_cmds+=("$name")
        fi
    done
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

# ----------------------------------------------------------------------------
# Per-command lifecycle hooks
#
# Define a function named `post_install_<binary>` (hyphens → underscores) to
# run a setup step after a successful install, or `pre_uninstall_<binary>` to
# run a teardown step before removal. Hooks receive no arguments; the binary
# is available at "$install_dir/<binary>" when they fire.
# ----------------------------------------------------------------------------

post_install_cronx() {
    "$install_dir/cronx" --setup
}

pre_uninstall_cronx() {
    "$install_dir/cronx" --takedown
}

run_hook() {
    # Runs a single hook function under x so its output only surfaces on
    # failure. The hook function itself is a shell function in this script;
    # `bash -c "$fn"` works because we export the function below.
    local kind="$1" binary="$2" fn="$3"
    "$x_bin" --quiet --msg "${kind} ${binary}" "bash -c '$fn'" \
        || failed_hooks+=("${binary} (${kind})")
}

run_pre_uninstall_hooks() {
    local binary fn
    (( ${#cmds_to_remove[@]} > 0 )) || return 0
    export install_dir sudo_cmd
    for binary in "${cmds_to_remove[@]}"; do
        fn="pre_uninstall_${binary//-/_}"
        declare -F "$fn" >/dev/null 2>&1 || continue
        export -f "$fn"
        run_hook "pre-uninstall" "$binary" "$fn"
    done
}

run_post_install_hooks() {
    local binary fn
    (( ${#installed_binaries[@]} > 0 )) || return 0
    export install_dir sudo_cmd
    for binary in "${installed_binaries[@]}"; do
        fn="post_install_${binary//-/_}"
        declare -F "$fn" >/dev/null 2>&1 || continue
        export -f "$fn"
        run_hook "post-install" "$binary" "$fn"
    done
}

install_binaries() {
    # Phases:
    #   1. Partition enabled_cmds into already-installed (skipped) vs to_install,
    #      unless force=1 (update mode), in which case everything is re-fetched.
    #   2. Resolve release tags serially in the parent so the tag cache is
    #      warm and the API isn't hit concurrently by parallel children.
    #   3. Prime sudo so parallel children don't trip a hidden password prompt.
    #   4. Spawn one x --parallel child per binary running the full
    #      fetch → extract → chmod → install pipeline.
    #   5. Post-check which files actually landed to populate
    #      installed_binaries / failed_binaries.
    local force="${1:-0}"
    local binary prev already
    (( ${#enabled_cmds[@]} > 0 )) || return 0

    local to_install=()
    for binary in "${enabled_cmds[@]}"; do
        if [[ "$force" -eq 1 ]]; then
            to_install+=("$binary")
            continue
        fi
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
        else
            to_install+=("$binary")
        fi
    done

    (( ${#to_install[@]} > 0 )) || return 0

    local repo tag prefix archive url extracted
    local cmds=()
    for binary in "${to_install[@]}"; do
        repo="$(repo_for "$binary")"
        if ! tag="$(tag_for "$repo")"; then
            failed_binaries+=("$binary")
            continue
        fi
        prefix="$(archive_prefix_for "$binary")"
        archive="${prefix}-${target}.tar.xz"
        url="https://github.com/$repo/releases/download/$tag/$archive"
        extracted="$tmpdir/${prefix}-${target}/${binary}"
        # Single shell string per child; chained && short-circuits so any
        # failing step fails the child. -f makes curl exit non-zero on HTTP
        # errors (instead of saving the error page), -s silences progress,
        # -L follows redirects.
        cmds+=("curl -sfL -o '$tmpdir/$archive' '$url' && [ -s '$tmpdir/$archive' ] && tar -xf '$tmpdir/$archive' -C '$tmpdir' && [ -f '$extracted' ] && chmod +x '$extracted' && $sudo_cmd install -m 0755 '$extracted' '$install_dir/$binary'")
    done

    (( ${#cmds[@]} > 0 )) || return 0

    [[ -n "$sudo_cmd" ]] && sudo -v

    local action="Installing"
    [[ "$force" -eq 1 ]] && action="Updating"
    "$x_bin" --msg "${action} ${#cmds[@]} binar$([ ${#cmds[@]} -eq 1 ] && echo y || echo ies)" \
        --quiet --parallel "${cmds[@]}" || true

    # Authoritative truth is the filesystem, not x's aggregate exit code.
    for binary in "${to_install[@]}"; do
        if [[ -x "$install_dir/$binary" ]]; then
            installed_binaries+=("$binary")
        else
            # Skip if we already recorded the failure during tag resolution.
            already=0
            if (( ${#failed_binaries[@]} > 0 )); then
                for prev in "${failed_binaries[@]}"; do
                    [[ "$prev" == "$binary" ]] && { already=1; break; }
                done
            fi
            [[ $already -eq 0 ]] && failed_binaries+=("$binary")
        fi
    done
}

uninstall_binaries() {
    local binary
    (( ${#cmds_to_remove[@]} > 0 )) || return 0

    local to_remove=()
    for binary in "${cmds_to_remove[@]}"; do
        [[ -e "$install_dir/$binary" ]] && to_remove+=("$binary")
    done

    (( ${#to_remove[@]} > 0 )) || return 0

    [[ -n "$sudo_cmd" ]] && sudo -v

    local cmds=()
    for binary in "${to_remove[@]}"; do
        cmds+=("$sudo_cmd rm -f '$install_dir/$binary'")
    done

    "$x_bin" --msg "Removing ${#cmds[@]} binar$([ ${#cmds[@]} -eq 1 ] && echo y || echo ies)" \
        --quiet --parallel "${cmds[@]}" || true

    for binary in "${to_remove[@]}"; do
        if [[ ! -e "$install_dir/$binary" ]]; then
            removed_binaries+=("$binary")
        else
            failed_removals+=("$binary")
        fi
    done
}

ensure_bin_command() {
    # The bin command re-runs this installer; always keep it installed.
    local name prev found=0 kept=()

    for name in "${enabled_cmds[@]}"; do
        [[ "$name" == "bin" ]] && found=1
    done
    [[ $found -eq 1 ]] || enabled_cmds+=("bin")

    for prev in "${cmds_to_remove[@]}"; do
        [[ "$prev" == "bin" ]] && continue
        kept+=("$prev")
    done
    cmds_to_remove=("${kept[@]}")
}

cleanup_downloaded_script() {
    # Remove install.sh when it was downloaded to the current directory.
    # Skip pipe/bash -c invocations, which never write a file to disk.
    local script="${BASH_SOURCE[0]:-$0}"

    case "$script" in
        bash|/bin/bash|/usr/bin/bash|/dev/fd/*|/dev/stdin)
            return 0
            ;;
    esac

    [[ "$(basename "$script")" == "install.sh" ]] || return 0
    [[ -f "$script" ]] || return 0

    local script_dir
    script_dir="$(cd "$(dirname "$script")" && pwd)"
    [[ "$script_dir" == "$PWD" ]] || return 0

    rm -f "$script"
}

print_summary() {
    echo ""
    if [[ ${#installed_binaries[@]} -gt 0 ]]; then
        if [[ "${UPDATE_MODE:-0}" -eq 1 ]]; then
            echo "Updated in $install_dir:"
        else
            echo "Installed to $install_dir:"
        fi
        local b
        for b in "${installed_binaries[@]}"; do
            echo "  - $b"
        done
    fi

    if [[ "${UPDATE_MODE:-0}" -eq 1 ]]; then
        :
    elif [[ ${#skipped_binaries[@]} -gt 0 ]]; then
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
        if [[ "${UPDATE_MODE:-0}" -eq 1 ]]; then
            echo "Failed to update:"
        else
            echo "Failed to install:"
        fi
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

    if [[ ${#failed_hooks[@]} -gt 0 ]]; then
        echo ""
        echo "Hooks that failed (binary state is otherwise unchanged):"
        local b
        for b in "${failed_hooks[@]}"; do
            echo "  - $b"
        done
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
    local UPDATE_MODE=0
    local enabled_cmds=()
    local pre_installed_cmds=()
    local cmds_to_remove=()
    local installed_binaries=()
    local skipped_binaries=()
    local removed_binaries=()
    local failed_binaries=()
    local failed_removals=()
    local failed_hooks=()
    local tmpdir=""
    local target=""
    local sudo_cmd=""
    local multiselect_bin=""
    local x_bin=""

    parse_args "$@"
    detect_target
    setup_tmpdir

    if [[ "$UPDATE_MODE" -eq 1 ]]; then
        bootstrap_x
        discover_installed_commands
        if [[ ${#enabled_cmds[@]} -eq 0 ]]; then
            echo "No installed binaries found in $install_dir." >&2
            exit 1
        fi
        prepare_install_dir
        install_binaries 1
        run_post_install_hooks
        print_summary
        cleanup_downloaded_script
        return 0
    fi

    bootstrap_multiselect
    bootstrap_x
    pick_commands
    ensure_bin_command
    prepare_install_dir
    run_pre_uninstall_hooks
    uninstall_binaries
    install_binaries
    run_post_install_hooks
    print_summary
    cleanup_downloaded_script
}

main "$@"
