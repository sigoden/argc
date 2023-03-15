# Zsh completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion()
{
    local argcfile line opts
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
        elif [[ "${opts[1]}" == "<FILE>" ]] || [[ "${opts[1]}" == "<PATH>" ]] || [[ "${opts[1]}" == "<FILE>..." ]] || [[ "${opts[1]}" == "<PATH>..." ]]; then
            opts=()
            _path_files
        elif [[ "${opts[1]}" == "<DIR>" ]] || [[ "${opts[1]}" == "<DIR>..." ]]; then
            opts=()
            _path_files -/
        fi
    fi
    
    if [[ ${#opts[@]} -gt 0 ]]; then
        compadd -- $opts[@]
    fi
}

compdef _argc_completion ${ARGC_SCRIPTS[@]}