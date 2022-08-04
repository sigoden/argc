_argc()
{
    local argcfile items
    argcfile=$(argc --argc-argcfile 2>/dev/null)
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    items=( $(argc --argc-compgen "$argcfile" ${words[@]:1} 2>/dev/null) )
    compadd $items[@]
    return 0
}

compdef _argc argc