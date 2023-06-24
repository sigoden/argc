_argc_complete_impl() {
    if [[ ! -f $1 ]]; then
        _path_files
        return
    fi
    local candidates=()
    while IFS=$'\n' read -r line; do
        if [[ "$line" == "" ]]; then line=$'\0'; fi
        candidates+=( "$line" )
    done < <(argc --argc-compgen zsh $@ 2>/dev/null)
    if [[ ${#candidates[@]} -eq 1 ]]; then
        if [[ "$candidates[1]" == "__argc_value:file" ]]; then
            _path_files
            return
        elif [[ "$candidates[1]" == "__argc_value:dir" ]]; then
            _path_files -/
            return
        fi
    fi
    if [[ ${#candidates[@]} -gt 0 ]]; then
        local values=()
        local displays=()
        for candidate in ${candidates[@]}; do
            IFS=$'\t' read -r value display <<< "$candidate"
            values+=( "$value" )
            displays+=( "$display" )
        done
        zstyle ":completion:${curcontext}:*" list-colors "=(#b)(-- *)=0=2;37"
        _describe "" displays values -Q -S ''
    fi
}

_argc_complete_locate() {
    if [[ $1 == "argc" ]]; then
       argc --argc-script-path 2>/dev/null
    else
       which $1
    fi
}

_argc_completer() {
    if [[ $words[$CURRENT] == "" ]]; then
        words+=( $'\0' )
    fi
    _argc_complete_impl $(_argc_complete_locate $words[1]) $words
}
