# Bash completion for scripts written with argc
# All argc scripts share the same completion function, put your scripts to $PATH, replace `mycmd1 mycmd2` blow with your scripts' names

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

complete -F _argc_script -o bashdefault -o default mycmd1 mycmd2