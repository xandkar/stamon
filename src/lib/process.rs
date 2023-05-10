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
        Err(anyhow!("Failure in '{} {:?}'. out: {:?}", cmd, args, out))
    }
}
