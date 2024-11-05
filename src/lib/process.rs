use std::{
    os::unix::process::CommandExt, sync::mpsc, thread, time::Duration,
};

use anyhow::{anyhow, Result};

pub fn spawn(
    cmd: &str,
    args: &[&str],
) -> Result<impl Iterator<Item = Result<String, std::io::Error>>> {
    let stdout = std::process::Command::new("stdbuf")
        .args(["-o", "L"])
        .arg(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn {cmd:?}: {e:?}"))?
        .stdout
        .ok_or_else(|| anyhow!("Failed to get stdout of: {:?}", cmd))?;
    let lines = {
        use std::io::BufRead;
        std::io::BufReader::new(stdout).lines()
    };
    Ok(lines)
}

pub fn exec(cmd: &str, args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new(cmd).args(args).output()?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        let stderr = String::from_utf8(out.stderr.clone());
        tracing::error!(
            ?cmd,
            ?args,
            ?out,
            ?stderr,
            "Failed to execute command."
        );
        Err(anyhow!("Failure in '{:?} {:?}'. out: {:?}", cmd, args, out))
    }
}

pub fn exec_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> anyhow::Result<Vec<u8>> {
    // For error messages:
    let cmd_str = format!("{cmd:?}");
    let args_str = format!("{args:?}");

    let child = std::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        // XXX Sets PGID to PID. So we can kill as group (with any children).
        .process_group(0)
        .spawn()?;
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    thread::spawn({
        let cmd_str = cmd_str.clone();
        let args_str = args_str.clone();
        move || {
            let result = child
            .wait_with_output()
            .map_err(anyhow::Error::from)
            .and_then(|out| {
                if out.status.success() {
                    Ok(out.stdout)
                } else {
                    Err(anyhow!("Command failure: cmd={cmd_str}, args={args_str}, output={out:?}."))
                }
            });
            tx.send(result).unwrap_or_else(|error| {
                tracing::error!(
                    ?error,
                    pid,
                    cmd = cmd_str,
                    args = args_str,
                    "Failed to return send cmd result. Receiver dropped."
                );
            })
        }
    });
    match rx.recv_timeout(timeout) {
        Ok(ok @ Ok(_)) => ok,
        Ok(err @ Err(_)) => err,
        Err(_) => {
            if let Err(error) = kill(pid) {
                tracing::error!(
                    ?error,
                    pid,
                    cmd = cmd_str,
                    args = args_str,
                    "Failed to kill timed-out process."
                );
            }
            Err(anyhow!("Timed-out: cmd={cmd_str}, args={args_str}."))
        }
    }
}

fn kill(pid: u32) -> anyhow::Result<()> {
    use nix::{sys::signal::Signal::SIGKILL, unistd::Pid};

    // Catch wrap arounds when going from u32 to i32:
    let pid: i32 = pid.try_into()?;
    let pid: Pid = Pid::from_raw(pid);
    nix::sys::signal::killpg(pid, SIGKILL)?;
    Ok(())
}
