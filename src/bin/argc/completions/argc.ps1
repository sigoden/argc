using namespace System.Management.Automation

$_argc_completer = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $words = @($commandAst.CommandElements | Where { $_.Extent.StartOffset -lt $cursorPosition } | ForEach-Object { $_.ToString() })
    if ($commandAst.CommandElements[-1].Extent.EndOffset -lt $cursorPosition) {
        $words += ''
    }
    $word1 = $words[0]
    $scriptfile = ""
    if ($word1 -eq "argc") {
        $scriptfile = (argc --argc-script-path 2>$null)
    } else {
        $scriptfile = (Get-Command $word1 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
    }
    if (-not(Test-Path -Path $scriptfile -PathType Leaf)) {
        return
    }
    $line = $words[1..($words.Count-1)] -join " "
    if ($line -eq "") {
        $line = " "
    }
    $candicates = @((argc --argc-compgen powershell $scriptfile $line 2>$null).Split("`n"))
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
