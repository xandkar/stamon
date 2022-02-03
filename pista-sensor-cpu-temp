#! /bin/bash

set -euo pipefail

_data_files=

data_files_set() {
    local hwmon name label file

    for hwmon in /sys/class/hwmon/*
    do
        echo "hwmon: $hwmon" >&2
        # TODO "k10temp" if for my Ryzen 9, what about other CPUs?
        name=$(< "$hwmon/name")
        echo "  name: $name" >&2
	if [[ "$name" =~ k10temp ]]
	then
	    for label in "$hwmon"/temp*_label
	    do
                echo "    label: $label" >&2
                # TODO The following is for Ryzen 9. Does if differ for other CPUs?
                #
                # Tdie: real temperature of the die. XXX Seems to have disappeared. Void Linux, kernel 5.15.14_1.
                # Tctl: real + offset, for use with fans
                # Tccd1, Tccd2, ...: temperatures of individual dies
                #
                # info sources:
                #   - Tctl vs Tdie:
                #     - https://www.reddit.com/r/Amd/comments/9drxgs/hwinfo64_cputctl_or_cputdie/e5jp4f7/
                #     - https://www.guru3d.com/articles-pages/amd-ryzen-threadripper-1950x-review,8.html
                #     - https://community.amd.com/t5/blogs/amd-ryzen-community-update/ba-p/415608
                #   - Tccd1, Tccd2, ...:
                #     - https://boinc.berkeley.edu/dev/forum_thread.php?id=14150
		if [[ "$(< "$label")" =~ Tccd ]]
		then
                    file=${label/_label/_input}
                    echo "      file: $file" >&2
		    _data_files="$_data_files $file"
		fi
	    done
	fi
    done
}

data_files_read() {
    awk \
        '
        BEGIN {printf "t"}
        {printf " %3d°C", $1 / 1000}
        END {printf "\n"}
        ' \
        ${_data_files:?'ERROR: No temperature data files found!'}
}

main() {
    data_files_set
    while :
    do
        data_files_read
        sleep "$1"
    done
}

trap '' PIPE

main "$*"
