_argc_completer()
{
    local word1="$words[1]"
    local scriptfile
    if [[ "$word1" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$word1")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        _path_files
        return
    fi
    local line="${words[2,$CURRENT]}"
    local IFS=$'\n'
    local candicates=( $(argc --argc-compgen zsh "$scriptfile" "$line" 2>/dev/null) )
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
        _describe '' candicates
    fi
}
