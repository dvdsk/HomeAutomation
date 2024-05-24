#!/usr/bin/env bash
set -e

BUILD_ARG=$1
SERVER="sgc"  # ssh config name or full adress
RELEASE=debug

if [[ BUILD_ARG == "--release" ]]; then
	RELEASE=release
fi

rustup override set nightly
cross build --target=aarch64-unknown-linux-gnu $BUILD_ARG
rsync -vh --progress \
  target/aarch64-unknown-linux-gnu/$RELEASE/sensor_central \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/sensor_central /home/ha/sensor_central
sudo chown ha:ha /home/ha/sensor_central
sudo systemctl restart sens.service
"

ssh -t sgc "$cmds"
