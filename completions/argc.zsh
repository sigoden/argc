# Zsh completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )
ARGC_BASH=${ARGC_BASH:-bash}

_argc_completion()
{
    local scriptfile=$(which $words[1])
    if [[ ! -f "$scriptfile" ]]; then
        return 0
    fi
    local line="${words[2,-1]}"
    local IFS=$'\n'
    local compgen_values=( $(argc --compgen "$scriptfile" "$line" 2>/dev/null) )
    local candicates=()
    local arg_value=""
    for item in ${compgen_values[@]}; do
        if [[ "$item" == '-'* ]]; then
            candicates+=( "$item" )
        elif [[ "$item" == \`*\` ]]; then
            local choices=( $("$ARGC_BASH" "$scriptfile" "${item:1:-1}" "$line" 2>/dev/null) )
            if [[ ${#choices[@]} -eq 1 ]]; then
                local value=${choices[1]}
                if [[ "$value" == '<'* ]] || [[ "$value" == '['* ]]; then
                    arg_value="$value"
                else
                    candicates+=( "$value" )
                fi
            else
                candicates=( "${candicates[@]}" "${choices[@]}" )
            fi
        elif [[ "$item" == '<'* ]] || [[ "$item" == '['* ]]; then
            arg_value="$item"
        else
            candicates+=( "$item" )
        fi
    done
    if [[ ${#candicates[@]} -eq 0 ]]; then
        if echo "$arg_value" | grep -qi '\(file\|path\)'; then
            _path_files
        elif echo "$arg_value" | grep -qi 'dir'; then
            _path_files -/
        fi
    elif [[ -n "$arg_value" ]]; then
        candicates+=( "$arg_value" )
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        compadd -- $candicates[@]
    fi
}

compdef _argc_completion ${ARGC_SCRIPTS[@]}