# Bash completion for scripts written with argc

_argc_script() {
    local argcfile cur opts
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=()
    argcfile=$(which ${COMP_WORDS[0]})
    if [ $? != 0 ]; then
        return 0
    fi
    opts=$(argc --argc-compgen "$argcfile" ${COMP_WORDS[@]:1:$((${#COMP_WORDS[@]} - 2))} 2>/dev/null)
    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
    return 0
}

complete -F _argc_script -o bashdefault -o default mycmd1 mycmd2 # just replace mycmd1 mycmd2 with your scripts
