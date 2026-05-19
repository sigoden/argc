_argc_require_params() {
    local message="$1" missed_envs="" item name render_name
    for item in "${@:2}"; do
        name="${item%%:*}"
        render_name="${item##*:}"
        if [[ -z "${!name:-}" ]]; then
            missed_envs="$missed_envs"$'\n'"  $render_name"
        fi
    done
    if [[ -n "${missed_envs}" ]]; then
        _argc_die "$message$missed_envs"
    fi
}
