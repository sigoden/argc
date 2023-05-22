using namespace System.Management.Automation

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
            return
        }
    }
    $tail = if ($wordToComplete.ToString() -eq "") { " " } else { "" }
    if ($commandAst.CommandElements.Count -gt 1) {
        $line = ($commandAst.CommandElements[1..($commandAst.CommandElements.Count - 1)] -join " ") + $tail
    } else {
        $line = $tail
    }
    $candicates = (argc --argc-compgen powershell "$scriptfile" "$line" 2>$null).Split("`n")
    if ($candicates.Count -eq 1) {
        if (($candicates[0] -eq "__argc_comp:file") -or ($candicates[0] -eq "__argc_comp:dir")) {
            return
        } elseif ($candicates[0] -eq "") {
            return ""
        }
    }
    $candicates | ForEach-Object { 
        $parts=($_ -split "`t")
        $value = $parts[0]
        $desc = if ($parts[1]) { $parts[1] } else { " " }
        [CompletionResult]::new($value, $value, [CompletionResultType]::ParameterValue, $desc)
    }
}
