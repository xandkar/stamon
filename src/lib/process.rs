use std::{sync::mpsc, thread, time::Duration};

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
    cmd: &'static str,
    args: &'static [&'static str],
    timeout: Duration,
) -> Option<anyhow::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = exec(cmd, args);
        tx.send(result).unwrap_or_else(|error| {
            tracing::error!(
                ?error,
                ?cmd,
                ?args,
                "Timed-out receiver. Can't send cmd result."
            );
        })
    });
    rx.recv_timeout(timeout).ok()
}
