use std::collections::HashMap;
use std::env;
use std::process;

use argc::NativeRuntime;

#[cfg(windows)]
use argc::utils::{
    ARGC_MSYS_NO_PATHCONV as SAVED, ARGC_MSYS_NO_PATHCONV_RESTORE as RESTORE, MSYS_NO_PATHCONV,
};
#[cfg(windows)]
use argc::Runtime;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::Path;

pub fn is_msys() -> bool {
    cfg!(windows) && env::var("MSYSTEM").is_ok()
}

/// Whether `shell` is the Git Bash executable used by argc on Windows.
#[cfg(windows)]
pub fn is_git_bash(runtime: &NativeRuntime, shell: &str) -> bool {
    let Some(bash_path) = runtime.bash_path() else {
        return false;
    };
    if paths_eq(shell, &bash_path) {
        return true;
    }
    let name = Path::new(shell)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    if !name.eq_ignore_ascii_case("bash.exe") && !name.eq_ignore_ascii_case("bash") {
        return false;
    }
    let Some(git_root) = Path::new(&bash_path).parent().and_then(|v| v.parent()) else {
        return false;
    };
    let git_root = normalize_path(&git_root.to_string_lossy()).to_ascii_lowercase();
    let shell = normalize_path(shell).to_ascii_lowercase();
    shell.starts_with(&format!("{}\\", git_root.trim_end_matches('\\')))
}

#[cfg(not(windows))]
pub fn is_git_bash(_runtime: &NativeRuntime, _shell: &str) -> bool {
    false
}

/// Save the caller's `MSYS_NO_PATHCONV` and disable path conversion for the
/// launched Git Bash process. Existing save state is preserved so nested
/// invocations keep restoring the original value.
#[cfg(windows)]
pub fn inject(envs: &mut HashMap<String, String>) {
    if env::var_os(RESTORE).is_none() {
        if let Some(value) = env::var_os(MSYS_NO_PATHCONV) {
            envs.insert(SAVED.to_string(), value.to_string_lossy().into_owned());
        }
        envs.insert(RESTORE.to_string(), "1".to_string());
    }
    envs.insert(MSYS_NO_PATHCONV.to_string(), "1".to_string());
}

#[cfg(not(windows))]
pub fn inject(_envs: &mut HashMap<String, String>) {}

pub fn envs() -> HashMap<String, String> {
    let mut envs = HashMap::new();
    inject(&mut envs);
    envs
}

/// Shell code prepended to `--argc-eval` output. It restores the caller's
/// `MSYS_NO_PATHCONV` before the command runs and keeps nested argc script
/// parsing (`--argc-eval`/`--argc-parallel`) conversion-free with a wrapper
/// function.
#[cfg(windows)]
pub fn restore_code() -> String {
    if env::var_os(RESTORE).is_none() {
        return String::new();
    }
    let mut code = String::new();
    if env::var_os(SAVED).is_some() {
        code.push_str("export MSYS_NO_PATHCONV=\"$ARGC_MSYS_NO_PATHCONV\"\n");
        code.push_str("export ARGC_MSYS_NO_PATHCONV\n");
    } else {
        code.push_str("unset MSYS_NO_PATHCONV\n");
    }
    code.push_str("export ARGC_MSYS_NO_PATHCONV_RESTORE\n");
    code.push_str("if ! declare -F argc >/dev/null 2>&1; then\n");
    code.push_str("    argc() {\n");
    code.push_str("        case \"${1-}\" in\n");
    code.push_str("            --argc-eval|--argc-parallel) MSYS_NO_PATHCONV=1 ARGC_MSYS_NO_PATHCONV_RESTORE=1 command argc \"$@\" ;;\n");
    code.push_str("            --argc-*) command argc \"$@\" ;;\n");
    code.push_str("            *) MSYS_NO_PATHCONV=1 ARGC_MSYS_NO_PATHCONV_RESTORE=1 command argc \"$@\" ;;\n");
    code.push_str("        esac\n");
    code.push_str("    }\n");
    code.push_str("    export -f argc\n");
    code.push_str("fi\n");
    code
}

#[cfg(not(windows))]
pub fn restore_code() -> String {
    String::new()
}

#[cfg(windows)]
fn paths_eq(a: &str, b: &str) -> bool {
    match (Path::new(a).canonicalize(), Path::new(b).canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => normalize_path(a).eq_ignore_ascii_case(&normalize_path(b)),
    }
}

#[cfg(windows)]
fn normalize_path(path: &str) -> String {
    path.replace('/', "\\")
}

pub fn cygpath_w(path: &str) -> Option<String> {
    let output = process::Command::new("cygpath")
        .args(["-w", path])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(windows)]
pub fn create_windows_shim(runtime: &NativeRuntime, script_path: &str) -> Option<String> {
    if Path::new(script_path)
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("sh"))
        .unwrap_or(false)
    {
        return None;
    }
    let shim_path = format!("{script_path}.cmd");
    if Path::new(&shim_path).exists() {
        return None;
    }
    let bash_path = runtime.bash_path()?;
    let name = Path::new(script_path).file_name()?.to_str()?;
    let content = format!(
        "@setlocal DisableDelayedExpansion\r\n\
@if not defined {restore} set \"{saved}=%{msys}%\"\r\n\
@if not defined {restore} set \"{restore}=1\"\r\n\
@set \"{msys}=1\"\r\n\
@\"{bash_path}\" --noprofile --norc \"%~dp0{name}\" %*\r\n\
@exit /b %ERRORLEVEL%\r\n",
        saved = SAVED,
        restore = RESTORE,
        msys = MSYS_NO_PATHCONV,
        bash_path = bash_path,
        name = name,
    );
    fs::write(&shim_path, content).ok()?;
    Some(shim_path)
}
