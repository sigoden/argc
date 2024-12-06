_argc_completer() {
    declare -a _argc_completer_words
    _argc_completer_parse_line

    export COMP_WORDBREAKS
    while IFS=$'\n' read -r line; do
        COMPREPLY+=( "$line" )
    done < <(argc --argc-compgen bash "" "${_argc_completer_words[@]}" 2>/dev/null)
}

_argc_completer_parse_line() {
    local line len i char prev_char word unbalance
    line="${COMP_LINE:0:$COMP_POINT}"
    len="${#line}"

    for ((i=0; i<len; i++)); do
        char="${line:i:1}"
        if [[ -n "$unbalance" ]]; then
            word="$word$char"
            if [[  "$unbalance" == "$char" ]]; then
                unbalance=""
            fi
        elif [[ "$char" == " " ]]; then
            if [[ "$prev_char" == "\\" ]]; then
                word="$word$char"
            elif [[ -n "$word" ]]; then
                _argc_completer_words+=( "$word" )
                word=""
            fi
        elif [[ "$char" == "'" || "$char" == '"' ]]; then
            word="$word$char"
            unbalance="$char"
        elif [[ "$char" == "\\" ]]; then
            if [[ "$prev_char" == "\\" ]]; then
                word="$word$char"
            fi
        else
            word="$word$char"
        fi
        prev_char="$char"
    done

    _argc_completer_words+=( "$word" )
}

complete -F _argc_completer -o nospace -o nosort \
    __COMMANDS__
