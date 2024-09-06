#!/usr/bin/env bash

SERVER="sgc"  # ssh config name or full address
cmds="sudo systemctl stop brain"
ssh -t $SERVER "$cmds"

RUST_LOG=reqwest::connect=info,hyper_util=info,sled=info,trace RUST_BACKTRACE=1 cargo r -- \
  --data-server 192.168.1.43:1235 \
  --mpd-ip 127.0.0.1 \
  --port 34326 \
  --hue-bridge-ip 192.168.1.11

cmds="sudo systemctl start brain"
ssh -t $SERVER "$cmds"
