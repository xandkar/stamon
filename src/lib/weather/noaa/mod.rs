use std::path::PathBuf;

use anyhow::{anyhow, Result};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Settings {
    pub station_id: String,
    pub user_agent: UserAgent,
    pub summary_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct UserAgent {
    // Data needed to construct user-agent header recommended by weather.gov:
    // ApplicationName/vX.Y (http://your.app.url/; contact.email@example.com)
    // https://stackoverflow.com/a/32641073/776984
    pub app_name: String,
    pub app_version: String,
    pub app_url: String,
    pub admin_email: String,
}

impl std::fmt::Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Self {
            app_name: name,
            app_version: vsn,
            app_url: url,
            admin_email: email,
        } = self;
        write!(f, "{}/{} ({}; {})", name, vsn, url, email)
    }
}

pub struct Observatory {
    module_path: String,
    url: String,
    user_agent: String,
    summary_file: Option<PathBuf>,
}

impl Observatory {
    pub fn new(
        Settings {
            user_agent,
            station_id,
            summary_file,
        }: &Settings,
    ) -> Result<Self> {
        let url = format!(
        "https://api.weather.gov/stations/{}/observations/latest?require_qc=false",
        station_id
    );
        let user_agent = user_agent.to_string();
        tracing::info!("url: {:?}", &url);
        tracing::info!("user_agent: {:?}", &user_agent);
        let summary_file = summary_file.clone();
        Ok(Self {
            module_path: module_path!().to_string(),
            url,
            user_agent,
            summary_file,
        })
    }
}

impl super::Observatory for Observatory {
    fn module_path(&self) -> &str {
        &self.module_path
    }

    fn fetch(&self) -> Result<super::Observation> {
        let client = reqwest::blocking::Client::new();
        let req = client
            .get(&self.url)
            .header(reqwest::header::ACCEPT, "application/vnd.noaa.obs+xml")
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .build()?;
        let resp = client.execute(req)?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                let payload = resp.text()?;
                let observation @ CurrentObservation { temp_f, .. } =
                    serde_xml_rs::from_str(&payload)?;
                match &self.summary_file {
                    None => (),
                    Some(path) => std::fs::write(
                        path,
                        observation.summary(chrono::offset::Local::now()),
                    )?,
                };
                Ok(super::Observation { temp_f })
            }
            s => Err(anyhow!("Error response: {:?} {:?}", s, resp)),
        }
    }
}

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
    fn summary(
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

    pub fn deserialize<'a, D>(
        deserializer: D,
    ) -> Result<chrono::DateTime<chrono::FixedOffset>, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::DateTime::parse_from_rfc2822(s.as_str())
            .map_err(serde::de::Error::custom)
    }
}
