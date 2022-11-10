use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(
        long = "format",
        short = 'f',
        default_value = "%a %b %d %H:%M:%S"
    )]
    format: String,

    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f64,
}

fn main() {
    let cli = Cli::parse();
    eprintln!(">>> cli: {cli:?}");
    let format = cli.format.as_str();
    let interval = std::time::Duration::from_secs_f64(cli.interval);
    loop {
        println!("{}", chrono::Local::now().format(format));
        std::thread::sleep(interval);
    }
}
