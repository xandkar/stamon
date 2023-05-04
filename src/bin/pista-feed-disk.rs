use std::ffi::CString;
use std::mem::MaybeUninit;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(default_value = "/")]
    path: String,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "d ")]
    prefix: String,

    #[clap(long = "postfix", default_value = "%")]
    postfix: String,
}

fn statfs(path: &str) -> Result<u64> {
    let path = CString::new(path)?;
    let mut buf: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
    let (total, free) = unsafe {
        match libc::statfs(path.as_ptr() as *const i8, buf.assume_init_mut())
        {
            0 => {
                let libc::statfs {
                    f_blocks: total,
                    f_bfree: free,
                    ..
                } = buf.assume_init();
                Ok((total, free))
            }
            n => Err(anyhow!("libc::statfs failed with {}", n)),
        }
    }?;
    let used = total - free;
    let used_percentage = (used as f64 / total as f64) * 100.0;
    Ok(used_percentage.ceil() as u64)
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    let mut stdout = std::io::stdout().lock();
    loop {
        match statfs(cli.path.as_str()) {
            Err(err) => tracing::error!("{:?}", err),
            Ok(percentage) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(
                        stdout,
                        "{}{}{}",
                        &cli.prefix, percentage, &cli.postfix
                    )
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e)
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}
