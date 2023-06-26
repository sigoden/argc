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
    var candidates = [(try { argc --argc-compgen elvish (all $args) 2>/dev/null } catch e { echo '' })]
    var skip = (num 0)
    if (> (count $candidates) (num 0)) {
        if (eq $candidates[0] '__argc_value:file') {
            set skip = (num 1)
            argc-complete-path $args[-1]
        } elif (eq $candidates[0] '__argc_value:dir') {
            set skip = (num 1)
            argc-complete-path &is_dir=$true $args[-1]
        }
    }
    all $candidates[$skip..] | each {|candidate| 
        var parts = [(str:split "\t" $candidate)]
        var code-suffix = (if (eq $parts[1] 1) { echo ' ' } else { echo '' })
        if (eq $parts[3] '') {
            edit:complex-candidate $parts[0] &display=(styled $parts[2] 'default') &code-suffix=$code-suffix
        } else {
            edit:complex-candidate $parts[0] &display=(styled $parts[2] 'default')(styled ' ' 'dim white bg-default')(styled '('$parts[3]')' 'dim white') &code-suffix=$code-suffix
        }
    }
}

fn argc-completer {|@args|
    var scriptfile = ''
    if (eq $args[0] 'argc')  {
        set scriptfile = (try { argc --argc-script-path 2>/dev/null } catch e { echo '' })
    } else {
        set scriptfile = (try { which $args[0] } catch e { echo '' })
        if (eq $scriptfile '') {
            argc-complete-path $args[-1]
            return
        }
    }
    argc-complete-impl (all (conj [$scriptfile] (all $args)))
}
