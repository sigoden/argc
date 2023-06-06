function _argc_completer
    set -l cmd (commandline -o)[1]
    set -l scriptfile
    if [ "$cmd" = "argc" ] 
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which "$cmd")
    end
    if not test -f "$scriptfile"
        __fish_complete_path
        return
    end
    set -l line (commandline -c)
    set -l candicates (argc --argc-compgen fish $scriptfile $line 2>/dev/null)
    if test (count $candicates) -eq 1
        if [ "$candicates[1]" = "__argc_comp:file" ]
            __fish_complete_path
            return
        else if [ "$candicates[1]" = "__argc_comp:dir" ]
            __fish_complete_directories 
            return
        end
    end
    for item in $candicates
        echo "$item"
    end
end
