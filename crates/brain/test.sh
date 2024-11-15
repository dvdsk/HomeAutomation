#!/usr/bin/env bash

RUST_LOG=reqwest::connect=info,hyper_util=info,sled=info,rumqttc::v5::state=info,zigbee_bridge::lights::cached_bridge::poll=trace,debug RUST_BACKTRACE=1 cargo r -- \
  --data-server 192.168.1.43:1235 \
  --mpd-ip 192.168.1.43 \
  --port 34326 \
  --hue-bridge-ip 192.168.1.11
