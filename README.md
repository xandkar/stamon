stamon
===============================================================================

![Screenshot](screenshot.png)

![test status](https://github.com/xandkar/stamon/actions/workflows/test.yml/badge.svg)
[![codecov](https://codecov.io/gh/xandkar/stamon/branch/dominus/graph/badge.svg)](https://codecov.io/gh/xandkar/stamon)
[![dependencies status](https://deps.rs/repo/github/xandkar/stamon/status.svg)](https://deps.rs/repo/github/xandkar/stamon)

Status monitors for textual status bars such as
[pista](https://github.com/xandkar/pista) and
[barista](https://github.com/xandkar/barista).

Each of the executables periodically collects the necessary data and spits out
an aggregate line.

Such lines can then be read by something like
[barista](https://github.com/xandkar/barista) and inserted into a desired
status area.

Linux-only.

Some things _may_ work on other unices (like maybe time, weather (`http`), disk
(`statfs`) and keymap (`x11`)), but I'm only testing this on mine and a
[friend](https://github.com/asinovski)'s laptops (Void, Debian Stable and
Ubuntu LTS). Help with improving this is most welcome!

TODO
----

### improvements

#### all

- [ ] configurable format strings: <https://github.com/vitiral/strfmt>
- [ ] switch from `chrono` to `time` crate

#### weather

- [ ] fallback/alternative weather data sources
- [ ] forecast: <https://weather-gov.github.io/api/general-faqs>
  - [ ] daily
  - [ ] hourly

#### power/battery

- [ ] updates from D-Bus, rather than `upower --monitor-detail`
