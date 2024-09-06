#!/usr/bin/env bash
set -e

SERVER="sgc"  # ssh config name or full address

cargo build --target=aarch64-unknown-linux-musl --release
rsync -vh --progress \
  ../../target/aarch64-unknown-linux-musl/release/brain \
  $SERVER:/tmp/

cmds="
sudo mv /tmp/brain /home/ha/brain
sudo chown ha:ha /home/ha/brain
sudo systemctl restart brain
"

ssh -t $SERVER "$cmds"
