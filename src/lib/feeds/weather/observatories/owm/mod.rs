mod data;

use anyhow::{anyhow, Result};

use crate::feeds::weather;

const UNITS: &str = "imperial"; // imperial | metric
const LANG: &str = "en"; // en | de

#[derive(Debug)]
pub struct Settings {
    pub coord: Coord,
    pub api_key: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Coord {
    lat: f64,
    lon: f64,
}

impl std::str::FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s
            .split(',')
            .map(|num| num.parse::<f64>())
            .collect::<Vec<Result<f64, std::num::ParseFloatError>>>()[..]
        {
            [Ok(lat), Ok(lon)] => Ok(Self { lat, lon }),
            // TODO Finer-grained errors?
            _ => Err(anyhow!("invalid format of coordinates. expected lat,lon, but got: {:?}", s)),
        }
    }
}

pub struct Observatory {
    module_path: String,
    url: String,
}

impl Observatory {
    pub fn new(
        Settings {
            coord: Coord { lat, lon },
            api_key,
        }: &Settings,
    ) -> Result<Self> {
        let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units={}&lang={}&appid={}",
            lat, lon, UNITS, LANG, api_key
        );
        tracing::info!("url: {:?}", &url);
        Ok(Self {
            module_path: module_path!().to_string(),
            url,
        })
    }
}

impl weather::Observatory for Observatory {
    fn module_path(&self) -> &str {
        &self.module_path
    }

    fn fetch(&self) -> Result<weather::Observation> {
        let client = reqwest::blocking::Client::new();
        let req = client.get(&self.url).build()?;
        let resp = client.execute(req)?;
        match resp.status() {
            reqwest::StatusCode::OK => {
                let payload = resp.text()?;
                let observation: data::CurrentWeather =
                    serde_json::from_str(&payload)?;
                let temp_f = {
                    assert_eq!("imperial", UNITS);
                    observation.main.temp
                };
                Ok(weather::Observation { temp_f })
            }
            s => Err(anyhow!("Error response: {:?} {:?}", s, resp)),
        }
    }
}
