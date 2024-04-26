use super::Runtime;

use anyhow::{anyhow, bail, Result};
use std::{env, fs, path::Path, process, thread};

#[derive(Debug, Clone, Copy, Default)]
pub struct NativeRuntime;

impl Runtime for NativeRuntime {
    fn os(&self) -> String {
        env::consts::OS.to_string()
    }

    fn shell_path(&self) -> Result<String> {
        match self.env_var("ARGC_SHELL_PATH") {
            Some(shell_path) => {
                if !self.exist_path(&shell_path) {
                    bail!("Invalid ARGC_SHELL_PATH, '{shell_path}' does not exist",);
                }
                Ok(shell_path)
            }
            None => self.bash_path().ok_or_else(|| anyhow!("Shell not found")),
        }
    }

    #[cfg(windows)]
    fn bash_path(&self) -> Option<String> {
        let bash_path = "C:\\Program Files\\Git\\bin\\bash.exe";
        if self.exist_path(bash_path) {
            return Some(bash_path.into());
        }
        let git_path = self.which("git")?;
        let git_parent_path = self.parent_path(&git_path)?;
        let bash_path = self.join_path(&self.parent_path(&git_parent_path)?, &["bin", "bash.exe"]);
        if self.exist_path(&bash_path) {
            return Some(bash_path);
        }
        let bash_path = self.join_path(&git_parent_path, &["bash.exe"]);
        if self.exist_path(&bash_path) {
            return Some(bash_path);
        }
        None
    }

    #[cfg(not(windows))]
    fn bash_path(&self) -> Option<String> {
        self.which("bash")
    }

    fn exec_bash_functions(
        &self,
        script_file: &str,
        functions: &[&str],
        args: &[String],
        envs: std::collections::HashMap<String, String>,
    ) -> Option<Vec<String>> {
        let shell = self.shell_path().ok()?;
        let shell_args = self.shell_args(&shell);
        let path_env = self.path_env_with_current_exe();
        let handles: Vec<_> = functions
            .iter()
            .map(|func| {
                let script_file = script_file.to_string();
                let args: Vec<String> = args.to_vec();
                let path_env = path_env.clone();
                let func = func.to_string();
                let shell = shell.clone();
                let shell_args = shell_args.clone();
                let envs = envs.clone();
                thread::spawn(move || {
                    process::Command::new(shell)
                        .args(shell_args)
                        .arg(&script_file)
                        .arg(Self::INTERNAL_SYMBOL)
                        .arg(&func)
                        .args(args)
                        .envs(envs)
                        .env("PATH", path_env)
                        .output()
                        .ok()
                        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
                        .unwrap_or_default()
                })
            })
            .collect();
        let result: Vec<String> = handles
            .into_iter()
            .map(|h| {
                h.join()
                    .ok()
                    .map(|v| v.trim().to_string())
                    .unwrap_or_default()
            })
            .collect();
        Some(result)
    }

    fn current_exe(&self) -> Option<String> {
        env::current_exe()
            .ok()
            .map(|path| path.to_string_lossy().into())
    }

    fn current_dir(&self) -> Option<String> {
        env::current_dir()
            .ok()
            .map(|path| path.to_string_lossy().into())
    }

    fn env_vars(&self) -> std::collections::HashMap<String, String> {
        env::vars().collect()
    }

    fn env_var(&self, name: &str) -> Option<String> {
        env::var(name).ok()
    }

    fn which(&self, name: &str) -> Option<String> {
        which::which(name)
            .ok()
            .map(|path| path.to_string_lossy().into())
    }

    fn exist_path(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn parent_path(&self, path: &str) -> Option<String> {
        Path::new(path)
            .parent()
            .map(|path| path.to_string_lossy().into())
    }

    fn join_path(&self, path: &str, parts: &[&str]) -> String {
        let mut path = Path::new(path).to_path_buf();
        for part in parts {
            path = path.join(part);
        }
        path.to_string_lossy().into()
    }

    fn chdir(&self, cwd: &str, cd: &str) -> Option<String> {
        let path = Path::new(cwd).join(cd).canonicalize().ok()?;
        Some(path.to_string_lossy().into())
    }

    fn metadata(&self, path: &str) -> Option<(bool, bool, bool)> {
        let mut meta = fs::symlink_metadata(path).ok()?;
        let is_symlink = meta.is_symlink();
        if is_symlink {
            meta = fs::metadata(path).ok()?;
        }
        let is_dir = meta.is_dir();
        #[cfg(target_family = "unix")]
        let is_executable = {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(target_family = "unix"))]
        let is_executable = false;
        Some((is_dir, is_symlink, is_executable))
    }

    fn read_dir(&self, path: &str) -> Option<Vec<String>> {
        let dir = fs::read_dir(path).ok()?;
        let mut paths = vec![];
        for entry in dir {
            let entry = entry.ok()?;
            paths.push(entry.file_name().to_string_lossy().into());
        }
        Some(paths)
    }

    fn read_to_string(&self, path: &str) -> Option<String> {
        let data = fs::read_to_string(path).ok()?;
        Some(data)
    }
}
