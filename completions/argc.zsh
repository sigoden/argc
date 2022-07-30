compdef _argc argc

_argc()
{
    local argcfile items
    argcfile=$(argc --argc-argcfile 2>/dev/null)
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    items=( $(argc --argc-compgen "$argcfile" ${words[@]:1} 2>/dev/null) )
    compadd -d $items
    return 0
}
