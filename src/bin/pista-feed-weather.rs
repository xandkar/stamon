use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};

use pista_feeds::feeds::weather::{
    self,
    observatories::{nws, owm},
};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, short, default_value_t = false)]
    debug: bool,

    #[clap(long, short, default_value_t = 1800)]
    interval: u64,

    // TODO Implement summary for any Observatory, like weather::Observatory::write_summary(file)
    #[clap(long)]
    nws_summary_file: Option<std::path::PathBuf>,

    // TODO Can NWS API accept coord instead of station ID?
    // TODO Can we lookup station ID by coordinates?
    // TODO Unify our CLI to accept just coordinates?
    #[clap(long)]
    nws_station_id: Option<String>,

    #[clap(long, default_value = "pista-feed-weather")]
    nws_app_name: String,

    #[clap(long, default_value = env!("CARGO_PKG_VERSION"))]
    nws_app_version: String,

    #[clap(long, default_value = "https://github.com/xandkar/pista-feeds")]
    nws_app_url: String,

    /// Give NWS a way to contact you to inform of API misuse (interval too
    /// short, etc), instead of just getting blocked. See "Authentication"
    /// section at: https://www.weather.gov/documentation/services-web-api
    #[clap(long, default_value = "user-has-not-provided-contact-info")]
    nws_admin_email: String,

    #[clap(long)]
    owm_coord: Option<owm::Coord>,

    #[clap(long)]
    owm_api_key: Option<String>,

    #[clap(long, short, num_args=1..)]
    observatories: Vec<ObservatoryName>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ObservatoryName {
    Nws,
    Owm,
}

impl std::str::FromStr for ObservatoryName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "nws" => Ok(Self::Nws),
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
                ObservatoryName::Nws => {
                    let station_id = self
                        .nws_station_id
                        .as_ref()
                        .ok_or_else(|| anyhow!("Missing NWS station id"))?
                        .to_string();
                    let user_agent = nws::UserAgent {
                        app_name: self.nws_app_name.to_string(),
                        app_version: self.nws_app_version.to_string(),
                        app_url: self.nws_app_url.to_string(),
                        admin_email: self.nws_admin_email.to_string(),
                    };
                    let settings = nws::Settings {
                        station_id,
                        user_agent,
                        summary_file: self.nws_summary_file.clone(),
                    };
                    let observatory = nws::Observatory::new(&settings)?;
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
                    let settings = owm::Settings { coord, api_key };
                    let observatory = owm::Observatory::new(&settings)?;
                    observatories.push(Box::new(observatory));
                }
            }
        }
        Ok(observatories)
    }
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();
    pista_feeds::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    weather::run(Duration::from_secs(cli.interval), cli.to_observatories()?)
}
