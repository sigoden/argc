_argc_complete_path() {
    local cur="$1"
    local kind="$2"
    if [[ "$kind" == "dir" ]]; then
        COMPREPLY=($(compgen -d -- "${cur}"))
    else
        COMPREPLY=($(compgen -f -- "${cur}"))
    fi
    compopt -o plusdirs
}

_argc_complete_impl() {
    local cur="${!#}"
    export COMP_WORDBREAKS
    local candidates
    while IFS=$'\n' read -r line; do
        candidates+=( "$line" )
    done < <(argc --argc-compgen bash "$@" 2>/dev/null)
    local space=0
    local skip=0
    if [[ ${#candidates[@]} -gt 0 ]]; then
        if [[ "${candidates[0]}" == "__argc_value:file" ]]; then
            skip=1
            _argc_complete_path "$cur"
            if [[ ${#COMPREPLY[@]} -eq 1 ]] && [[ ${#candidates[@]} -eq 1 ]]; then
                space=1
            fi
        elif [[ "${candidates[0]}" == "__argc_value:dir" ]]; then
            skip=1
            _argc_complete_path "$cur" dir
        fi
    fi
    if [[ $space -eq 1 ]]; then
        compopt +o nospace
    fi
    COMPREPLY=( "${candidates[@]:$skip}" "${COMPREPLY[@]}" )
}

_argc_completer() {
    local words=( ${COMP_LINE:0:${COMP_POINT}} )
    local cur="${COMP_WORDS[COMP_CWORD]}"
    if [[ "$cur" == "" ]]; then
        words+=( "" )
    fi

    local scriptfile
    if [[ "${words[0]}" == "argc" ]]; then
       scriptfile="$(argc --argc-script-path 2>/dev/null)"
    else
       scriptfile="$(which "${words[0]}")"
    fi
    if [[ ! -f "$scriptfile" ]]; then
        _argc_complete_path "$cur"
        return
    fi

    _argc_complete_impl "$scriptfile" "${words[@]}"
}
