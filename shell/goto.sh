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
        upgrade)
            __goto_bin upgrade
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
