use argc::Shell;
use std::env;

const BASH_SCRIPT: &str = include_str!("argc.bash");

const ZSH_SCRIPT: &str = include_str!("argc.zsh");

const POWERSHELL_SCRIPT: &str = include_str!("argc.ps1");

const FISH_SCRIPT: &str = include_str!("argc.fish");

const ELVISH_SCRIPT: &str = include_str!("argc.elv");

const NUSHELL_SCRIPT: &str = include_str!("argc.nu");

const XONSH_SCRIPT: &str = include_str!("argc.xsh");

pub fn generate(shell: Shell, args: &[String]) -> String {
    let mut cmds = vec!["argc".to_string()];
    cmds.extend(args.to_vec());

    match shell {
        Shell::Bash => {
            let code = format!(
                "complete -F _argc_completer -o nospace -o nosort {}",
                cmds.join(" ")
            );
            format!("{BASH_SCRIPT}\n{code}")
        }
        Shell::Elvish => {
            let code = cmds
                .iter()
                .map(|v| format!("set edit:completion:arg-completer[{v}] = $argc-completer~"))
                .collect::<Vec<String>>()
                .join("\n");
            format!("{ELVISH_SCRIPT}\n{code}")
        }
        Shell::Fish => {
            let code = cmds
                .iter()
                .map(|v| format!("complete -x -k -c {v} -a \"(_argc_completer)\""))
                .collect::<Vec<String>>()
                .join("\n");
            format!("{FISH_SCRIPT}\n{code}")
        }
        Shell::Generic => String::new(),
        Shell::Nushell => {
            let scripts_env_var = "ARGC_NUSHELL_SCRIPTS";
            let code = if env::var(scripts_env_var).is_ok() {
                format!("$env.{scripts_env_var} = $env.{scripts_env_var} ++ {cmds:?}")
            } else {
                format!("$env.{scripts_env_var} = {cmds:?}")
            };
            format!("{NUSHELL_SCRIPT}\n{code}")
        }
        Shell::Powershell => {
            let code = cmds.iter().map(|v| format!("Register-ArgumentCompleter -Native -ScriptBlock $_argc_completer -CommandName {v}")).collect::<Vec<String>>().join("\n");
            format!("{POWERSHELL_SCRIPT}\n{code}")
        }
        Shell::Xonsh => {
            let scripts_env_var = "ARGC_XONSH_SCRIPTS";
            let code = if env::var(scripts_env_var).is_ok() {
                format!("__xonsh__.env['{scripts_env_var}'].extend({cmds:?})")
            } else {
                format!("__xonsh__.env['{scripts_env_var}'] = {cmds:?}")
            };
            format!("{XONSH_SCRIPT}\n{code}")
        }
        Shell::Zsh => {
            let code = format!("compdef _argc_completer {}", cmds.join(" "));
            format!("{ZSH_SCRIPT}\n{code}")
        }
    }
}

#[test]
fn feature() {
    format!("{:?}", vec!["a", "b"]);
}
