use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use notify::{
    event::{DataChange, Event, EventKind::Modify, ModifyKind},
    RecursiveMode, Watcher,
};

#[derive(Parser)]
struct Cli {
    #[clap(long = "device", default_value = "intel_backlight")]
    device: String,

    #[clap(long = "prefix", default_value = "☀ ")]
    prefix: String,
}

#[derive(Debug)]
struct Paths {
    // TODO Figure-out lifetime stuff needed to convert these PathBufs to Paths:
    max: PathBuf,
    cur: PathBuf,
}

impl Paths {
    fn new(device: &str) -> Paths {
        let dev: PathBuf = ["/sys/class/backlight/", device].iter().collect();
        Paths {
            max: [&dev, &PathBuf::from("max_brightness")].iter().collect(),
            cur: [&dev, &PathBuf::from("brightness")].iter().collect(),
        }
    }
}

fn file_to_u64(path: &Path) -> Result<u64> {
    let data = std::fs::read_to_string(path)?;
    let data = data.trim();
    let int = data.parse()?;
    Ok(int)
}

fn print(prefix: &String, max: u64, cur: u64) {
    let max = max as f64;
    let cur = cur as f64;
    println!("{}{:3.0}%", prefix, cur / max * 100.0);
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
    let paths = Paths::new(&cli.device);
    let max = file_to_u64(&paths.max)?;
    let cur = file_to_u64(&paths.max)?;
    let mut pre = cur;
    tracing::info!("max: {}, cur: {}", max, cur);
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(sender)?;
    watcher.watch(Path::new(&paths.cur), RecursiveMode::Recursive)?;
    print(&cli.prefix, max, cur);
    for event in receiver {
        match event {
            Ok(event) => {
                if let Event {
                    kind: Modify(ModifyKind::Data(DataChange::Any)),
                    ..
                } = event
                {
                    match file_to_u64(&paths.cur) {
                        Err(e) => {
                            tracing::error!(
                                "Failure to read current value: {:?}",
                                e
                            );
                        }
                        Ok(cur) => {
                            if cur != pre {
                                print(&cli.prefix, max, cur);
                            }
                            pre = cur;
                        }
                    }
                }
            }
            Err(err) => tracing::error!("watch error: {:?}", err),
        }
    }
    Ok(())
}
