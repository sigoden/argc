using namespace System.Management.Automation

function _argc_complete_impl([array]$words) {
    $candidates = @((argc --argc-compgen powershell $words 2>$null).Split("`n"))
    if ($candidates.Count -eq 1) {
        if (($candidates[0] -eq "__argc_value:file") -or ($candidates[0] -eq "__argc_value:dir")) {
            return
        } elseif ($candidates[0] -eq "") {
            return ""
        }
    }
    $candidates | ForEach-Object { 
        $parts = ($_ -split "`t")
        $value = $parts[0]
        $description = ""
        if ($parts[1] -eq "1") {
            $value = $value + " "
        }
        if ($parts[3] -eq "") {
            $description = $parts[2]
        } else {
            $description = $parts[2] + "$([char]0x1b)[38;5;238m (" + $parts[3] + ")$([char]0x1b)[0m"
        }
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    }
}

$_argc_completer = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $words = @($commandAst.CommandElements | Where { $_.Extent.StartOffset -lt $cursorPosition } | ForEach-Object { $_.ToString() })
    if ($commandAst.CommandElements[-1].Extent.EndOffset -lt $cursorPosition) {
        $words += ''
    }
    $scriptfile = ''
    if ($words[0] -eq "argc") {
        $scriptfile = (argc --argc-script-path 2>$null)
    } else {
        $scriptfile = (Get-Command $words[0] -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
    }
    if (-not(Test-Path -Path $scriptfile -PathType Leaf)) {
        return
    }
    $words = @($scriptfile) + $words
    _argc_complete_impl $words
}
