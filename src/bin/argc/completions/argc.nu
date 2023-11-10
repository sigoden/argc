def _argc_completer [args: list<string>] {
    argc --argc-compgen nushell "" $args
        | split row "\n" | range 0..-2 
        | each { |line| $line | split column "\t" value description } | flatten 
}

let external_completer = {{|spans| 
    if (not ($env.ARGC_SCRIPTS | find $spans.0 | is-empty)) {{
        _argc_completer $spans
    }} else {{
        # default completer
    }}
}}

$env.config.completions.external = {{
    enable: true
    max_results: 100
    completer: $external_completer
}}
