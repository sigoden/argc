use str

fn argc-completer {|@args|
    var candidates = [(argc --argc-compgen elvish (all (conj [''] (all $args))))]
    all $candidates | each {|candidate| 
        var parts = [(str:split "\t" $candidate)]
        var code-suffix = (if (eq $parts[1] 1) { echo ' ' } else { echo '' })
        var display = (if (eq $parts[3] '') {
            put (styled $parts[2] $parts[4])
        } else {
            put (styled $parts[2] $parts[4])(styled ' ' 'dim white bg-default')(styled '('$parts[3]')' 'dim white')
        })
        edit:complex-candidate $parts[0] &display=$display &code-suffix=$code-suffix
    }
}
