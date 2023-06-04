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
    var word1 = $words[0]
    var scriptfile = (try {
        if (eq $word1 'argc')  {
            argc --argc-script-path
        } else {
            which $word1
        }
    }  catch e {
        echo ''
    })
    if (not (path:is-regular $scriptfile)) {
        argc-complete-path $words[-1]
        return
    }
    var line = (all $words[1..] | str:join ' ')
    if (eq $line '') {
        set line = ' '
    }
    var candicates = [(argc --argc-compgen elvish $scriptfile $line)]
    if (eq (count $candicates) (num 1)) {
        if (eq $candicates[0] '__argc_comp:file') {
            argc-complete-path $words[-1]
            return
        } elif (eq $candicates[0] '__argc_comp:dir') {
            argc-complete-path &is_dir=$true $words[-1]
            return
        }
    }
    var pad = (num 2)
    var candicates = [(all $candicates | each {|x|
        var parts = [(str:split "\t" $x)]
        var text = $parts[0]
        var text-len = (count $text)
        if (> $text-len $pad) {
            set pad = $text-len
        }
        var desc = (if (eq (count $parts) (num 1)) { echo ' ' } else { echo $parts[1] })
        put [$text $desc]
    })]
    var pad = (+ $pad (num 2))
    all $candicates | each {|x| 
        var spaces = (repeat (- $pad (count $x[0])) ' ' | str:join '')
        edit:complex-candidate $x[0] &display=$x[0]$spaces$x[1] &code-suffix=' '
    }
}
