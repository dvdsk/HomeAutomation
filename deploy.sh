#!/usr/bin/env bash
set -e

BUILD_ARG=$1
DIR=debug;
if [[ "$BUILD_ARG" == "--release" ]]; then 
	DIR=release
fi

SERVER="sgc"  # ssh config name or full address

cross build --target=aarch64-unknown-linux-gnu $BUILD_ARG
rsync -vh --progress \
  target/aarch64-unknown-linux-gnu/$DIR/brain \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/brain /home/ha/brain
sudo chown ha:ha /home/ha/brain
sudo systemctl restart ha
"

ssh -t sgc "$cmds"
