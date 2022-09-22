use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, serde::Deserialize)]
struct CurrentObservation {
    dewpoint_string: String,
    location: String,
    //ob_url: String, // METAR file URL
    observation_time_rfc822: String,
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
        time_downloaded: chrono::DateTime<chrono::Local>,
    ) -> Result<String> {
        let time_observed = chrono::DateTime::parse_from_rfc2822(
            self.observation_time_rfc822.as_str(),
        )?
        .with_timezone(&chrono::Local)
        .to_rfc2822();
        let time_downloaded = time_downloaded.to_rfc2822();
        let summary = format!(
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
            time_observed,
            time_downloaded
        );
        Ok(summary)
    }
}

#[derive(Debug, Parser)]
struct Cli {
    station_id: String,

    #[clap(long = "interval", short = 'i', default_value_t = 1800)]
    interval: u64,

    #[clap(long = "summary-file", short = 's')]
    summary_file: Option<std::path::PathBuf>,

    #[clap(long = "app-name", default_value = "pista-sensor-weather")]
    app_name: String,

    #[clap(long = "app-version", default_value = "HEAD")]
    app_version: String,

    #[clap(
        long = "app-url",
        default_value = "https://github.com/xandkar/pista-sensors"
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
            let time_downloaded = chrono::offset::Local::now();
            let summary = observation.summary(time_downloaded)?;
            match summary_file {
                None => (),
                Some(path) => std::fs::write(path, summary)?,
            };
            Ok(observation.temp_f)
        }
        s => Err(anyhow!("Error response: {:?} {:?}", s, resp)),
    }
}

pub fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();
    let cli = Cli::parse();
    let user_agent = UserAgent::from_cli(&cli).to_string();
    log::info!("cli: {:?}", &cli);
    log::info!("user_agent: {:?}", &user_agent);
    let url = format!(
        "https://api.weather.gov/stations/{}/observations/latest?require_qc=false",
        &cli.station_id
    );
    log::info!("url: {:?}", &url);
    let interval_error_init = 15;
    let mut interval_error_curr = interval_error_init;
    let mut interval;
    loop {
        match download(&url, &user_agent, &cli.summary_file) {
            Err(e) => {
                log::error!("Failure in data download: {:?}", e);
                interval = interval_error_curr;
                interval_error_curr *= 2;
                log::warn!("Next retry in {} seconds.", interval);
            }
            Ok(temp_f) => {
                println!("{:3.0}°F", temp_f);
                interval = cli.interval;
                interval_error_curr = interval_error_init;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}
