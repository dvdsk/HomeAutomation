#!/usr/bin/env bash
set -e

BUILD_ARG=$1
SERVER="sgc"  # ssh config name or full address
RELEASE=debug
NAME=data-server

if [[ BUILD_ARG == "--release" ]]; then
	RELEASE=release
fi

cross build --target=aarch64-unknown-linux-gnu $BUILD_ARG
rsync -vh --progress \
  target/aarch64-unknown-linux-gnu/$RELEASE/$NAME \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/$NAME /home/ha/$NAME
sudo chown ha:ha /home/ha/$NAME
sudo systemctl restart data-server.service
"

ssh -t sgc "$cmds"
