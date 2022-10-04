#! /bin/sh

case "$1" in
    '') name='intel_backlight';;
     *) name="$1";;
esac

path="/sys/class/backlight/$name"

current_state() {
        awk '
            FILENAME ~ "/max_brightness$" {max = $1; next}
            FILENAME ~     "/brightness$" {cur = $1; next}
            END                           {printf("☀ %3d%%\n", (cur / max) * 100)}
        ' \
        "$path/max_brightness" \
        "$path/brightness"
}

trap '' PIPE

current_state

fswatch --event Updated "$path/brightness" \
| while read -r _
do
    current_state
done
