pub mod backlight;
pub mod bluetooth;
pub mod disk;
pub mod math;
pub mod mem;
pub mod process;
pub mod pulseaudio;
pub mod upower;
pub mod weather;

// TODO Everything must implement State
//      - State.new/init
//      - State.update
//      - State.write
//      which can then be tested by giving a sequence of updates and examining
//      the data written to the buffer.
//
//      Perhaps notifications can be an output of State.update?

// pub trait State {
//     type Update;
//
//     fn update(update: Self::Update) -> anyhow::Result<()>; // TODO notifications?
//     fn write<W: std::io::Write>(buf: W) -> anyhow::Result<()>;
// }

pub fn tracing_init() -> anyhow::Result<()> {
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
    Ok(())
}
