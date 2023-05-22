_argc_complete()
{
    local cmd=$words[1]
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        return
    fi
    local line="${words[2,-1]}"
    local IFS=$'\n'
    local candicates=( $(argc --argc-compgen zsh "$scriptfile" "$line" 2>/dev/null) )
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "$candicates[1]" == "__argc_comp:file" ]]; then
            candicates=()
            _path_files
        elif [[ "$candicates[1]" == "__argc_comp:dir" ]]; then
            candicates=()
            _path_files -/
        fi
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        _describe '' candicates
    fi
}
