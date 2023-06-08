_argc_completer()
{
    local cmd="$words[1]"
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        _path_files
        return
    fi
    if [[ "$words[$CURRENT]" == "" ]]; then
        words+=( $'\0' )
    fi
    local IFS=$'\n'
    local candicates=($(argc --argc-compgen zsh "$scriptfile" $words 2>/dev/null))
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "$candicates[1]" == "__argc_comp:file" ]]; then
            _path_files
            return
        elif [[ "$candicates[1]" == "__argc_comp:dir" ]]; then
            _path_files -/
            return
        fi
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        local values=()
        local displays=()
        for candicate in ${candicates[@]}; do
            IFS=$'\t' read value space description <<< "$candicate"
            if [[ $space == 1 ]]; then
                values+=( "$value " )
            else
                values+=( "$value" )
            fi
            if [[ -n "$description" ]]; then
                displays+=( "$value:$description" )
            else
                displays+=( "$value" )
            fi
        done
        _describe "" displays values -Q -S ''
    fi
}
