def _argc_complete_path [name: string, is_dir: bool] {
    let sep = if $nu.os-info.family == "windows" {
        "\\"
    } else {
        "/"
    }
    let paths = (try {ls ($name + '*')} catch { [] })
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
    mut candidates = ((do { argc --argc-compgen nushell $args } | complete | get stdout) | split row "\n" | range 0..-2)
    if ($candidates | length) > 0  {
        if $candidates.0 == '__argc_value:file' {
            $candidates = ($candidates | skip 1 | append (_argc_complete_path $cur false))
        } else if $candidates.0 == '__argc_value:dir' {
            $candidates = ($candidates | skip 1 | append (_argc_complete_path $cur true))
        }
    }
    $candidates | _argc_complete_list
}

def _argc_completer [args: list<string>] {
    mut scriptfile = ''
    if $args.0 == 'argc' {
        $scriptfile = (do { argc --argc-script-path } | complete | get stdout)
    } else {
        $scriptfile = (which $args.0 | get 0.path)
        if ($scriptfile | is-empty) {
            return (_argc_complete_path ($args | last) false | _argc_complete_list)
        }
    }
   _argc_complete_impl ($args | insert 0 $scriptfile)
}
