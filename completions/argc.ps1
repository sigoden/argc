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
    $opts = (argc --compgen "$argcfile" "$cmds" 2>$null).Split("`n")
    $opts2 = @()
    foreach ($opt in $opts) {
        if ($opt -match '^-') {
            $opts2 += $opt
        } elseif ($opt -match '^`[^` ]+`$') {
            $choices = (& "$argcfile" $opt.Substring(1, $opt.Length - 2) 2>$null).Split("`n")
            $opts2 += $choices
        } elseif ($opt -match '^<') {
            if ($opt -imatch "file|path>(\.\.\.)?") {
                $comp_file = True
            } elseif ($opt -imatch "dir>(\.\.\.)?") {
                $comp_dir = True;
            } else {
                $opts2 += $opt
            }
        } else {
            $opts2 += $opt
        }
    }

    $result = ($opts2 | 
        Where-Object { $_ -like "$wordToComplete*" } |
        ForEach-Object { 
            if ($_.StartsWith("-")) {
                $t = [System.Management.Automation.CompletionResultType]::ParameterName
            } else {
                $t = [System.Management.Automation.CompletionResultType]::ParameterValue
            }
            [System.Management.Automation.CompletionResult]::new($_, $_, $t, '-')
        })

    $paths = @()
    if ($comp_file) {
        $paths = (Get-ChildItem -Path "$wordToComplete*" | Select-Object -ExpandProperty Name)
    } elseif ($comp_dir) {
        $paths = (Get-ChildItem -Attributes Directory -Path "$wordToComplete*" | Select-Object -ExpandProperty Name)
    }
    foreach ($path in $paths) {
        $t = [System.Management.Automation.CompletionResultType]::ParameterValue
        $result.Add([System.Management.Automation.CompletionResult]::new($path, $path, $t, '-'))
    }

    return $result
}

Register-ArgumentCompleter -Native -ScriptBlock $_argc_completion -CommandName $ARGC_SCRIPTS