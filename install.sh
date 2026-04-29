#!/usr/bin/env bash
set -euo pipefail

REPO="${GOTO_REPO:-immanuel-peter/goto}"
INSTALL_DIR="${GOTO_INSTALL_DIR:-$HOME/.local/bin}"
SHELL_DIR="${GOTO_SHELL_DIR:-$HOME/.config/goto}"
BIN_NAME="__goto_bin"
TMP_FILE=""

info() {
    printf '%s\n' "$1"
}

fail() {
    printf 'goto: error: %s\n' "$1" >&2
    exit 1
}

detect_artifact() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os:$arch" in
        Darwin:arm64) printf 'goto-macos-aarch64' ;;
        Darwin:x86_64) printf 'goto-macos-x86_64' ;;
        Linux:x86_64) printf 'goto-linux-x86_64' ;;
        Linux:aarch64 | Linux:arm64) printf 'goto-linux-aarch64' ;;
        *) fail "unsupported platform: $os $arch" ;;
    esac
}

download() {
    local url destination
    url="$1"
    destination="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$destination"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$destination"
    else
        fail "curl or wget is required"
    fi
}

write_shell_wrapper() {
    mkdir -p "$SHELL_DIR"
    cat > "$SHELL_DIR/goto.sh" <<'EOF'
goto() {
    if [ "$#" -eq 0 ]; then
        __goto_bin list
        return
    fi

    case "$1" in
        add)
            __goto_bin add "${2-}" "${3:-.}"
            ;;
        remove|rm)
            __goto_bin remove "${2-}"
            ;;
        list|ls)
            __goto_bin list
            ;;
        help|-h|--help)
            __goto_bin --help
            ;;
        version|-V|--version)
            __goto_bin --version
            ;;
        *)
            local target
            target=$(__goto_bin resolve "$1") || return 1
            cd "$target" || return 1
            ;;
    esac
}
EOF
}

write_fish_wrapper() {
    local fish_dir
    fish_dir="$HOME/.config/fish"

    if [ ! -d "$fish_dir" ]; then
        return
    fi

    mkdir -p "$fish_dir/functions"
    cat > "$fish_dir/functions/goto.fish" <<'EOF'
function goto
    if test (count $argv) -eq 0
        __goto_bin list
        return $status
    end

    switch $argv[1]
        case add
            set -l dir .
            if test (count $argv) -ge 3
                set dir $argv[3]
            end
            __goto_bin add $argv[2] $dir
        case remove rm
            __goto_bin remove $argv[2]
        case list ls
            __goto_bin list
        case help -h --help
            __goto_bin --help
        case version -V --version
            __goto_bin --version
        case '*'
            set -l target (__goto_bin resolve $argv[1])
            or return 1
            cd "$target"
            or return 1
    end
end
EOF
}

source_line() {
    if [ -z "${GOTO_SHELL_DIR:-}" ]; then
        printf '%s\n' '[ -f "$HOME/.config/goto/goto.sh" ] && source "$HOME/.config/goto/goto.sh"'
    else
        printf '[ -f "%s/goto.sh" ] && source "%s/goto.sh"\n' "$SHELL_DIR" "$SHELL_DIR"
    fi
}

inject_shell_source() {
    local line rc
    line="$(source_line)"

    for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile"; do
        if [ ! -f "$rc" ]; then
            continue
        fi

        if ! grep -qF "$line" "$rc"; then
            {
                printf '\n# goto shell integration\n'
                printf '%s\n' "$line"
            } >> "$rc"
        fi
    done
}

main() {
    local artifact url
    artifact="$(detect_artifact)"
    url="https://github.com/$REPO/releases/latest/download/$artifact"
    TMP_FILE="$(mktemp)"
    trap 'rm -f "${TMP_FILE:-}"' EXIT

    mkdir -p "$INSTALL_DIR"
    info "Downloading for ${artifact#goto-}"
    download "$url" "$TMP_FILE"

    chmod +x "$TMP_FILE"
    mv "$TMP_FILE" "$INSTALL_DIR/$BIN_NAME"
    TMP_FILE=""

    write_shell_wrapper
    write_fish_wrapper
    inject_shell_source

    info "Installation complete"
    info "Restart shell or source $SHELL_DIR/goto.sh"
}

main "$@"
