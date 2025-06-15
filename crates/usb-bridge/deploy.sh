#!/usr/bin/env bash
set -e

BUILD_ARG=--release
NAME=usb-bridge

cargo build --target=aarch64-unknown-linux-musl $BUILD_ARG

for server in "atlantis"; do
	rsync -vh --progress \
	  ../../target/aarch64-unknown-linux-musl/release/$NAME \
	  $server:/tmp/
	rsync -vh --progress \
	  $NAME.service \
	  $server:/tmp/
	rsync -vh --progress \
	  $NAME.rules \
	  $server:/tmp/

	cmds="
	sudo mv /tmp/$NAME /usr/bin/
	sudo mv /tmp/$NAME.service /etc/systemd/system
	sudo mv /tmp/$NAME.rules /etc/udev/rules.d
	sudo udevadm control --reload-rules 
	sudo udevadm trigger
	sudo systemctl daemon-reload
	sudo systemctl enable $NAME.service
	sudo systemctl restart $NAME.service
	"

	ssh -t $server "$cmds"
done
