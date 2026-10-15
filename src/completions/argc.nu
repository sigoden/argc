def _argc_completer [args: list<string>] {
    argc --argc-compgen nushell "" ...$args
        | lines
        | each { |line| $line | split column "\t" value description }
        | flatten
}

let external_completer = {|place|
    let spans = if (($place | describe) | str starts-with "record") {
        $place.command
    } else {
        $place
    }
    _argc_completer $spans
}

$env.config.completions.external.enable = true
$env.config.completions.external.completer = $external_completer
