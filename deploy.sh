#!/usr/bin/env bash
set -e

BUILD_ARG=$1
SERVER="sgc"  # ssh config name or full adress

cross build --target=aarch64-unknown-linux-gnu $BUILD_ARG --features "sensors_connected live_server"
rsync -vh --progress \
  target/aarch64-unknown-linux-gnu/debug/HomeAutomation \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/HomeAutomation /home/ha/homeAutomation
sudo chown ha:ha /home/ha/homeAutomation
sudo systemctl restart ha.service
"

ssh -t sgc "$cmds"
