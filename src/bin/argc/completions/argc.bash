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
    if [[ ! -f "$1" ]]; then
        _argc_complete_path "$cur"
        return
    fi

    export COMP_WORDBREAKS
    local IFS=$'\n'
    local candidates=($(argc --argc-compgen bash "$@" 2>/dev/null))
    if [[ ${#candidates[@]} -eq 1 ]]; then
        if [[ "${candidates[0]}" == "__argc_comp:file" ]]; then
            _argc_complete_path "$cur"
            return
        elif [[ "${candidates[0]}" == "__argc_comp:dir" ]]; then
            _argc_complete_path "$cur" dir
            return
        fi
    fi
    if [[ ${#candidates[@]} -gt 0 ]]; then
        compopt -o nospace
        COMPREPLY=(${candidates[@]})
    fi
}

_argc_complete_locate() {
    if [[ "$1" == "argc" ]]; then
       argc --argc-script-path 2>/dev/null
    else
       which "$1"
    fi
}

_argc_completer() {
    local words=( ${COMP_LINE:0:${COMP_POINT}} )
    local cur="${COMP_WORDS[COMP_CWORD]}"
    if [[ "$cur" == "" ]]; then
        words+=( "" )
    fi
    _argc_complete_impl "$(_argc_complete_locate "${words[0]}")" "${words[@]}"
}
