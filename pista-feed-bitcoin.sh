#! /bin/sh

trap '' PIPE

while :
do
    printf 'btc %10.2f\n' $(curl https://www.bitstamp.net/api/v2/ticker/btcusd/ | jq '.["last"]' -r)
    sleep "$1"
done
