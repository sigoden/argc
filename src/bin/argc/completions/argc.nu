def _argc_complete_path [name: string, is_dir: bool] {
    let sep = if $nu.os-info.family == "windows" {
        "\\"
    } else {
        "/"
    }
    let paths = (ls ($name + '*'))
    mut paths = if $is_dir {
        $paths | where type == dir
    } else {
        $paths
    }
    let homedir = ('~' | path expand)
    let num_paths = ($paths | length)
    $paths | each {|it| 
        let value = (if $it.type == 'dir' {
            $it.name + $sep 
        } else {
            $it.name + ' '
        })
        if ($name | str starts-with '~') {
            $value | str replace $homedir '~'
        } else if ($name | str starts-with ('.' + $sep)) {
            '.' + $sep + $value
        } else {
            $value
        }
    }
}

def _argc_complete_list [] {
    each { |line| $line | split column "\t" value description } | flatten 
}

def _argc_complete_impl [args: list<string>] {
    let cur = ($args | last)
    if not ($args.0 | path exists) {
        return (_argc_complete_path $cur false | _argc_complete_list)
    }
    mut candidates = ((do { argc --argc-compgen nushell $args } | complete | get stdout) | split row "\n" | range 0..-2)
    if ($candidates | length) == 1  {
        if $candidates.0 == '__argc_value:file' {
            $candidates = (_argc_complete_path $cur false)
        } else if $candidates.0 == '__argc_value:dir' {
            $candidates = (_argc_complete_path $cur true)
        }
    }
    $candidates | _argc_complete_list
}

def _argc_completer [args: list<string>] {
    let scriptfile = (try { 
        if $args.0 == 'argc' {
            do { argc --argc-script-path } | complete | get stdout 
        } else {
            which $args.0 | get 0.path
        }
    })
   _argc_complete_impl ($args | insert 0 $scriptfile)
}
