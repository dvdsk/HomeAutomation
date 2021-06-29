#!/usr/bin/env bash
set -e

BUILD_ARG=$1
SERVER="sgc"  # ssh config name or full adress

cross build --target=armv7-unknown-linux-gnueabihf $BUILD_ARG --features "sensors_connected live_server"
rsync -vh --progress \
  target/armv7-unknown-linux-gnueabihf/debug/HomeAutomation \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/HomeAutomation /home/ha/homeAutomation
sudo chown ha:ha /home/ha/homeAutomation
sudo systemctl restart ha.service
"

ssh -t sgc "$cmds"
