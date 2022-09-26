example sensors for use with [pista](https://github.com/xandkar/pista)
===============================================================================
[![Build Status](https://travis-ci.org/xandkar/pista-sensors.svg?branch=master)](https://travis-ci.org/xandkar/pista-sensors)
![Screenshot](screenshot.jpg)

Example configurations
----------------------
- [direct.sh](examples/direct.sh): basic setup, can be forked from `.xinitrc`
- [via-tmux.sh](examples/via-tmux.sh): init-like setup with tmux, can be
  conveniently (re)started at any time, independently of `.xinitrc`

Dependencies
------------

- [`fswatch`](https://github.com/emcrisostomo/fswatch) for `pista-sensor-backlight`
- OpenBSD `netcat` for `pista-sensor-mpd.sh`
- `jq` for multiple things
- `curl` for multiple things
- `hxpipe` for `pista-sensor-weather`
- `iwconfig` for `pista-sensor-wifi`

Notes
-----

### Volume

#### How to switch from intervaled polling to reactive updates?

    $ pactl subscribe | awk '/^Event .+ on sink .+$/'
    Event 'change' on sink #0
    Event 'change' on sink #0
    Event 'change' on sink #0
    ^CGot SIGINT, exiting.

seems to do the main trick, but still requires to trigger another call to check
the volume value. Is there any event on the system that carries this value?

#### TODO study how they do it:
- https://git.alsa-project.org/?p=alsa-utils.git;a=blob;f=alsactl/monitor.c;hb=HEAD
- https://github.com/illegalprime/alsa-monitor-node
