using namespace System.Management.Automation

function _argc_complete_path([string]$cur, [bool]$is_dir) {
    $prefix = ''
    $quoted = $false
    if ($cur.StartsWith('"') -or $cur.StartsWith("'")) {
        $prefix = $prefix + $cur.SubString(0, 1)
        $quoted = $true
        $cur = $cur.SubString(1)
    }
    if ($cur.StartsWith('~') -or $cur.StartsWith('/') -or $cur.StartsWith('\')) {
        $cur = (Resolve-Path $cur.SubString(0, 1)).Path + $cur.SubString(1)
    }
    if ($cur -eq "") {
        $cur = ".\"
    }
    $cur = $cur -replace '/','\'
    if ($cur.Contains('\')) {
        $prefix = $prefix + ($cur -replace '\[^\]+$','\')
    }
    $paths = @()
    if ($is_dir) {
        $paths = (Get-ChildItem -Attributes Directory -Path "$cur*")
    } else {
        $paths = (Get-ChildItem -Path "$cur*")
    }

    $paths | ForEach-Object {
        $name = $_.Name
        $file = $true
        if ($_.Attributes -band [System.IO.FileAttributes]::Directory) {
            $name = $name + '\'
            $file = $false
        }
        $value = $prefix + $name
        if (-not($quoted)) {
            if ($value -match '[()<>\[\]{}"` #$&,;@|]') {
                $value = "'" + $value + "'"
            }
            if ($file) {
                $value = $value + ' '
            }
        }
        $description = $name
        [CompletionResult]::new($value, $description, [CompletionResultType]::ParameterValue, " ")
    } 
}

function _argc_complete_impl([array]$words) {
    $candidates = @((argc --argc-compgen powershell $words 2>$null) -split "`n")
    if ($candidates.Count -eq 0) {
        return ""
    }
    $skip = 0
    $paths = @()
    if ($candidates.Count -gt 0) {
        if ($candidates[0] -eq "__argc_value:file") {
            $skip = 1
            $paths = (_argc_complete_path $words[-1] $false)
        } elseif ($candidates[0] -eq "__argc_value:dir") {
            $skip = 1
            $paths = (_argc_complete_path $words[-1] $true)
        }
    }
    $candidates = ($candidates | Select-Object -Skip $skip | ForEach-Object { 
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
    })
    $paths + $candidates
}

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
    if ($words[0] -eq "argc") {
        $scriptfile = (argc --argc-script-path 2>$null)
    } else {
        $scriptfile = (Get-Command $words[0] -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
        if (-not($scriptfile)) {
            return (_argc_complete_path $words[-1] $false)
        }
    }
    if (-not($scriptfile)) {
        $scriptfile = $emptyS
    }
    $words = @($scriptfile) + $words
    _argc_complete_impl $words
}
