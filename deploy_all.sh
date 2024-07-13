#!/usr/bin/env bash

set -e

for script in `find . -type f -name "deploy.sh"`; do
	$script
done
