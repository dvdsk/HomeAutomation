#!/usr/bin/env bash
set -e

BUILD_ARG=--release
NAME=desk-sensors

cargo build --target=aarch64-unknown-linux-musl $BUILD_ARG

for server in "atlantis"; do
	if [ $server == "atlantis" ]; then 
		sed -i 's/--bedroom large/--bedroom small/' $NAME.service
	else
		sed -i 's/--bedroom small/--bedroom large/' $NAME.service
	fi

	rsync -vh --progress \
	  ../../target/aarch64-unknown-linux-musl/release/$NAME \
	  $server:/tmp/
	rsync -vh --progress \
	  $NAME.service \
	  $server:/tmp/

	cmds="
	sudo mv /tmp/$NAME /usr/bin/
	sudo mv /tmp/$NAME.service /etc/systemd/system
	sudo systemctl daemon-reload
	sudo systemctl enable $NAME.service
	sudo systemctl restart $NAME.service
	"

	ssh -t $server "$cmds"
done
