#!/usr/bin/env bash

cargo build --profile=release-with-debug
echo '1' | sudo tee /proc/sys/kernel/perf_event_paranoid

samply record ../../target/release-with-debug/brain \
  --data-server 192.168.1.43:1235 \
  --mpd-ip 192.168.1.43 \
  --http-port 34326 \
  --mqtt-ip 192.168.1.43
