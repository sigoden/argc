use path
use re
use str

fn argc-complete-path {|arg &is_dir=$false|
    edit:complete-filename $arg | each {|c|
        var x = $c[stem]
        if (or (not $is_dir) (path:is-dir $x)) {
            put $c
        }
    }
}

fn argc-complete-impl {|@args|
    if (not (path:is-regular &follow-symlink=$true $args[0])) {
        argc-complete-path $args[-1]
        return
    }
    var candidates = [(try { argc --argc-compgen elvish (all $args) } catch e { echo '' })]
    if (eq (count $candidates) (num 1)) {
        if (eq $candidates[0] '__argc_value:file') {
            argc-complete-path $args[-1]
            return
        } elif (eq $candidates[0] '__argc_value:dir') {
            argc-complete-path &is_dir=$true $args[-1]
            return
        }
    }
    all $candidates | each {|candidate| 
        var parts = [(str:split "\t" $candidate)]
        var code-suffix = (if (eq $parts[1] 1) { echo ' ' } else { echo '' })
        if (eq $parts[3] '') {
            edit:complex-candidate $parts[0] &display=(styled $parts[2] 'default') &code-suffix=$code-suffix
        } else {
            edit:complex-candidate $parts[0] &display=(styled $parts[2] 'default')(styled ' ' 'dim white bg-default')(styled '('$parts[3]')' 'dim white') &code-suffix=$code-suffix
        }
    }
}

fn argc-complete-locate {|cmd|
    try {
        if (eq $cmd 'argc')  {
            argc --argc-script-path
        } else {
            which $cmd
        }
    } catch e {
        echo ''
    }
}

fn argc-completer {|@args|
    argc-complete-impl (all (conj [(argc-complete-locate $args[0])] (all $args)))
}
