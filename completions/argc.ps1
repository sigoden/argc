using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'argc' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $argcfile = $(argc --argc-argcfile 2>$null)
    if (!$argcfile) {
        return;
    }
    $elms = $commandAst.CommandElements
    $cmdargs = $elms[1..($elms.length - 1)] -join " "
    $completions = (argc --argc-compgen "$argcfile" "$cmdargs") -split " " | % {
        $name = $_ -replace '^-+',''
        return [CompletionResult]::new($_, $name, [CompletionResultType]::ParameterName, '-')
    }
    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } | Sort-Object -Property ListItemText
}
