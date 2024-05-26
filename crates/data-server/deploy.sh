#!/usr/bin/env bash
set -e

BUILD_ARG=$1
SERVER="sgc"  # ssh config name or full adress
RELEASE=debug
NAME=datasplitter

if [[ BUILD_ARG == "--release" ]]; then
	RELEASE=release
fi

cross build --target=aarch64-unknown-linux-gnu $BUILD_ARG
rsync -vh --progress \
  target/aarch64-unknown-linux-gnu/$RELEASE/$NAME \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/$NAME /home/data/$NAME
sudo chown data:data /home/data/$NAME
sudo systemctl restart datasplitter.service
"

ssh -t sgc "$cmds"
