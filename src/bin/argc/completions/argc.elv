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

fn argc-completer {|@words|
    var cmd = $words[0]
    var scriptfile = (try {
        if (eq $cmd 'argc')  {
            argc --argc-script-path
        } else {
            which $cmd
        }
    } catch e {
        echo ''
    })
    if (not (path:is-regular &follow-symlink=$true $scriptfile)) {
        argc-complete-path $words[-1]
        return
    }
    var candicates = [(try { argc --argc-compgen elvish $scriptfile (all $words) } catch e { echo '' })]
    if (eq (count $candicates) (num 1)) {
        if (eq $candicates[0] '__argc_comp:file') {
            argc-complete-path $words[-1]
            return
        } elif (eq $candicates[0] '__argc_comp:dir') {
            argc-complete-path &is_dir=$true $words[-1]
            return
        }
    }
    all $candicates | each {|candicate| 
        var parts = [(str:split "\t" $candicate)]
        var code-suffix = (if (eq $parts[1] 1) { echo ' ' } else { echo '' })
        if (eq $parts[2] '') {
            edit:complex-candidate $parts[0] &display=(styled $parts[0] 'default') &code-suffix=$code-suffix
        } else {
            edit:complex-candidate $parts[0] &display=(styled $parts[0] 'default')(styled ' ' 'dim white bg-default')(styled '('$parts[2]')' 'dim white') &code-suffix=$code-suffix
        }
    }
}
