use anyhow::Result;
use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(long, short, default_value_t = false)]
    debug: bool,

    #[clap(
        long = "format",
        short = 'f',
        default_value = "%a %b %d %H:%M:%S"
    )]
    format: String,

    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    let format = cli.format.as_str();
    let interval = std::time::Duration::from_secs_f64(cli.interval);
    let mut stdout = std::io::stdout().lock();
    loop {
        if let Err(e) = {
            use std::io::Write;
            writeln!(stdout, "{}", chrono::Local::now().format(format))
        } {
            tracing::error!("Failed to write to stdout: {:?}", e);
        }
        std::thread::sleep(interval);
    }
}
