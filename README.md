![test status](https://github.com/xandkar/pista-feeds/actions/workflows/test.yml/badge.svg)
[![codecov](https://codecov.io/gh/xandkar/pista-feeds/branch/dominus/graph/badge.svg)](https://codecov.io/gh/xandkar/pista-feeds)
[![dependencies status](https://deps.rs/repo/github/xandkar/pista-feeds/status.svg)](https://deps.rs/repo/github/xandkar/pista-feeds)

pista-feeds
===============================================================================
![Screenshot](screenshot.png)

Status data for [pista](https://github.com/xandkar/pista).

Each of the executables periodically collects the necessary data and spits out
an aggregate.

Which I then redirect to a [FIFO](https://en.wikipedia.org/wiki/Named_pipe) to
be read by [pista](https://github.com/xandkar/pista) and inserted into my
[dwm](https://dwm.suckless.org/) status area.

Linux-only.

Some things _may_ work on other unices (like maybe time, weather (`http`), disk
(`statfs`) and keymap (`x11`)), but I'm only testing this on mine and a
[friend](https://github.com/asinovski)'s laptops (Void, Debian Stable and
Ubuntu LTS). Help with improving this is most welcome!

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
