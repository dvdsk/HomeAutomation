#!/usr/bin/env bash
set -e

SERVER="sgc"  # ssh config name or full address
BUILD_ARG=--release
RELEASE=release

cargo build --target=aarch64-unknown-linux-musl $BUILD_ARG
rsync -vh --progress \
  ../../target/aarch64-unknown-linux-musl/$RELEASE/desk-sensors \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/desk-sensors /home/ha/desk-sensors
sudo chown ha:ha /home/ha/desk-sensors
sudo systemctl restart desk-sensors.service
"

ssh -t sgc "$cmds"
