_argc_require_tools() {
    local tool missing_tools=()
    for tool in "$@"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools+=("$tool")
        fi
    done
    if [[ "${#missing_tools[@]}" -gt 0 ]]; then
        echo "error: missing tools: ${missing_tools[*]}" >&2
        exit 1
    fi
}
