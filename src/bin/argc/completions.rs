use anyhow::{bail, Result};

const BASH_SCRIPT: &str = r###"
_argc_complete() {
    local cmd=${COMP_WORDS[0]}
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        return 0
    fi
    cur="${COMP_WORDS[COMP_CWORD]}"
    local line=${COMP_LINE:${#COMP_WORDS[0]}}
    local IFS=$'\n'
    local candicates=($(argc --argc-compgen "$scriptfile" "$line" 2>/dev/null))
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "${candicates[0]}" == "__argc_comp:file" ]]; then
            candicates=()
            COMPREPLY=($(compgen -f -- "${cur}"))
        elif [[ "${candicates[0]}" == "__argc_comp:dir" ]]; then
            candicates=()
            COMPREPLY=($(compgen -d -- "${cur}"))
        fi
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        candicates=($(compgen -W "${candicates[*]}" -- "${cur}"))
        if [ ${#candicates[@]} -gt 0 ]; then
            COMPREPLY=( "${COMPREPLY[@]}" $(printf '%q\n' "${candicates[@]}") )
        fi
    fi
}
"###;

const FISH_SCRIPT: &str = r###"
function _argc_complete
    set -l tokens (commandline -c | string trim -l | string split " " --)
    set -l cmd "$tokens[1]"
    set -l scriptfile
    if [ "$cmd" = "argc" ] 
        set scriptfile (argc --argc-script-path 2>/dev/null)
    else
        set scriptfile (which "$cmd")
    end
    if not test -f "$scriptfile"
        return 0
    end
    set -l line "$tokens[2..]"
    set -l IFS '\n'
    set -l candicates (argc --argc-compgen "$scriptfile" "$line" 2>/dev/null)
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
        echo $item
    end
end
"###;

const ZSH_SCRIPT: &str = r###"
_argc_complete()
{
    local cmd=$words[1]
    local scriptfile
    if [[ "$cmd" == "argc" ]]; then
       scriptfile=$(argc --argc-script-path 2>/dev/null)
    else
       scriptfile=$(which "$cmd")
    fi
    if [[ ! -f "$scriptfile" ]]; then
        return 0
    fi
    local line="${words[2,-1]}"
    local IFS=$'\n'
    local candicates=( $(argc --argc-compgen "$scriptfile" "$line" 2>/dev/null) )
    if [[ ${#candicates[@]} -eq 1 ]]; then
        if [[ "$candicates[1]" == "__argc_comp:file" ]]; then
            candicates=()
            _path_files
        elif [[ "$candicates[1]" == "__argc_comp:dir" ]]; then
            candicates=()
            _path_files -/
        fi
    fi
    if [[ ${#candicates[@]} -gt 0 ]]; then
        compadd -- $candicates[@]
    fi
}
"###;

const POWERSHELL_SCRIPT: &str = r###"
$_argc_complete = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $cmd = $commandAst.CommandElements[0]
    if ($cmd -eq "argc") {
        $scriptfile = (argc --argc-script-path 2>$null)
    } else {
        $scriptfile = (Get-Command $cmd  -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
    }
    if (!$scriptfile) {
        $scriptfile = $cmd
        if (-not(Test-Path -Path $scriptfile -PathType Leaf)) {
            return;
        }
    }
    $tail = if ($wordToComplete.ToString() -eq "") { " " } else { "" }
    if ($commandAst.CommandElements.Count -gt 1) {
        $line = ($commandAst.CommandElements[1..($commandAst.CommandElements.Count - 1)] -join " ") + $tail
    } else {
        $line = $tail
    }
    $candicates = (argc --argc-compgen "$scriptfile" "$line" 2>$null).Split("`n")
    if ($candicates.Count -eq 1) {
        if ($candicates[0] -eq "__argc_comp:file") {
            $candicates = (Get-ChildItem -Path "$wordToComplete*" | Select-Object -ExpandProperty Name)
        } elseif ($candicates[0] -eq "__argc_comp:dir") {
            $candicates = (Get-ChildItem -Attributes Directory -Path "$wordToComplete*" | Select-Object -ExpandProperty Name)
        }
    }
    $candicates | 
        Where-Object { $_ -like "$wordToComplete*" } |
        ForEach-Object { 
            if ($_.StartsWith("-")) {
                $t = [System.Management.Automation.CompletionResultType]::ParameterName
            } else {
                $t = [System.Management.Automation.CompletionResultType]::ParameterValue
            }
            [System.Management.Automation.CompletionResult]::new($_, $_, $t, '-')
        }
}
"###;

pub(crate) fn generate(args: &[String]) -> Result<String> {
    let shell = match args.get(0) {
        Some(v) => v,
        None => {
            bail!("No shell specified, Please specify the one of bash,zsh,fish,powershell")
        }
    };
    let mut cmds = vec!["argc"];
    cmds.extend(args[1..].iter().map(|v| v.as_str()));
    let output = match shell.as_str() {
        "bash" => {
            let registers = format!("complete -F _argc_complete {}", cmds.join(" "));
            format!("{BASH_SCRIPT}\n{registers}\n",)
        }
        "fish" => {
            let lines: Vec<String> = cmds
                .iter()
                .map(|v| format!(r#"complete -x -c {v}  -n 'true' -a "(_argc_complete)""#))
                .collect();
            let registers = lines.join("\n");
            format!("{FISH_SCRIPT}\n{registers}\n",)
        }
        "zsh" => {
            let registers = format!("compdef _argc_complete {}", cmds.join(" "));
            format!("{ZSH_SCRIPT}\n{registers}\n",)
        }
        "powershell" => {
            let lines: Vec<String> = cmds.iter().map(|v| format!("Register-ArgumentCompleter -Native -ScriptBlock $_argc_complete -CommandName {v} ")).collect();
            let registers = lines.join("\n");
            format!("{POWERSHELL_SCRIPT}\n{registers}\n",)
        }
        _ => {
            bail!("Invalid shell value, shell must be one of bash,zsh,fish,powershell")
        }
    };
    Ok(output)
}
