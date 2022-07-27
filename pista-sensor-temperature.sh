#! /bin/bash

find_thermal_zone() {
    local -r _type="$1"
    awk \
        -v _type="$_type" \
        '
        $0 ~ ("^" _type "$") {
            split(FILENAME, f, "thermal_zone");
            split(f[2], f2, "/");
            print f2[1]}
        ' \
        /sys/class/thermal/thermal_zone*/type
}

case "$1" in
    '') thermal_zone="$(find_thermal_zone x86_pkg_temp)";;
     *) thermal_zone="$1"
esac

awk '{printf("%d°C\n", $1 / 1000)}' "/sys/class/thermal/thermal_zone${thermal_zone}/temp"
