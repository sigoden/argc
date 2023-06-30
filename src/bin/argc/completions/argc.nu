def _argc_completer [args: list<string>] {
    argc --argc-compgen nushell "" $args
        | split row "\n" | range 0..-2 
        | each { |line| $line | split column "\t" value description } | flatten 
}
