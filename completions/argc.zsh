# Zsh completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion()
{
    local argcfile line opts opts2 comp_file comp_dir
    argcfile=$(which $words[1])
    line="${words[2,-1]}"
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    IFS=$'\n'
    opts=( $(argc --compgen "$argcfile" "$line" 2>/dev/null) )
    opts2=()
    for opt in ${opts[@]}; do
        if [[ "$opt" == '-'* ]]; then
            opts2+=( "$opt" )
        elif [[ "$opt" == \`*\` ]]; then
            local choices=( $(bash "$argcfile" "${opt:1:-1}" 2>/dev/null) )
            opts2=( "${opts2[@]}" "${choices[@]}" )
        elif [[ "$opt" == '<'* ]]; then
            if echo "$opt" | grep -qi '\(file\|path\)>\(\.\.\.\)\?'; then
                comp_file=1
            elif echo "$opt" | grep -qi 'dir>\(\.\.\.\)\?'; then
                comp_dir=1
            else
                opts2+=( "$opt" )
            fi
        else
            opts2+=( "$opt" )
        fi
    done
    if [[ "$comp_file" == 1 ]]; then
        _path_files
    elif [[ "$comp_dir" == 1 ]]; then
        _path_files -/
    fi
    if [[ ${#opts2[@]} -gt 0 ]]; then
        compadd -- $opts2[@]
    fi
}

compdef _argc_completion ${ARGC_SCRIPTS[@]}