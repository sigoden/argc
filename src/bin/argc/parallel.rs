use anyhow::Result;
use argc::{NativeRuntime, Runtime};
use std::collections::HashMap;
use std::process::{self, Command};
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

pub const PARALLEL_SYMBOL: &str = "___parallel___";

pub fn parallel(
    runtime: NativeRuntime,
    shell: &str,
    script_file: &str,
    args: &[String],
) -> Result<()> {
    let jobs = to_jobs(args);
    let jobs_len = jobs.len();
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    let path_env = runtime.path_env_with_current_exe();
    let mut shell_extra_args = runtime.shell_args(shell);
    shell_extra_args.push(script_file.to_string());
    shell_extra_args.push(PARALLEL_SYMBOL.to_string());
    for (i, job_args) in jobs.into_iter().enumerate() {
        let tx = tx.clone();
        let shell = shell.to_string();
        let path_env = path_env.clone();
        let shell_extra_args = shell_extra_args.clone();
        pool.execute(move || {
            Command::new(shell)
                .args(shell_extra_args)
                .args(job_args)
                .env("ARGC_PARALLEL", "1")
                .env("PATH", path_env)
                .output()
                .ok()
                .and_then(|output| tx.send((i, output)).ok());
        });
    }
    pool.join();
    drop(tx);
    let mut job_outputs = HashMap::new();
    let mut exit = 0;
    for (i, job_output) in rx {
        if !job_output.status.success() {
            exit = 1;
        }
        job_outputs.insert(
            i,
            (
                String::from_utf8_lossy(&job_output.stdout).to_string(),
                String::from_utf8_lossy(&job_output.stderr).to_string(),
            ),
        );
    }
    for i in 0..jobs_len {
        if let Some((stdout, stderr)) = job_outputs.get(&i) {
            if !stdout.is_empty() {
                print!("{stdout}")
            }
            if !stderr.is_empty() {
                eprint!("{stderr}")
            }
        }
    }
    process::exit(exit)
}

fn to_jobs(args: &[String]) -> Vec<Vec<String>> {
    let mut jobs = Vec::new();
    let mut current = vec![];
    for arg in args {
        if arg == ":::" {
            if current.is_empty() {
                continue;
            }
            jobs.push(std::mem::take(&mut current).to_vec())
        } else {
            current.push(arg.to_string())
        }
    }
    if !current.is_empty() {
        jobs.push(std::mem::take(&mut current).to_vec())
    }
    jobs
}
