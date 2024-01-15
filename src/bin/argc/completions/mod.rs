use argc::Shell;
use semver::Version;
use std::env;

const BASH_SCRIPT: &str = include_str!("argc.bash");
const ELVISH_SCRIPT: &str = include_str!("argc.elv");
const FISH_SCRIPT: &str = include_str!("argc.fish");
const NUSHELL_SCRIPT: &str = include_str!("argc.nu");
const POWERSHELL_SCRIPT: &str = include_str!("argc.ps1");
const XONSH_SCRIPT: &str = include_str!("argc.xsh");
const ZSH_SCRIPT: &str = include_str!("argc.zsh");

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
        Shell::Nushell => {
            if env::var("NU_VERSION")
                .ok()
                .and_then(|v| Version::parse(&v).ok())
                .map(|v| v < Version::new(0, 89, 0))
                .unwrap_or_default()
            {
                // https://github.com/nushell/nushell/pull/11289
                NUSHELL_SCRIPT.replace("...$args", "$args")
            } else {
                NUSHELL_SCRIPT.to_string()
            }
        }
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
        Shell::Tcsh => {
            let commands = [vec!["argc".to_string()], args.to_vec()].concat();
            commands
                .into_iter()
                .map(|v| format!(r#"complete {v} 'p@*@`echo "$COMMAND_LINE'"''"'" | xargs argc --argc-compgen tcsh ""`@@';{}"#, "\n"))
                .collect::<Vec<String>>().join("")
        }
    }
}
