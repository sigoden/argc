using namespace System.Management.Automation

$_argc_completer = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $words = @($commandAst.CommandElements | Where { $_.Extent.StartOffset -lt $cursorPosition } | ForEach-Object { $_.ToString() })
    $emptyS = ''
    if ($PSVersionTable.PSVersion.Major -eq 5) {
        $emptyS = '""'
    }
    if ($commandAst.CommandElements[-1].Extent.EndOffset -lt $cursorPosition) {
        $words += $emptyS
    }
    @((argc --argc-compgen powershell $emptyS $words) -split "`n") | ForEach-Object { 
        $parts = ($_ -split "`t")
        $value = (if ($parts[1] -eq "1") { $parts[0] + " " } else { $parts[0] })
        $description = if ($parts[3] -eq "") {
            $description = "$([char]0x1b)[" + $parts[4] + "m" + $parts[2] + "$([char]0x1b)[0m"
        } else {
            $description = "$([char]0x1b)[" + $parts[4] + "m" + $parts[2] + "$([char]0x1b)[38;5;244m (" + $parts[3] + ")$([char]0x1b)[0m"
        }
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    }
}
