#! /bin/bash

current_state() {
        local -r prefix="$1"

	pactl list sinks \
	| awk \
	    -v prefix="$prefix" \
	    -v default_sink="$(pactl info | awk '/^Default Sink:/ {print $3}')" \
	    '
	    /^Sink \#[0-9]+$/ {
		sub("^#", "", $2)
		sink = $2
		next
	    }

	    /\tState:/ {
		state[sink] = $2
		next
	    }

	    /\tName:/ {
		name[sink] = $2
		next
	    }

	    /\tMute:/ {
		mute[sink] = $2
		next
	    }

	    # Volume: front-left: 45732 /  70% / -9.38 dB,   front-right: 45732 /  70% / -9.38 dB
	    /\tVolume:/ {
		delete vol_parts
		delete left_parts
		delete right_parts
		sub("^\t+Volume: +", "")
		split($0, vol_parts, ", +")
		sub("^front-left: +", "", vol_parts[1])
		sub("^front-right: +", "", vol_parts[2])
		split(vol_parts[1], left_parts, " +/ +")
		split(vol_parts[2], right_parts, " +/ +")
		vol_left[sink] = left_parts[2]
		vol_right[sink] = right_parts[2]
		next
	    }

	    END {
		for (sink in state) {
		    if (name[sink] == default_sink) {
			show = "--"
			if (mute[sink] == "yes")
			    show = "X"
			else if (mute[sink] == "no")
			    show = vol_left[sink]
			else
			    printf("Unexpected value for mute field: %s\n" mute[sink]) > "/dev/stderr"

                        printf("%s %4s\n", prefix, show)
		    }
		}
	    }
	    '
}

trap '' PIPE

prefix="${1-v}"

# Initial reading
current_state "$prefix"

pactl subscribe | grep --line-buffered "^Event 'new' on sink" \
| while read -r _
do
    current_state "$prefix"
done
