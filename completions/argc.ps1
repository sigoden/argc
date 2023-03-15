# Powershell completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion to a argc script, simply add the script name to $ARGC_SCRIPTS.

$ARGC_SCRIPTS = ("mycmd1","mycmd2")

$_argc_completion = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $argcfile = (Get-Command $commandAst.CommandElements[0]  -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
    if (!$argcfile) {
        $argcfile = $commandAst.CommandElements[0]
        if (-not(Test-Path -Path $argcfile -PathType Leaf)) {
            return;
        }
    }
    if ($wordToComplete.ToString() -eq "") {
        $tail = " "
    } else {
        $tail = ""
    }
    if ($commandAst.CommandElements.Count -gt 1) {
        $cmds = ($commandAst.CommandElements[1..($commandAst.CommandElements.Count - 1)] -join " ") + $tail
    } else {
        $cmds = $tail
    }
    $comps = (argc --compgen "$argcfile" "$cmds" 2>$null)
    if ($comps -match '^`[^` ]+`$') {
        $comps = (& "$argcfile" $comps.Substring(1, $comps.Length - 2) 2>$null)
    } elseif ($comps -eq "<FILE>" -or $comps -eq "<PATH>" -or $comps -eq "<FILE>..." -or $comps -eq "<PATH>...") {
        $comps = ("")
    } elseif ($comps -eq "<DIR>" -or $comps -eq "<DIR>...") {
        $comps = ("")
    }
    $comps -split "`n" | 
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

Register-ArgumentCompleter -Native -ScriptBlock $_argc_completion -CommandName $ARGC_SCRIPTS