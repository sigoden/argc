_argc_completer() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local cmd="${COMP_WORDS[0]}"
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        _argc_complete_path "$cur"
        return
    fi
    local line="${COMP_LINE:0:${COMP_POINT}}"
    local IFS=$'\n'
    export COMP_WORDBREAKS
    if [[ "$cur" == "" ]]; then
        line="$line ''"
    fi
    local candicates=($(echo "$line" | _argc_complete_balance_quotes | xargs argc --argc-compgen bash "$scriptfile" 2>/dev/null))
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "${candicates[0]}" == "__argc_comp:file" ]]; then
            _argc_complete_path "$cur"
            return
        elif [[ "${candicates[0]}" == "__argc_comp:dir" ]]; then
            _argc_complete_path "$cur" dir
            return
        fi
    fi

    if [[ ${#candicates[@]} -gt 0 ]]; then
        compopt -o nospace
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

_argc_complete_balance_quotes() {
    awk -v quotes="\"'" '{
        print $0 unbalance_quotations($0)
    }

    function unbalance_quotations(input) {
        split(input, chars, "")
        balances = ""
        for (i=1; i <= length(input); i++) {
            ch = chars[i]
            if (index(quotes, ch) > 0) {
                if (substr(balances, 1, 1) == ch) {
                    balances = substr(balances, 2)
                } else {
                    balances = ch balances
                }
            }
        }
        return balances
    }'
}