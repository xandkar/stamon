#[test]
fn current_observation() {
    let payload =
        std::fs::read_to_string("tests/weather-gov-current-observation.xml")
            .unwrap();
    let super::CurrentObservation {
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
