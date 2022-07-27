#! /bin/bash

set -euo pipefail

trap '' PIPE

loop() {
    local -r interval="$1"

    while :
    do
        free | awk '$1 == "Mem:" {printf("m %3d%%\n", ($3 / $2) * 100); exit 0}'
        sleep "$interval"
    done
}

args="$*"
if [[ "$args" =~ ^[0-9]+$ && "$args" -gt 0 ]]; then
    loop "$args"
else
    printf 'Error: expected a single positive integer argument, but received "%s"\n' "$args" >&2
    exit 1
fi
