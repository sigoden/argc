# Bash completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion() {
    local argcfile cur opts line index IFS
    cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=()
    argcfile=$(which ${COMP_WORDS[0]})
    if [[ $? != 0 ]]; then
        return 0
    fi
    line=${COMP_LINE:${#COMP_WORDS[0]}}
    IFS=$'\n'
    opts=($(argc --compgen "$argcfile" "$line" 2>/dev/null))
    if [[ ${#opts[@]} == 0 ]]; then
        return 0
    elif [[ ${#opts[@]} == 1 ]]; then
        if [[ "$opts" == \`*\` ]]; then
            opts=($(bash "$argcfile" "${opts:1:-1}" 2>/dev/null))
        elif [[ "$opts" == "<FILE>" ]] || [[ "$opts" == "<PATH>" ]] || [[ "$opts" == "<FILE>..." ]] || [[ "$opts" == "<PATH>..." ]]; then
            opts=()
            compopt +o filenames 
        elif [[ "$opts" == "<DIR>" ]] || [[ "$opts" == "<DIR>..." ]]; then
            opts=()
            compopt +o dirnames
        fi
    fi
    if [[ ${#opts[@]} -gt 0 ]]; then
        CANDIDATES=($(compgen -W "${opts[*]}" -- "${cur}"))
        if [ ${#CANDIDATES[*]} -gt 0 ]; then
            COMPREPLY=($(printf '%q\n' "${CANDIDATES[@]}"))
        fi
    fi
}

complete -F _argc_completion -o bashdefault -o default ${ARGC_SCRIPTS[@]}