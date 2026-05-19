_argc_maybe_flag_option() {
    local signs="$1" arg="$2"
    if [[ -z "$signs" ]]; then
        return 1
    fi
    local cond=false
    if [[ "$signs" == *"+"* ]]; then
        if [[ "$arg" =~ ^\+[^+].* ]]; then
            cond=true
        fi
    elif [[ "$arg" == -* ]]; then
        if (( ${#arg} < 3 )) || [[ ! "$arg" =~ ^---.* ]]; then
            cond=true
        fi
    fi
    if [[ "$cond" == "false" ]]; then
        return 1
    fi
    local value="${arg%%=*}"
    if [[ "$value" =~ [[:space:]] ]]; then
        return 1
    fi
    return 0
}
