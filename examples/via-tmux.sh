#! /bin/bash

set -e

SESSION='pista'
BASE_DIR="$HOME/.pista-in"

tmux_new_win() {
    # Have to pass the window id because incrementing a global is futile, since
    # tmux_new_win will be called from a new process each time and a global's
    # value will be reset.
    local -r win="$1"
    local -r cmd="$2"

    echo "[debug] win:\"$win\" cmd:\"$cmd\"" >&2
    tmux new-window -t "$SESSION"
    tmux send-keys  -t "$SESSION":"$win".0 "$cmd" ENTER
}

start_feed() {
    local -r win="$1"
    local -r exe="$2"
    shift 2
    local -r arg="$*"
    local -r fifo="$BASE_DIR/$exe"

    rm -f "$fifo"
    mkfifo "$fifo"
    tmux_new_win "$win" "$exe $arg > $fifo; notify-send -u critical 'pista-feed exited!' \"$exe\n\$?\""
    echo "$fifo"
}

_start() {
    local -r wifi_if=$(iwconfig | grep -v '^lo' | awk '/^[^\ ]/ {print $1}')

    tmux new-session -d -s "$SESSION"

    # Have to increment window ids manually, because an increment operation
    # would be executed local to each subprocess if I did something like:
    #     $(start_feed $((win++)) pista-feed-foo)
    tmux_new_win \
        0 \
        "pista \
        -l 3 \
        -i 0 \
        -f ' (' \
        -s ')  (' \
        -r ') ' \
        -x \
        $(start_feed  1  pista-feed-upower)            11 120  \
        $(start_feed  2  pista-feed-wifi $wifi_if 5)    8  10  \
        $(start_feed  3  pista-feed-bluetooth)          9  10  \
        $(start_feed  4  pista-feed-backlight)         10  -1  \
        $(start_feed  5  pista-feed-volume)             8  -1  \
        $(start_feed  6  pista-feed-mpd)               17   5  \
        $(start_feed  7  pista-feed-weather-gov \
                             -n \
                             -i $(( 30 * 60 )) KJFK)        8   $(( 30 * 60 )) \
        $(start_feed  8  pista-feed-time)              21   2 \
        ; notify-send -u critical 'pista exited!' \"$exe\n\$?\""
}

_stop() {
    tmux kill-session -t "$SESSION"
}

_restart() {
    _stop || true
    _start
}

_attach() {
    tmux attach -t "$SESSION"
}

main() {
    local -r cmd="$1"

    mkdir -p "$BASE_DIR"
    cd "$BASE_DIR"
    case "$cmd" in
        'start'   ) _start;;
        'stop'    ) _stop;;
        'restart' ) _restart;;
        'attach'  ) _attach;;
        *)
            echo "[error] Unknown command: \"$cmd\". Known: start|stop|restart|attach"
            exit 1;;
    esac
}

main "$*"
