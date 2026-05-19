_argc_split_positionals() {
    _argc_split_positionals_values=()
    local values_index="$1" values_size="$2" delimiter="$3" item values
    local split_values=("${argc__positionals[@]:values_index:values_size}")
    for item in "${split_values[@]}"; do
        IFS="$delimiter" read -r -a values <<<"$item"
        _argc_split_positionals_values+=("${values[@]}")
    done
    local heads=() tails=() tails_index=$((values_index + values_size))
    if [[ $values_index -gt 0 ]]; then
        heads=("${argc__positionals[@]:0:values_index}")
    fi
    if [[ $tails_index -lt ${#argc__positionals[@]} ]]; then
        tails=("${argc__positionals[@]:tails_index}")
    fi
    argc__positionals=("${heads[@]}" "${_argc_split_positionals_values[@]}" "${tails[@]}")
}
