# Powershell completion for scripts written with argc
#
# All argc scripts share the same completion function.
# To add completion for a  argc script, you just need:
# 1. put your scripts to $PATH
# 2. add your script name to $ARGC_SCRIPTS

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
    if ($wordToComplete) {
        $cmds = $commandAst.CommandElements[1..($commandAst.CommandElements.Count - 2)]
    } else {
        $cmds = $commandAst.CommandElements[1..($commandAst.CommandElements.Count - 1)]
    }
    (argc --compgen "$argcfile" $cmds 2>$null) -split " " | 
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

Register-ArgumentCompleter -Native -ScriptBlock $_argc_completion -CommandName $ARGC_SCRIPTS