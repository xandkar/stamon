#! /bin/sh

trap '' PIPE

while :
do
    df ~ | awk 'NR == 2 {sub("%$", "", $5); printf "d %3d%%\n", $5}'
    sleep "$1"
done
