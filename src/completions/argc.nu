def _argc_completer [args: list<string>] {
    argc --argc-compgen nushell "" ...$args
        | split row "\n"
        | each { |line| $line | split column "\t" value description }
        | flatten 
}

let external_completer = {|spans| 
    _argc_completer $spans
}

$env.config.completions.external.enable = true
$env.config.completions.external.completer = $external_completer
