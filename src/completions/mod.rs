use crate::Shell;

const BASH_SCRIPT: &str = include_str!("argc.bash");
const FISH_SCRIPT: &str = include_str!("argc.fish");
const NUSHELL_SCRIPT: &str = include_str!("argc.nu");
const POWERSHELL_SCRIPT: &str = include_str!("argc.ps1");
const ZSH_SCRIPT: &str = include_str!("argc.zsh");

pub fn generate_completions(shell: Shell, commands: &[String]) -> String {
    match shell {
        Shell::Bash => {
            let commands = commands.join(" ");
            BASH_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Fish => {
            let commands = commands.join(" ");
            FISH_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Generic => String::new(),
        Shell::Nushell => NUSHELL_SCRIPT.to_string(),
        Shell::Powershell => {
            let commands = commands
                .iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(",");
            POWERSHELL_SCRIPT.replace("__COMMANDS__", &commands)
        }
        Shell::Zsh => {
            let commands = commands.join(" ");
            ZSH_SCRIPT.replace("__COMMANDS__", &commands)
        }
    }
}
