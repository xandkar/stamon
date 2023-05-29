use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};

use pista_feeds::feeds::weather;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, short, default_value_t = 1800)]
    interval: u64,

    // TODO Implement summary for any Observatory, like weather::Observatory::write_summary(file)
    #[clap(long)]
    noaa_summary_file: Option<std::path::PathBuf>,

    // TODO Can NOAA API accept coord instead of station ID?
    // TODO Can we lookup station ID by coordinates?
    // TODO Unify our CLI to accept just coordinates?
    #[clap(long)]
    noaa_station_id: Option<String>,

    #[clap(long, default_value = "pista-feed-weather")]
    noaa_app_name: String,

    #[clap(long, default_value = env!("CARGO_PKG_VERSION"))]
    noaa_app_version: String,

    #[clap(
        long,
        default_value = "https://github.com/xandkar/pista-feeds-rs"
    )]
    noaa_app_url: String,

    #[clap(long, default_value = "user-has-not-provided-contact-info")]
    noaa_admin_email: String,

    #[clap(long)]
    owm_coord: Option<weather::openweathermap::Coord>,

    #[clap(long)]
    owm_api_key: Option<String>,

    #[clap(long, short, num_args=1..)]
    observatories: Vec<ObservatoryName>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ObservatoryName {
    Noaa,
    Owm,
}

impl std::str::FromStr for ObservatoryName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "noaa" => Ok(Self::Noaa),
            "owm" => Ok(Self::Owm),
            _ => Err(anyhow!("unknown observatory name: {:?}", s)),
        }
    }
}

impl Cli {
    fn to_observatories(&self) -> Result<Vec<Box<dyn weather::Observatory>>> {
        let mut observatories: Vec<Box<dyn weather::Observatory>> =
            Vec::new();
        for o in &self.observatories {
            match o {
                ObservatoryName::Noaa => {
                    let station_id = self
                        .noaa_station_id
                        .as_ref()
                        .ok_or_else(|| anyhow!("missing noaa station id"))?
                        .to_string();
                    let user_agent = weather::noaa::UserAgent {
                        app_name: self.noaa_app_name.to_string(),
                        app_version: self.noaa_app_version.to_string(),
                        app_url: self.noaa_app_url.to_string(),
                        admin_email: self.noaa_admin_email.to_string(),
                    };
                    let settings = weather::noaa::Settings {
                        station_id,
                        user_agent,
                        summary_file: self.noaa_summary_file.clone(),
                    };
                    let observatory =
                        weather::noaa::Observatory::new(&settings)?;
                    observatories.push(Box::new(observatory));
                }
                ObservatoryName::Owm => {
                    let coord = self.owm_coord.ok_or_else(|| {
                        anyhow!(
                            "missing lat,lon coordinates for OWM observatory"
                        )
                    })?;
                    let api_key: String = self
                        .owm_api_key
                        .as_ref()
                        .ok_or_else(|| {
                            anyhow!("missing API key for OWM observatory")
                        })?
                        .to_string();
                    let settings =
                        weather::openweathermap::Settings { coord, api_key };
                    let observatory =
                        weather::openweathermap::Observatory::new(&settings)?;
                    observatories.push(Box::new(observatory));
                }
            }
        }
        Ok(observatories)
    }
}

pub fn main() -> Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    weather::run(Duration::from_secs(cli.interval), cli.to_observatories()?)
}
