_argc_check_bool() {
    local env_name="$1" param_name=$2
    local env_value="${!env_name}"
    if [[ "$env_value" == "true" ]] || [[ "$env_value" == "1" ]]; then
        return 0
    elif [[ "$env_value" == "false" ]] || [[ "$env_value" == "0" ]]; then
        return 1
    else
        _argc_die "error: environment variable '$env_name' has invalid value for param '$param_name'"
    fi
}
