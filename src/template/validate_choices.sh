_argc_validate_choices() {
    local render_name="$1" raw_choices="$2" choices item choice concated_choices=""
    while IFS= read -r line; do
        choices+=("$line")
    done <<<"$raw_choices"
    for choice in "${choices[@]}"; do
        if [[ -z "$concated_choices" ]]; then
            concated_choices="$choice"
        else
            concated_choices="$concated_choices, $choice"
        fi
    done
    for item in "${@:3}"; do
        local pass=0 choice
        for choice in "${choices[@]}"; do
            if [[ "$item" == "$choice" ]]; then
                pass=1
            fi
        done
        if [[ $pass -ne 1 ]]; then
            _argc_die "error: invalid value \`$item\` for $render_name"$'\n'"  [possible values: $concated_choices]"
        fi
    done
}
