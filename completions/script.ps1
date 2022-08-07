# Powershell completion for scripts written with argc
# All argc scripts share the same completion function, put your scripts to $PATH, replace `mycmd1,mycmd2` blow with your scripts' names

$_argc_script_completion = {
    param($wordToComplete, $commandAst, $cursorPosition)
    $argcfile = (Get-Command $commandAst.CommandElements[0]  -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
    if (!$argcfile) {
        return;
    }
    if ($wordToComplete) {
        $cmds = $commandAst.CommandElements[1..($commandAst.CommandElements.Count - 2)]
    } else {
        $cmds = $commandAst.CommandElements[1..($commandAst.CommandElements.Count - 1)]
    }
    (argc --argc-compgen "$argcfile" $cmds 2>$null) -split " " | 
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