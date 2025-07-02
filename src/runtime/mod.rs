#[cfg(feature = "native-runtime")]
pub mod navite;

use anyhow::Result;
use std::{collections::HashMap, env};

pub trait Runtime
where
    Self: Copy + Clone,
{
    const INTERNAL_SYMBOL: &'static str = "___internal___";
    fn os(&self) -> String;
    fn shell_path(&self) -> Result<String>;
    fn bash_path(&self) -> Option<String>;
    fn exec_bash_functions(
        &self,
        script_file: &str,
        functions: &[&str],
        args: &[String],
        envs: HashMap<String, String>,
    ) -> Option<Vec<String>>;
    fn current_exe(&self) -> Option<String>;
    fn current_dir(&self) -> Option<String>;
    fn env_vars(&self) -> HashMap<String, String>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn which(&self, name: &str) -> Option<String>;
    fn exist_path(&self, path: &str) -> bool;
    fn parent_path(&self, path: &str) -> Option<String>;
    fn join_path(&self, path: &str, parts: &[&str]) -> String;
    fn chdir(&self, cwd: &str, cd: &str) -> Option<String>;
    fn metadata(&self, path: &str) -> Option<(bool, bool, bool)>;
    fn read_dir(&self, path: &str) -> Option<Vec<String>>;
    fn read_to_string(&self, path: &str) -> Option<String>;

    fn is_windows(&self) -> bool {
        self.os() == "windows"
    }

    fn shell_args(&self, shell_path: &str) -> Vec<String> {
        if let Some(name) = self.basename(shell_path).map(|v| v.to_lowercase()) {
            match name.as_str() {
                "bash" => vec!["--noprofile".to_string(), "--norc".to_string()],
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn basename(&self, path: &str) -> Option<String> {
        let parts: Vec<_> = path.split(['/', '\\']).collect();
        let last_part = parts.last()?;
        let name = match last_part.rsplit_once('.') {
            Some((v, _)) => v.to_string(),
            None => last_part.to_string(),
        };
        Some(name)
    }

    fn load_dotenv(&self, path: &str) -> Option<HashMap<String, String>> {
        let contents = self.read_to_string(path)?;
        let mut output = HashMap::new();
        for line in contents.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let env_name = key.trim().to_string();
                let env_value = value.trim().to_string();
                let env_value = if (env_value.starts_with('"') && env_value.ends_with('"'))
                    || (env_value.starts_with('\'') && env_value.ends_with('\''))
                {
                    &env_value[1..env_value.len() - 1]
                } else {
                    &env_value
                };

                if env::var(&env_name).is_err() {
                    output.insert(env_name, env_value.to_string());
                }
            }
        }
        Some(output)
    }

    fn path_env_with_current_exe(&self) -> String {
        let mut path_env = self.env_var("PATH").unwrap_or_default();
        if let Some(exe_dir) = self
            .current_exe()
            .and_then(|exe_path| self.parent_path(&exe_path))
        {
            if self.is_windows() {
                path_env = format!("{exe_dir};{path_env}")
            } else {
                path_env = format!("{exe_dir}:{path_env}")
            }
        }
        path_env
    }
}
