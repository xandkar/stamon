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
    let path = CString::new(path).unwrap();
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
    }
    .unwrap();
    let used = total - free;
    let used_percentage = (used as f64 / total as f64) * 100.0;
    Ok(used_percentage.ceil() as u64)
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();
    let cli = Cli::parse();
    loop {
        match statfs(cli.path.as_str()) {
            Err(err) => log::error!("{:?}", err),
            Ok(percentage) => {
                println!("{}{}{}", &cli.prefix, percentage, &cli.postfix);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}
