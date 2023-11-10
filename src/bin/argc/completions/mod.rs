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
    let mut cmds = args.to_vec();
    let completion_shell = format!("ARGC_COMPLETION_{}", shell.name().to_uppercase());
    let exist_completion = std::env::var(&completion_shell).ok().unwrap_or_default();
    let append_mode = exist_completion == "1";
    if !append_mode {
        cmds.insert(0, "argc".to_string());
    }
    let mut share_script = String::new();
    let mut cmds_code = String::new();
    match shell {
        Shell::Bash => {
            share_script = format!("{BASH_SCRIPT}\nexport {completion_shell}=1\n");
            cmds_code = format!(
                "complete -F _argc_completer -o nospace -o nosort {}",
                cmds.join(" ")
            );
        }
        Shell::Elvish => {
            share_script = format!("{ELVISH_SCRIPT}\nset E:{completion_shell} = 1\n");
            cmds_code = cmds
                .iter()
                .map(|v| {
                    if append_mode {
                        format!("set edit:completion:arg-completer[{v}] = $edit:completion:arg-completer[argc]")
                    } else {
                        format!("set edit:completion:arg-completer[{v}] = $argc-completer~")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
        }
        Shell::Fish => {
            share_script = format!("{FISH_SCRIPT}\nset -gx {completion_shell} 1\n");
            cmds_code = cmds
                .iter()
                .map(|v| format!("complete -x -k -c {v} -a \"(_argc_completer)\""))
                .collect::<Vec<String>>()
                .join("\n");
        }
        Shell::Generic => {}
        Shell::Nushell => {
            share_script = format!("{NUSHELL_SCRIPT}\n$env.{completion_shell} = 1\n");
            if append_mode {
                cmds_code = format!("$env.ARGC_SCRIPTS = $env.ARGC_SCRIPTS ++ {cmds:?}");
            } else {
                cmds_code = format!("$env.ARGC_SCRIPTS = {cmds:?}");
            }
        }
        Shell::Powershell => {
            share_script = format!("{POWERSHELL_SCRIPT}\n$env:{completion_shell} = 1\n");
            cmds_code = cmds.iter().map(|v| format!("Register-ArgumentCompleter -Native -ScriptBlock $_argc_completer -CommandName {v}")).collect::<Vec<String>>().join("\n");
        }
        Shell::Xonsh => {
            share_script = format!("{XONSH_SCRIPT}\n${completion_shell} = 1\n");
            if append_mode {
                cmds_code = format!("ARGC_SCRIPTS.extend({cmds:?})");
            } else {
                cmds_code = format!("ARGC_SCRIPTS = {cmds:?}");
            }
        }
        Shell::Zsh => {
            share_script = format!("{ZSH_SCRIPT}\nexport {completion_shell}=1\n");
            cmds_code = format!("compdef _argc_completer {}", cmds.join(" "));
        }
    };
    if append_mode {
        if cmds.is_empty() {
            return Ok(String::new());
        }
        Ok(cmds_code.to_string())
    } else {
        Ok(format!("{share_script}{cmds_code}"))
    }
}

#[test]
fn feature() {
    format!("{:?}", vec!["a", "b"]);
}
