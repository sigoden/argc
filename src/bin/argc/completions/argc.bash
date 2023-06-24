_argc_complete_path() {
    local cur="$1"
    local kind="$2"
    if [[ "$kind" == "dir" ]]; then
        compopt -o nospace -o plusdirs > /dev/null 2>&1
        COMPREPLY=($(compgen -d -- "${cur}"))
    else
        compopt -o nospace -o plusdirs > /dev/null 2>&1
        COMPREPLY=($(compgen -f -- "${cur}"))
    fi
}

_argc_complete_impl() {
    local cur="${!#}"
    export COMP_WORDBREAKS
    local candidates
    mapfile -t candidates < <(argc --argc-compgen bash "$@" 2>/dev/null)
    if [[ ${#candidates[@]} -eq 1 ]]; then
        if [[ "${candidates[0]}" == "__argc_value:file" ]]; then
            _argc_complete_path "$cur"
            return
        elif [[ "${candidates[0]}" == "__argc_value:dir" ]]; then
            _argc_complete_path "$cur" dir
            return
        fi
    fi
    if [[ ${#candidates[@]} -gt 0 ]]; then
        compopt -o nospace
        COMPREPLY=( "${candidates[@]}" )
    fi
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
