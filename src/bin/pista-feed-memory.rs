mod mem {
    use std::io::BufRead; // To access the lines() method.

    use anyhow::Result;

    pub struct Info {
        total: u64,
        available: u64,
    }

    impl Info {
        pub fn read() -> Result<Self> {
            let path = "/proc/meminfo";
            let file = std::fs::File::open(path)?;
            let reader = std::io::BufReader::new(file);
            let mut total = None;
            let mut avail = None;
            for line_result in reader.lines() {
                match (total, avail) {
                    (Some(_), Some(_)) => break,
                    (_, _) => {
                        let line = line_result?;
                        let mut fields = line.split_whitespace();
                        match (fields.next(), fields.next(), fields.next()) {
                            (
                                Some("MemTotal:"),
                                Some(num),
                                Some(_), // Ignoring units since we only report percentage.
                            ) => {
                                total = num.parse().ok();
                            }
                            (
                                Some("MemAvailable:"),
                                Some(num),
                                Some(_), // Ignoring units since we only report percentage.
                            ) => {
                                avail = num.parse().ok();
                            }
                            (_, _, _) => (),
                        }
                    }
                }
            }
            Ok(Self {
                total: total.unwrap_or(0),
                available: avail.unwrap_or(0),
            })
        }

        fn used(&self) -> u64 {
            self.total - self.available
        }

        pub fn used_pct(&self) -> f64 {
            (self.used() as f64 / self.total as f64) * 100.0
        }
    }
}

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "m ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("Cli: {:?}", &cli);
    let mut stdout = std::io::stdout().lock();
    loop {
        match mem::Info::read() {
            Ok(m) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(stdout, "{}{:3.0}%", &cli.prefix, m.used_pct())
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e)
                }
            }
            Err(e) => {
                tracing::error!("Failure to read /proc/meminfo: {:?}", e)
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}