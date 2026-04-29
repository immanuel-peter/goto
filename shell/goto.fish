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
        case upgrade
            __goto_bin upgrade
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
