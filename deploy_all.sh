#!/usr/bin/env bash

set -e

for script in `find . -type f -name "deploy.sh"`; do
	dir=`dirname "$script"`
	cd "$dir"
	# the deploy scripts need to run in the right working directory
	. `basename "$script"`
	cd -
done
