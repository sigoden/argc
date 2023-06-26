_argc_complete_impl() {
    local candidates=()
    while IFS=$'\n' read -r line; do
        if [[ "$line" == "" ]]; then line=$'\0'; fi
        candidates+=( "$line" )
    done < <(argc --argc-compgen zsh $@ 2>/dev/null)
    local skip=0
    if [[ ${#candidates[@]} -gt 0 ]]; then
        if [[ "$candidates[1]" == "__argc_value:file" ]]; then
            skip=1
            _path_files
        elif [[ "$candidates[1]" == "__argc_value:dir" ]]; then
            skip=1
            _path_files -/
        fi
    fi
    if [[ ${#candidates[@]} -gt $skip ]]; then
        local values=()
        local displays=()
        for candidate in ${candidates[@]:$skip}; do
            IFS=$'\t' read -r value display <<< "$candidate"
            values+=( "$value" )
            displays+=( "$display" )
        done
        zstyle ":completion:${curcontext}:*" list-colors "=(#b)(-- *)=0=2;37:=(#b)(--[A-Za-z0-9_-]#)( * -- *)=0==2;37"
        _describe "" displays values -Q -S ''
    fi
}

_argc_completer() {
    if [[ $words[$CURRENT] == "" ]]; then
        words+=( $'\0' )
    fi
    local scriptfile
    if [[ $words[1] == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
        if [[ $? -eq 1 ]]; then
            scriptfile=$'\0'
        fi
    else
       scriptfile=$(which $words[1])
        if [[ $? -eq 1 ]]; then
            _path_files
            return
        fi
    fi
    _argc_complete_impl "$scriptfile" $words
}
