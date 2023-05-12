data-feeds
===============================================================================
![Build status](https://github.com/xandkar/pista-feeds-rs/actions/workflows/rust.yml/badge.svg)
![Screenshot](screenshot.jpg)
Data feed processes for use with [pista](https://github.com/xandkar/pista).

TODO
----

### improvements
#### all
- [ ] configurable format strings https://github.com/vitiral/strfmt
- [ ] switch from `chrono` to `time` crate

#### weather
- [ ] fallback/alternative weather data sources
- [ ] forecast: https://weather-gov.github.io/api/general-faqs
  - [ ] daily
  - [ ] hourly

#### power/battery
- [ ] updates from D-Bus, rather than `upower --monitor-detail`
