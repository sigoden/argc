_argc_completer() {
    local words
    _argc_completer_parse_line

    export COMP_WORDBREAKS
    while IFS=$'\n' read -r line; do
        COMPREPLY+=( "$line" )
    done < <(argc --argc-compgen bash "" "${words[@]}" 2>/dev/null)
}

_argc_completer_parse_line() {
    local line len i char prev_char word unbalance word_index
    word_index=0
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
                words[$word_index]="$word"
                word_index=$((word_index+1))
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
    words[$word_index]="$word"
}
