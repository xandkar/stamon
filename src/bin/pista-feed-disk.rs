use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(default_value = "/")]
    path: String,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "d ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    let path = cli.path.as_str();
    let mut stdout = std::io::stdout().lock();
    loop {
        match pista_feeds::disk::usage(path) {
            Err(err) => {
                tracing::error!("Failed to read disk usage info: {:?}", err)
            }
            Ok(None) => {
                tracing::error!("Failed to calculate disk usage percentage")
            }
            Ok(Some(percentage)) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(stdout, "{}{:3.0}%", &cli.prefix, percentage)
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}
