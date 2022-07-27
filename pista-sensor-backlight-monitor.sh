#! /bin/bash

set -e
set -o pipefail

current_value() {
    sudo ddcutil getvcp 10 \
    | grep -o ' current value = \+[0-9]\+' \
    | awk '{print $4}'
}

main() {
    trap '' PIPE

    local -r interval="${1:-60}" # 1st arg or default to 60.

    printf '[info] starting with interval: "%s"\n' "$interval" >&2
    while :
    do
        printf '☀ %3d%%\n' "$(current_value)"
        sleep "$interval"
    done
}

main "$@"
