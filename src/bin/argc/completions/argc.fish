function _argc_completer
    set -l words (commandline -c | string trim -l | string split " " --)
    set -l word1 $words[1]
    set -l scriptfile
    if [ "$word1" = "argc" ] 
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which "$word1")
    end
    if not test -f "$scriptfile"
        return
    end
    set -l line "$words[2..]"
    set -l candicates (argc --argc-compgen fish $scriptfile $line 2>/dev/null)
    if test (count $candicates) -eq 1
        if [ "$candicates[1]" = "__argc_comp:file" ]
            set candicates
            __fish_complete_path
        else if [ "$candicates[1]" = "__argc_comp:dir" ]
            set candicates
            __fish_complete_directories 
        end
    end
    for item in $candicates
        echo "$item"
    end
end
