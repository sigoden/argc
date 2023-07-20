use anyhow::Result;

use argc::Shell;

const BASH_SCRIPT: &str = include_str!("argc.bash");

const ZSH_SCRIPT: &str = include_str!("argc.zsh");

const POWERSHELL_SCRIPT: &str = include_str!("argc.ps1");

const FISH_SCRIPT: &str = include_str!("argc.fish");

const ELVISH_SCRIPT: &str = include_str!("argc.elv");

const NUSHELL_SCRIPT: &str = include_str!("argc.nu");

const XONSH_SCRIPT: &str = include_str!("argc.xsh");

pub fn generate(shell: Shell, args: &[String]) -> Result<String> {
    let mut cmds = vec!["argc"];
    cmds.extend(args.iter().map(|v| v.as_str()));
    let output = match shell {
        Shell::Bash => {
            let code = format!(
                "complete -F _argc_completer -o nospace -o nosort {}",
                cmds.join(" ")
            );
            format!("{BASH_SCRIPT}\n{code}\n",)
        }
        Shell::Elvish => {
            let lines: Vec<String> = cmds
                .iter()
                .map(|v| format!(r#"set edit:completion:arg-completer[{v}] = $argc-completer~"#))
                .collect();
            let code = lines.join("\n");
            format!("{ELVISH_SCRIPT}\n{code}\n",)
        }
        Shell::Fish => {
            let lines: Vec<String> = cmds
                .iter()
                .map(|v| format!(r###"complete -x -k -c {v} -a "(_argc_completer)""###))
                .collect();
            let code = lines.join("\n");
            format!("{FISH_SCRIPT}\n{code}\n",)
        }
        Shell::Generic => String::new(),
        Shell::Nushell => {
            let cmds = format!("{cmds:?}");
            format!(
                r###"{NUSHELL_SCRIPT}
if ('ARGC_SCRIPTS' in $env) {{
    let-env ARGC_SCRIPTS = ($env.ARGC_SCRIPTS | append {cmds} | uniq)
}} else {{
    let-env ARGC_SCRIPTS = {cmds} 
}}

let external_completer = {{|spans| 
    if (not ($env.ARGC_SCRIPTS | find $spans.0 | is-empty)) {{
        _argc_completer $spans
    }} else {{
        # default completer
    }}
}}

$env.config.completions.external = {{
    enable: true
    max_results: 100
    completer: $external_completer
}}
"###,
            )
        }
        Shell::Powershell => {
            let lines: Vec<String> = cmds.iter().map(|v| format!("Register-ArgumentCompleter -Native -ScriptBlock $_argc_completer -CommandName {v} ")).collect();
            let code = lines.join("\n");
            format!("{POWERSHELL_SCRIPT}\n{code}\n",)
        }
        Shell::Xonsh => {
            let cmds = format!("{cmds:?}");
            format!(
                r###"{XONSH_SCRIPT}
if 'ARGC_SCRIPTS' in globals():
    ARGC_SCRIPTS.extend(item for item in {cmds} 
        if item not in ARGC_SCRIPTS)
else:
    ARGC_SCRIPTS = {cmds} 
"###,
            )
        }
        Shell::Zsh => {
            let code = format!("compdef _argc_completer {}", cmds.join(" "));
            format!("{ZSH_SCRIPT}\n{code}\n",)
        }
    };
    Ok(output)
}

#[test]
fn feature() {
    format!("{:?}", vec!["a", "b"]);
}
