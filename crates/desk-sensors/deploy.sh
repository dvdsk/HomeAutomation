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

ssh -t $SERVER "$cmds"


SERVER="atlantis"  # ssh config name or full address
cargo build --target=armv7-unknown-linux-musleabihf $BUILD_ARG
rsync -vh --progress \
  ../../target/armv7-unknown-linux-musleabihf/$RELEASE/desk-sensors \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/desk-sensors /home/eva/desk-sensors
sudo chown eva:eva /home/eva/desk-sensors
sudo systemctl restart desk-sensors.service
"

ssh -t $SERVER "$cmds"
