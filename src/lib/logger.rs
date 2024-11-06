use anyhow::Result;
use tracing_subscriber::{filter::Directive, EnvFilter};

pub fn init(level: tracing::Level) -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Directive::from(level))
                .from_env()?,
        )
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        // FIXME fmt::time::LocalTime::rfc_3339 prints "<unknown time>" sometimes.
        //       The feature was disabled in time crate due to safety
        //       impossibility under multiple threads. It maybe possible that
        //       tracing-subscriber will switch to chrono instead:
        //       https://github.com/tokio-rs/tracing/issues/2080
        //.with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
