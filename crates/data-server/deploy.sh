#!/usr/bin/env bash
set -e

BUILD_ARG=--release
SERVER="sgc"  # ssh config name or full address
NAME=data-server

cargo build --target=aarch64-unknown-linux-musl $BUILD_ARG
rsync -vh --progress \
  ../../target/aarch64-unknown-linux-musl/release/$NAME \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/$NAME /home/ha/$NAME
sudo chown ha:ha /home/ha/$NAME
sudo systemctl restart $NAME.service
"
ssh -t $SERVER "$cmds"
