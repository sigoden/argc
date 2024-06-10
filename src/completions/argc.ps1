using namespace System.Management.Automation

$_argc_completer = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $words = @($commandAst.CommandElements | Where { $_.Extent.StartOffset -lt $cursorPosition } | ForEach-Object {
        $word =  $_.ToString() 
        if ($word.Length -gt 2) {
            if (($word.StartsWith('"') -and $word.EndsWith('"')) -or ($word.StartsWith("'") -and $word.EndsWith("'"))) {
                $word = $word.Substring(1, $word.Length - 2)
            }
        }
        return $word
    })
    $emptyS = ''
    if ($PSVersionTable.PSVersion.Major -eq 5) {
        $emptyS = '""'
    }
    $lastElemIndex = -1
    if ($words.Count -lt $commandAst.CommandElements.Count) {
        $lastElemIndex = $words.Count - 1
    }
    if ($commandAst.CommandElements[$lastElemIndex].Extent.EndOffset -lt $cursorPosition) {
        $words += $emptyS
    }
    @((argc --argc-compgen powershell $emptyS $words) -split "`n") | ForEach-Object {
        $parts = ($_ -split "`t")
        if ($parts[1] -eq "1") {
            $value = $parts[0] + " "
        } else {
            $value = $parts[0]
        }
        if ($parts[3] -eq "") {
            $description = "$([char]0x1b)[" + $parts[4] + "m" + $parts[2] + "$([char]0x1b)[0m"
        } else {
            $description = "$([char]0x1b)[" + $parts[4] + "m" + $parts[2] + "$([char]0x1b)[38;5;244m (" + $parts[3] + ")$([char]0x1b)[0m"
        }
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    }
}

@(__COMMANDS__) | 
    ForEach-Object {
        Register-ArgumentCompleter -Native -ScriptBlock $_argc_completer -CommandName $_
    }
