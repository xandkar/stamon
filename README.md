data-feeds
===============================================================================
![Screenshot](screenshot.jpg)
Data feed processes for use with [pista](https://github.com/xandkar/pista).

TODO
----

### main
- [ ] time
- [x] keyboard layout
- [x] weather
- [x] mpd status
- [x] volume
- [x] backlight brightness
- [x] disk usage
- [x] memory usage
- [ ] bluetooth status
- [x] wifi status
- [x] ethernet status
- [ ] battery charge
- [ ] microphone (see `notes/microphone-use.txt`)

### improvements

#### all
- [x] configurable prefix in each feed
- [ ] configurable format strings https://github.com/vitiral/strfmt

#### weather
- [ ] fallback/alternative weather data sources
- [ ] forecast: https://weather-gov.github.io/api/general-faqs
  - [ ] daily
  - [ ] hourly

#### power/battery
- [ ] updates from D-Bus, rather than `upower --monitor-detail`
