#! /bin/bash

set -euo pipefail

trap '' PIPE

loop() {
    local -r interval="$1"

    while :
    do
        free | awk '$1 == "Mem:" {
            total = $2
            available = $7
            used = total - available
            printf("m %3d%%\n", (used / total) * 100);
            exit 0
        }'
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
