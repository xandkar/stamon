#! /bin/sh

trap '' PIPE

while :
do
    awk '{printf("L:%.1f\n", $1)}' /proc/loadavg
    sleep "$1"
done
