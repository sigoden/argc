_argc_completer() {
    local words=( ${COMP_LINE:0:${COMP_POINT}} )
    local cur="${COMP_WORDS[COMP_CWORD]}"
    if [[ "$cur" == "" ]]; then
        words+=( "" )
    fi

    export COMP_WORDBREAKS
    while IFS=$'\n' read -r line; do
        COMPREPLY+=( "$line" )
    done < <(argc --argc-compgen bash "" "${words[@]}" 2>/dev/null)
}
