#[derive(serde::Deserialize, Debug)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Weather {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Main {
    pub feels_like: f32,
    pub grnd_level: Option<f64>, // Preassure hPa
    pub humidity: f64,           // Percentage
    pub pressure: f64,           // hPa
    pub sea_level: Option<f64>,  // Preassure hPa
    pub temp: f32,               // XXX Why not f64?
    pub temp_max: f64,
    pub temp_min: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Wind {
    pub speed: f64,
    pub deg: f64,
    pub gust: Option<f64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Clouds {
    pub all: f64, // Percentage
}

#[derive(serde::Deserialize, Debug)]
pub struct Volume {
    #[serde(rename = "1h")]
    pub h1: Option<f64>,

    #[serde(rename = "3h")]
    pub h3: Option<f64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sys {
    #[serde(rename = "type")]
    pub typ: Option<u64>,
    pub country: String,
    pub id: Option<u64>,
    pub message: Option<f64>,
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(serde::Deserialize, Debug)]
pub struct CurrentWeather {
    pub id: u64,
    pub base: String,
    pub clouds: Clouds,
    pub cod: u64,
    pub coord: Coord,
    pub dt: i64,
    pub main: Main,
    pub name: String,
    pub rain: Option<Volume>,
    pub snow: Option<Volume>,
    pub sys: Sys,
    pub timezone: i64,
    pub visibility: u64,
    pub weather: Vec<Weather>,
    pub wind: Wind,
}
