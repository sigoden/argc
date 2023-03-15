# Zsh completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion()
{
    local argcfile line opts opts2
    argcfile=$(which $words[1])
    line="${words[2,-1]}"
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    IFS=$'\n'
    opts=( $(argc --compgen "$argcfile" "$line" 2>/dev/null) )
    if [[ ${#opts[@]} == 0 ]]; then
        return 0
    elif [[ ${#opts[@]} == 1 ]]; then
        if [[ "${opts[1]}" == \`*\` ]]; then
            opts=( $(bash "$argcfile" "${opts:1:-1}" 2>/dev/null) )
        fi
    fi
    opts2=()
    for item in "${opts[@]}"; do
        if [[ "$item" == "<FILE>" ]] || [[ "$item" == "<PATH>" ]] || [[ "$item" == "<FILE>..." ]] || [[ "$item" == "<PATH>..." ]]; then
            _path_files
        elif [[ "$item" == "<DIR>" ]] || [[ "$item" == "<DIR>..." ]]; then
            _path_files -/
        else
            opts2+=("$item")
        fi
    done

    if [[ ${#opts2[@]} -gt 0 ]]; then
        compadd -- $opts2[@]
    fi
}

compdef _argc_completion ${ARGC_SCRIPTS[@]}