_argc_completer() {
    local new_words
    _argc_completer_reassemble_words

    local candidates=() values=() displays=() colors display_value
    while IFS=$'\n' read -r line; do
        if [[ "$line" == "" ]]; then line=$'\0'; fi
        candidates+=( "$line" )
    done < <(argc --argc-compgen zsh $'\0' $new_words 2>/dev/null)
    for candidate in ${candidates[@]}; do
        IFS=$'\t' read -r value display color_key color <<< "$candidate"
        colors="$colors:=(#b)($color_key)( * -- *)=0=$color=2;37:=(#b)($color_key)()=0=$color=2;37"
        values+=( "${value}" )
        displays+=( "$display" )
    done
    zstyle ":completion:${curcontext}:*" list-colors "${colors:1}:=(#b)(-- *)=0=2;37:=(#b)(--[A-Za-z0-9_-]#)( * -- *)=0==2;37"
    _describe "" displays values -Q -S '' -o nosort
}

_argc_completer_reassemble_words() {
    local i cword
    new_words=()
    for ((i=1; i<=$CURRENT; i++)); do
        cword="$words[$i]"
        if [[ "$cword" == "" ]]; then
            new_words+=( $'\0' )
        else
            if [[ "$cword" == *"\\"* ]]; then
                local j char next_char cword_len word
                cword_len="${#cword}"
                for ((j=0; j<$cword_len; j++)); do
                    char="${cword:$j:1}"
                    if [[ "$char" == "\\" ]]; then
                        next_char="${cword:$((j+1)):1}"
                        if [[ "$next_char" == "\\" ]]; then
                            word="$word$char"
                            j=$((j+1))
                        fi
                    else
                        word="$word$char"
                    fi
                done
                cword="$word"
            fi
            new_words+=( "$cword" )
        fi
    done
}
