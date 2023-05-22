use anyhow::Result;

use argc::Shell;

const BASH_SCRIPT: &str = include_str!("argc.bash");

const ZSH_SCRIPT: &str = include_str!("argc.zsh");

const POWERSHELL_SCRIPT: &str = include_str!("argc.ps1");

const FISH_SCRIPT: &str = include_str!("argc.fish");

const ELVISH_SCRIPT: &str = include_str!("argc.elv");

const NUSHELL_SCRIPT: &str = include_str!("argc.nu");

pub fn generate(shell: Shell, args: &[String]) -> Result<String> {
    let mut cmds = vec!["argc"];
    cmds.extend(args.iter().map(|v| v.as_str()));
    let output = match shell {
        Shell::Bash => {
            let code = format!("complete -F _argc_complete {}", cmds.join(" "));
            format!("{BASH_SCRIPT}\n{code}\n",)
        }
        Shell::Zsh => {
            let code = format!("compdef _argc_complete {}", cmds.join(" "));
            format!("{ZSH_SCRIPT}\n{code}\n",)
        }
        Shell::Powershell => {
            let lines: Vec<String> = cmds.iter().map(|v| format!("Register-ArgumentCompleter -Native -ScriptBlock $_argc_complete -CommandName {v} ")).collect();
            let code = lines.join("\n");
            format!("{POWERSHELL_SCRIPT}\n{code}\n",)
        }
        Shell::Fish => {
            let lines: Vec<String> = cmds
                .iter()
                .map(|v| format!(r#"complete -x -c {v} -a "(_argc_complete)""#))
                .collect();
            let code = lines.join("\n");
            format!("{FISH_SCRIPT}\n{code}\n",)
        }
        Shell::Elvish => {
            let lines: Vec<String> = cmds
                .iter()
                .map(|v| format!(r#"set edit:completion:arg-completer[{v}] = $argc-complete~"#))
                .collect();
            let code = lines.join("\n");
            format!("{ELVISH_SCRIPT}\n{code}\n",)
        }
        Shell::Nushell => {
            let code = format!("{cmds:?}");
            format!(
                r###"{NUSHELL_SCRIPT}

let argc_scripts = {code}

let external_completer = {{|spans| 
    if (not ($argc_scripts | find $spans.0 | is-empty)) {{
        argc_complete $spans
    }} else {{
        # default completer
    }}
}}

let-env config = {{
  completions: {{
    external: {{
      enable: true
      completer: $external_completer
    }}
  }}
}}
"###,
            )
        }
    };
    Ok(output)
}

#[test]
fn feature() {
    format!("{:?}", vec!["a", "b"]);
}
