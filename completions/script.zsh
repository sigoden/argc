# Zsh completion for scripts written with argc

_argc_script()
{
    local argcfile values
    argcfile=$(which $words[1])
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    values=( $(argc --argc-compgen "$argcfile" $words[2,-2] 2>/dev/null) )
    compadd -- $values[@]
    return 0
}

compdef _argc_script mycmd1 mycmd2 # just replace mycmd1 mycmd2 with your scripts
