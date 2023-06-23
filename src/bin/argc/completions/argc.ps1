using namespace System.Management.Automation

function _argc_complete_impl([array]$array) {
    if (-not(Test-Path -Path $array[0] -PathType Leaf)) {
        return
    }
    $candidates = @((argc --argc-compgen powershell $array 2>$null).Split("`n"))
    if ($candidates.Count -eq 1) {
        if (($candidates[0] -eq "__argc_comp:file") -or ($candidates[0] -eq "__argc_comp:dir")) {
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
            $description = "$([char]0x1b)[92m" + $parts[2] + "$([char]0x1b)[0m"
        } else {
            $description = "$([char]0x1b)[92m" + $parts[2] + "$([char]0x1b)[0m" + "$([char]0x1b)[38;5;238m (" + $parts[3] + ")$([char]0x1b)[0m"
        }
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    }
}

function _argc_complete_locate($cmd) {
    if ($cmd -eq "argc") {
        argc --argc-script-path 2>$null
    } else {
        Get-Command $cmd -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
    }
}

function _argc_completer {
    param($wordToComplete, $commandAst, $cursorPosition)
    $array = @($commandAst.CommandElements | Where { $_.Extent.StartOffset -lt $cursorPosition } | ForEach-Object { $_.ToString() })
    if ($commandAst.CommandElements[-1].Extent.EndOffset -lt $cursorPosition) {
        $array += ''
    }
    $array = @(_argc_complete_locate $array[0]) + $array
    _argc_complete_impl $array
}
