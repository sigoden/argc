_argc_complete() {
    local cmd=${COMP_WORDS[0]}
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        return
    fi
    cur="${COMP_WORDS[COMP_CWORD]}"
    local line=${COMP_LINE:${#COMP_WORDS[0]}}
    local IFS=$'\n'
    export COMP_WORDBREAKS
    local candicates=($(argc --argc-compgen bash "$scriptfile" "$line" 2>/dev/null))
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "${candicates[0]}" == "__argc_comp:file" ]]; then
            candicates=()
            _argc_complete_path
        elif [[ "${candicates[0]}" == "__argc_comp:dir" ]]; then
            candicates=()
            _argc_complete_path -d
        fi
    fi

    _argc_complete_nospace "${candicates[@]}"

    if [[ ${#candicates[@]} -gt 0 ]]; then
        COMPREPLY=(${candicates[@]})
    fi
}

_argc_complete_path() {
    if type _filedir >/dev/null 2>&1; then
        _filedir ${1-}
    else
        if [[ ${1-} == "-d" ]]; then
            compopt -o nospace -o plusdirs > /dev/null 2>&1
            COMPREPLY=($(compgen -d -- "${cur}"))
        else
            compopt -o nospace -o plusdirs > /dev/null 2>&1
            COMPREPLY=($(compgen -f -- "${cur}"))
        fi
    fi
}

_argc_complete_nospace() {
    if [[ $# -eq 0 ]]; then
        return
    fi
    local nospace=1
    local value last_char
    for value in ${@}; do
        last_char="${value: -1}"
        if [[ ! "$COMP_WORDBREAKS" == *"$last_char"* ]]; then
            nospace=0
            break
        fi
    done
    if [[ "$nospace" == "1" ]]; then
        compopt -o nospace > /dev/null 2>&1
    fi
}