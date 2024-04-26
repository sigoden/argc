#!/usr/bin/env bash
set -e

# @cmd Test the project
# @alias t
test() {
    cargo test "$@"
}

# @cmd Test features matrix
test-features() {
    cargo hack --no-dev-deps check --feature-powerset --depth 2 --lib
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
# @option -s --shell[=bash|elvish|fish|nushell|powershell|xonsh|zsh|tcsh] shell type
# @arg cmds* any other scripts based on argc
setup-shell() {
    case $argc_shell in
        bash) echo "source <(argc --argc-completions bash ${argc_cmds[@]})" ;;
        elvish) echo "eval (argc --argc-completions elvish ${argc_cmds[@]} | slurp)" ;;
        fish) echo "argc --argc-completions fish ${argc_cmds[@]} | source" ;;
        nushell) echo "argc --argc-completions nushell | save -f argc.nu"$'\n'"source argc.nu" ;;
        powershell) echo "argc --argc-completions powershell ${argc_cmds[@]} | Out-String | Invoke-Expression" ;;
        xonsh) echo "exec(\$(argc --argc-completions xonsh ${argc_cmds[@]}))" ;;
        zsh) echo "source <(argc --argc-completions zsh ${argc_cmds[@]})" ;;
        tcsh) echo "eval \`argc --argc-completions tcsh ${argc_cmds[@]}\`" ;;
    esac
}

eval "$(argc --argc-eval "$0" "$@")"