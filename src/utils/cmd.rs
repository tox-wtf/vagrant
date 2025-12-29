// utils/sh.rs

use std::collections::HashMap;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;

use color_eyre::Result;
use color_eyre::eyre::Context;
use thiserror::Error;
use wait_timeout::ChildExt;

use crate::CONFIG;

#[derive(Error, Debug)]
enum CmdError {
    #[error("output in stderr")]
    OutputInStderr,

    #[error("nonzero status")]
    NonzeroStatus,

    #[error("empty stdout")]
    EmptyStdout,

    #[error("timeout")]
    Timeout,

    #[error("io error")]
    Io(#[from] io::Error),
}

/// # Lowish level function to execute a command and return stdout
#[allow(clippy::similar_names, clippy::unwrap_used)]
pub fn cmd(cmd: &[&str], env: HashMap<&str, &str>, cwd: &str) -> Result<String> {
    trace!("Evaluating command: {}", cmd.join(" "));

    let (arg0, args) = cmd.split_first().expect("command should not be empty");
    let mut child = Command::new(arg0)
        .args(args)
        .envs(env)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .wrap_err("Failed to spawn command")?;

    let timeout = CONFIG.get().expect("Config should be initialized").fetch_timeout;
    trace!("Spawned command with a timeout of {timeout} seconds");

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let out_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).unwrap();
        buf
    });

    let err_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        stderr.read_to_end(&mut buf).unwrap();
        buf
    });

    let timeout = Duration::from_secs(timeout);
    let Some(status) = child.wait_timeout(timeout)? else {
        child.kill().expect("Could not kill child");
        child.wait().expect("Failed to wait on child");
        return Err(CmdError::Timeout).wrap_err("Timed out");
    };

    let out_buf = out_thread.join().unwrap();
    let err_buf = err_thread.join().unwrap();

    let code = status.code().unwrap_or(1);
    trace!("Command exited with code {code}");

    let out = String::from_utf8_lossy(&out_buf).to_string();
    let err = String::from_utf8_lossy(&err_buf).to_string();

    trace!("Received output in stdout:\n{out}");

    if !err.is_empty() {
        warn!("{err}");
        return Err(CmdError::OutputInStderr).wrap_err("Output in stderr");
    }

    if out.trim().is_empty() {
        warn!("No output in stdout");
        return Err(CmdError::EmptyStdout).wrap_err("No output in stdout");
    }

    if code != 0 {
        warn!("Exited with nonzero status: {code}");
        return Err(CmdError::NonzeroStatus)
            .wrap_err_with(|| format!("Exited with nonzero status: {code}"));
    }

    Ok(out)
}
