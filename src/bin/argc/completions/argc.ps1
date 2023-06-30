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
    @((argc --argc-compgen powershell $emptyS $words) -split "`n") | Select-Object -Skip $skip | ForEach-Object { 
        $parts = ($_ -split "`t")
        $value = $parts[0]
        $description = ""
        if ($parts[1] -eq "1") {
            $value = $value + " "
        }
        if ($parts[3] -eq "") {
            $description = $parts[2]
        } else {
            $description = $parts[2] + "$([char]0x1b)[38;5;244m (" + $parts[3] + ")$([char]0x1b)[0m"
        }
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    }
}
