#!/usr/bin/env bash

set -e

# @cmd Test the project
# @alias t
test() {
    cargo test $@
}

# @cmd Check the project
# @alias c
check() {
    cargo fmt --all --check
    cargo clippy --all
    cargo test
}

# @cmd Fix the project
# @alias f
fix() {
    cargo fmt --all
    cargo clippy --fix --all --allow-dirty
}

# @cmd Code for setup shell to load argc completion script
# @option -s --shell[=bash|elvish|fish|nushell|powershell|xonsh|zsh] shell type
# @arg cmds* any other scripts based on argc
setup-shell() {
    case $argc_shell in
     bash)
        cat <<EOF
source <(argc --argc-completions bash ${argc_cmds[@]})
EOF
     ;;
     elvish)
        cat <<EOF
eval (argc --argc-completions elvish ${argc_cmds[@]} | slurp)
EOF
     ;;
     fish)
        cat <<EOF
argc --argc-completions fish ${argc_cmds[@]} | source
EOF
     ;;
     nushell)
        cat <<EOF
argc --argc-completions nushell ${argc_cmds[@]} | save -f /tmp/argc.nu
source /tmp/argc.nu
EOF
     ;;
     powershell)
        cat <<EOF
argc --argc-completions powershell ${argc_cmds[@]} | Out-String | Invoke-Expression
EOF
     ;;
     xonsh)
        cat <<EOF
exec(\$(argc --argc-completions xonsh ${argc_cmds[@]}))
EOF
     ;;
     zsh)
        cat <<EOF
source <(argc --argc-completions zsh ${argc_cmds[@]})
EOF
     ;;
    esac
}

eval "$(argc --argc-eval "$0" "$@")"