#!/usr/bin/env bash

RUST_LOG=brain=trace,zigbee_bridge=info,info RUST_BACKTRACE=1 cargo r -- \
  --data-server 192.168.1.43:1235 \
  --mpd-ip 192.168.1.43 \
  --http-port 34326 \
  --mqtt-ip 192.168.1.43
