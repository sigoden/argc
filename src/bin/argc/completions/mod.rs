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
    match shell {
        Shell::Bash => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            let commands = commands.join(" ");
            BASH_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Elvish => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            let commands = commands
                .into_iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(" ");
            ELVISH_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Fish => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            let commands = commands.join(" ");
            FISH_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Generic => String::new(),
        Shell::Nushell => NUSHELL_SCRIPT.to_string(),
        Shell::Powershell => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            let commands = commands
                .into_iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(",");
            POWERSHELL_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Xonsh => {
            let mut cmds = args.to_vec();
            let scripts_env_var = "ARGC_XONSH_SCRIPTS";
            if env::var(scripts_env_var).is_ok() {
                format!("__xonsh__.env['{scripts_env_var}'].extend({cmds:?})")
            } else {
                cmds.insert(0, "argc".to_string());
                let code = format!("__xonsh__.env['{scripts_env_var}'] = {cmds:?}");
                format!("{XONSH_SCRIPT}\n{code}")
            }
        }
        Shell::Zsh => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            let commands = commands.join(" ");
            ZSH_SCRIPT.replace("__COMMANDS__", &commands)
        }
    }
}

#[test]
fn feature() {
    format!("{:?}", vec!["a", "b"]);
}
