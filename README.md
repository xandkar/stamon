![test status](https://github.com/xandkar/pista-feeds-rs/actions/workflows/test.yml/badge.svg)
[![codecov](https://codecov.io/gh/xandkar/pista-feeds-rs/branch/dominus/graph/badge.svg)](https://codecov.io/gh/xandkar/pista-feeds-rs)
[![dependencies status](https://deps.rs/repo/github/xandkar/pista-feeds-rs/status.svg)](https://deps.rs/repo/github/xandkar/pista-feeds-rs)

pista-feeds
===============================================================================
![Screenshot](screenshot.png)

Data feed processes that I use with [pista](https://github.com/xandkar/pista)
in my [dwm](https://dwm.suckless.org/) status area.


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
