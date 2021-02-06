example sensors for use with [pista](https://github.com/xandkar/pista)
===============================================================================

![Screenshot](screenshot.jpg)

Example configurations
----------------------
- [example](example): basic setup, can be forked from `.xinitrc`
- [example-via-tmux](example-via-tmux): init-like setup with tmux, can be
  conveniently (re)started at any time, independently of `.xinitrc`

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
