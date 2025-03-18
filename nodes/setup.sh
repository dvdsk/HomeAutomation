#!/usr/bin/env bash
set -e

# for newer or different prorammers edit idVendor and idProduct (use lsusb to find these)
UDEV_FILE=/etc/udev/rules.d/70-programmer.rules
UDEV_RULE1='# ST-LINK/V2'
UDEV_RULE2='ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"'
UDEV_RULE3='# ST-LINK/V2-1'
UDEV_RULE4='ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"'
UDEV_RULE5='# CMSIS_DAP compatible adapter'
UDEV_RULE6='ATTRS{product}=="*CMSIS-DAP*", MODE="660", GROUP="plugdev", TAG+="uaccess"'
UDEV_RULE5='# CMSIS_DAP compatible misspelled adapter'
UDEV_RULE6='ATTRS{product}=="*CMSIS_DAP*", MODE="660", GROUP="plugdev", TAG+="uaccess"'


function install_tools {
	echo "installing tools"
	rustup override set nightly
	rustup target add thumbv7em-none-eabihf

	# # workaround for issue: https://github.com/knurling-rs/probe-run/issues/289
	# sudo apt remove libusb-dev libusb-0.1-4

	# sudo apt install -y libusb-1.0-0-dev libudev-dev
	curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
	cargo install flip-link # linker with stack overflow protection
	# rustup component add llvm-tools-preview
}

# add udev rules if they do not yet exist
function fix_udev_rules {
	sudo groupadd dialout || true
	sudo usermod -a -G dialout $USER

	if [ ! -f "${UDEV_FILE}" ]; then
		echo "${UDEV_RULE1}" | sudo tee -a $UDEV_FILE > /dev/null
		echo "${UDEV_RULE2}" | sudo tee -a $UDEV_FILE > /dev/null
		echo "${UDEV_RULE3}" | sudo tee -a $UDEV_FILE > /dev/null
		echo "${UDEV_RULE4}" | sudo tee -a $UDEV_FILE > /dev/null
		echo "${UDEV_RULE5}" | sudo tee -a $UDEV_FILE > /dev/null
		echo "${UDEV_RULE6}" | sudo tee -a $UDEV_FILE > /dev/null
		sudo udevadm control --reload-rules
		sudo udevadm trigger
		echo "created udev rules: $UDEV_FILE"
	fi
}

install_tools
echo $(fix_udev_rules) # without the echo $() this does not work.. bash magic...

printf "\x1b[31mdo please manually check lsub and udev/rules-d for probe rule\x1b[0m\n";
printf "\x1b[31mdo NOT FORGET to DISCONNECT and RECONNECT the programmer\x1b[0m\n";
printf "\x1b[31mdo NOT FORGET to RE-LOGIN the shell to reload groups\x1b[0m\n";
