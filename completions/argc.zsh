# Zsh completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

ARGC_SCRIPTS=( mycmd1 mycmd2 )

_argc_completion()
{
    local argcfile values
    argcfile=$(which $words[1])
    if [[ $? -ne 0 ]]; then
        return 0
    fi
    values=( $(argc --compgen "$argcfile" $words[2,-2] 2>/dev/null) )
    if [[ "$values" = __argc_compgen_cmd:* ]]; then
        values=( $(bash "$argcfile" ${values#__argc_compgen_cmd:}) )
    fi
    compadd -- $values[@]
    return 0
}

compdef _argc_completion ${ARGC_SCRIPTS[@]}