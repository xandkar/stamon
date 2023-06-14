use anyhow::Result;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

pub fn init(debug: bool) -> Result<()> {
    let level = if debug {
        LevelFilter::DEBUG.into()
    } else {
        LevelFilter::INFO.into()
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level)
                .from_env()?,
        )
        .with_writer(std::io::stderr)
        .with_ansi(debug)
        .with_file(debug)
        .with_line_number(debug)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
