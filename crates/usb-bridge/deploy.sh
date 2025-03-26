#!/usr/bin/env bash
set -e

BUILD_ARG=--release
SERVER1="sgc"  # ssh config name or full address
SERVER2="eva@192.168.1.101"  # ssh config name or full address
NAME=usb-bridge

cargo build --target=aarch64-unknown-linux-musl $BUILD_ARG
cargo build --target=armv7-unknown-linux-musleabihf $BUILD_ARG
rsync -vh --progress \
  ../../target/aarch64-unknown-linux-musl/release/$NAME \
  $SERVER1:/tmp/
rsync -vh --progress \
  ../../target/armv7-unknown-linux-musleabihf/release/$NAME \
  $SERVER2:/tmp/

cmds="
sudo mv /tmp/$NAME /home/ha/$NAME
sudo chown ha:ha /home/ha/$NAME
sudo systemctl restart $NAME.service
"

ssh -t $SERVER1 "$cmds"

cmds="
mv /tmp/$NAME /home/eva/$NAME
sudo chown eva:eva /home/eva/$NAME
sudo systemctl restart $NAME.service
"

ssh -t $SERVER2 "$cmds"
