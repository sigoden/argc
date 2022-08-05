# Zsh completion for argc

_argc()
{
    local argcfile values
    argcfile=$(argc --argc-argcfile 2>/dev/null)
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    values=( $(argc --argc-compgen "$argcfile" $words[2,-2] 2>/dev/null) )
    compadd -- $values[@]
    return 0
}

compdef _argc argc