_argc_completer() {
    if [[ $words[$CURRENT] == "" ]]; then
        words+=( $'\0' )
    fi
    local candidates=()
    while IFS=$'\n' read -r line; do
        if [[ "$line" == "" ]]; then line=$'\0'; fi
        candidates+=( "$line" )
    done < <(argc --argc-compgen zsh $'\0' $words 2>/dev/null)
    local values=()
    local displays=()
    local colors
    for candidate in ${candidates[@]}; do
        IFS=$'\t' read -r value display display_value color <<< "$candidate"
        colors="$colors:=(#b)($display_value)( * -- *)=0=$color=2;37:=(#b)($display_value)()=0=$color=2;37"
        values+=( "${value}" )
        displays+=( "$display" )
    done
    zstyle ":completion:${curcontext}:*" list-colors "${colors:1}:=(#b)(-- *)=0=2;37:=(#b)(--[A-Za-z0-9_-]#)( * -- *)=0==2;37"
    _describe "" displays values -Q -S ''
}
