#! /bin/bash

trap '' PIPE

interval="${1-5}"
prefix="${2-d}"

while :
do
    df ~ | awk -v prefix="$prefix" 'NR == 2 {sub("%$", "", $5); printf "%s%3d%%\n", prefix, $5}'
    sleep "$interval"
done
