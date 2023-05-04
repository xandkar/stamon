use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, serde::Deserialize)]
struct CurrentObservation {
    dewpoint_string: String,
    location: String,

    #[serde(with = "serde_rfc2822")]
    observation_time_rfc822: chrono::DateTime<chrono::FixedOffset>,

    //ob_url: String, // METAR file URL
    pressure_string: String,
    relative_humidity: String,
    station_id: String,
    //temp_c: f32,
    temp_f: f32,
    temperature_string: String,
    visibility_mi: f32,
    weather: String,
    wind_string: String,
}

impl CurrentObservation {
    pub fn summary(
        &self,
        download_time: chrono::DateTime<chrono::Local>,
    ) -> String {
        format!(
            "\n\
            {} ({})\n\
            \n\
            {}\n\
            {}\n\
            \n\
            humidity   : {}%\n\
            wind       : {}\n\
            pressure   : {}\n\
            dewpoint   : {}\n\
            visibility : {} miles\n\
            \n\
            observed   : {}\n\
            downloaded : {}\n\
            ",
            self.location,
            self.station_id,
            self.weather,
            self.temperature_string,
            self.relative_humidity,
            self.wind_string,
            self.pressure_string,
            self.dewpoint_string,
            self.visibility_mi,
            self.observation_time_rfc822
                .with_timezone(&chrono::Local)
                .to_rfc2822(),
            download_time.to_rfc2822()
        )
    }
}

// TODO Do we really need the custom module? Is there nothing in chrono already?
// https://serde.rs/custom-date-format.html
mod serde_rfc2822 {
    use serde::Deserialize; // String::deserialize method

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<chrono::DateTime<chrono::FixedOffset>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::DateTime::parse_from_rfc2822(s.as_str())
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Parser)]
struct Cli {
    station_id: String,

    #[clap(long = "interval", short = 'i', default_value_t = 1800)]
    interval: u64,

    #[clap(long = "summary-file", short = 's')]
    summary_file: Option<std::path::PathBuf>,

    #[clap(long = "app-name", default_value = "pista-feed-weather")]
    app_name: String,

    #[clap(long = "app-version", default_value = "HEAD")]
    app_version: String,

    #[clap(
        long = "app-url",
        default_value = "https://github.com/xandkar/pista-feeds"
    )]
    app_url: String,

    #[clap(
        long = "admin-email",
        default_value = "user-has-not-provided-contact-info"
    )]
    admin_email: String,
}

struct UserAgent {
    // Data needed to construct user-agent header recommended by weather.gov:
    // ApplicationName/vX.Y (http://your.app.url/; contact.email@example.com)
    // https://stackoverflow.com/a/32641073/776984
    app_name: String,
    app_version: String,
    app_url: String,
    admin_email: String,
}

impl UserAgent {
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            app_name: cli.app_name.to_string(),
            app_version: cli.app_version.to_string(),
            app_url: cli.app_url.to_string(),
            admin_email: cli.admin_email.to_string(),
        }
    }
}

impl std::fmt::Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}/{} ({}; {})",
            self.app_name, self.app_version, self.app_url, self.admin_email
        )
    }
}

fn download(
    url: &str,
    user_agent: &str,
    summary_file: &Option<std::path::PathBuf>,
) -> Result<f32> {
    let client = reqwest::blocking::Client::new();
    let req = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.noaa.obs+xml")
        .header(reqwest::header::USER_AGENT, user_agent)
        .build()?;
    let resp = client.execute(req)?;
    match resp.status() {
        reqwest::StatusCode::OK => {
            let payload = resp.text()?;
            let observation: CurrentObservation =
                serde_xml_rs::from_str(&payload)?;
            match summary_file {
                None => (),
                Some(path) => std::fs::write(
                    path,
                    observation.summary(chrono::offset::Local::now()),
                )?,
            };
            Ok(observation.temp_f)
        }
        s => Err(anyhow!("Error response: {:?} {:?}", s, resp)),
    }
}

pub fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    let user_agent = UserAgent::from_cli(&cli).to_string();
    tracing::info!("cli: {:?}", &cli);
    tracing::info!("user_agent: {:?}", &user_agent);
    let url = format!(
        "https://api.weather.gov/stations/{}/observations/latest?require_qc=false",
        &cli.station_id
    );
    tracing::info!("url: {:?}", &url);
    let interval_error_init = 15;
    let mut interval_error_curr = interval_error_init;
    let mut interval;
    let mut stdout = std::io::stdout().lock();
    loop {
        match download(&url, &user_agent, &cli.summary_file) {
            Err(e) => {
                tracing::error!("Failure in data download: {:?}", e);
                interval = interval_error_curr;
                interval_error_curr *= 2;
                tracing::warn!("Next retry in {} seconds.", interval);
            }
            Ok(temp_f) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(stdout, "{:3.0}°F", temp_f)
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e)
                }
                interval = cli.interval;
                interval_error_curr = interval_error_init;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

#[test]
fn current_observation() {
    let payload =
        std::fs::read_to_string("tests/weather-gov-current-observation.xml")
            .unwrap();
    let CurrentObservation {
        dewpoint_string,
        location,
        observation_time_rfc822,
        pressure_string,
        relative_humidity,
        station_id,
        temp_f,
        temperature_string,
        visibility_mi,
        weather,
        wind_string,
    } = serde_xml_rs::from_str(&payload).unwrap();
    assert_eq!("51.1 F (10.6 C)", dewpoint_string);
    assert_eq!("Manchester Airport, NH", location);
    assert_eq!(
        "Wed, 21 Sep 2022 14:53:00 +0000",
        observation_time_rfc822.to_rfc2822()
    );
    assert_eq!("1013.9 mb", pressure_string);
    assert_eq!("63", relative_humidity);
    assert_eq!("KMHT", station_id);
    assert_eq!(64.0, temp_f);
    assert_eq!("64 F (17.8 C)", temperature_string);
    assert_eq!(10.0, visibility_mi);
    assert_eq!("Mostly Cloudy", weather);
    assert_eq!("NW at 11.4 MPH (10 KT)", wind_string);
}
