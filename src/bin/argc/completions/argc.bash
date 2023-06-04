_argc_completer() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local word1="${COMP_WORDS[0]}"
    local scriptfile
    if [[ "$word1" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$word1")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        _argc_complete_path "$cur"
        return
    fi
    local line="${COMP_WORDS[@]:1:COMP_CWORD}"

    local IFS=$'\n'
    export COMP_WORDBREAKS
    local candicates=($(argc --argc-compgen bash "$scriptfile" "$line" 2>/dev/null))
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "${candicates[0]}" == "__argc_comp:file" ]]; then
            _argc_complete_path "$cur"
            return
        elif [[ "${candicates[0]}" == "__argc_comp:dir" ]]; then
            candicates=()
            _argc_complete_path "$cur" dir
            return
        fi
    fi

    _argc_complete_nospace "${candicates[@]}"

    if [[ ${#candicates[@]} -gt 0 ]]; then
        COMPREPLY=(${candicates[@]})
    fi
}

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