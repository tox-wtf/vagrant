// utils/sh.rs

use std::collections::HashMap;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::Duration;

use color_eyre::Result;
use color_eyre::eyre::Context;
use thiserror::Error;
use tracing::{trace, warn};
use wait_timeout::ChildExt;

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
#[allow(clippy::similar_names)]
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

    let Some(status) = child.wait_timeout(Duration::from_secs(32)).expect("Failed to wait on child") else {
        child.kill().expect("Could not kill child");
        child.wait().expect("Failed to wait on child");
        return Err(CmdError::Timeout).wrap_err("Timed out");
    };

    let code = status.code().unwrap_or(1);

    let mut out_buf = Vec::new();
    child.stdout.as_mut().expect("Handle present").read_to_end(&mut out_buf)?;
    let out = String::from_utf8_lossy(&out_buf).to_string();

    let mut err_buf = Vec::new();
    child.stderr.as_mut().expect("Handle present").read_to_end(&mut err_buf)?;
    let err = String::from_utf8_lossy(&err_buf).to_string();

    trace!("{out}");

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
        return Err(CmdError::NonzeroStatus).wrap_err_with(|| format!("Exited with nonzero status: {code}"));
    }

    Ok(out)
}
