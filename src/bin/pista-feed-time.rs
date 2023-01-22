use anyhow::Result;
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

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    tracing_subscriber::filter::LevelFilter::INFO.into(),
                )
                .from_env()?,
        )
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let cli = Cli::parse();
    tracing::info!("Cli: {:?}", &cli);
    let format = cli.format.as_str();
    let interval = std::time::Duration::from_secs_f64(cli.interval);
    loop {
        println!("{}", chrono::Local::now().format(format));
        std::thread::sleep(interval);
    }
}
