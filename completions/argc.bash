# Bash completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion() {
    local argcfile cur opts
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=()
    argcfile=$(which ${COMP_WORDS[0]})
    if [ $? != 0 ]; then
        return 0
    fi
    opts=$(argc --compgen "$argcfile" ${COMP_WORDS[@]:1:$((${#COMP_WORDS[@]} - 2))} 2>/dev/null)
    if [[ "$opts" = __argc_compgen_cmd:* ]]; then
        COMPREPLY=( $(compgen -W "$(bash "$argcfile" ${opts#__argc_compgen_cmd:})" -- "${cur}") )
    else
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
    fi
    return 0
}

complete -F _argc_completion -o bashdefault -o default ${ARGC_SCRIPTS[@]}