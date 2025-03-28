#!/usr/bin/env bash

set -e

function download_and_export_to_csv {
	if [ ! -d data ]; then 
		ssh sgc -q <<- ENDSSH
	set -e
	sudo systemctl stop data-store
	cd /home/ha
	sudo tar -czf data.tar.gz data
	sudo chown david:david data.tar.gz
	cp data.tar.gz /tmp/data.tar.gz
	ENDSSH

		rsync sgc:/tmp/data.tar.gz .
		tar -xzf data.tar.gz
	else 
		echo "data directory already exists, skipping fetch data from server"
	fi

	if [ ! -d data_updated ]; then 
		cargo r --release -- --data-dir data export
		cp -r data data_updated
		rm $(find data_updated | grep '.byteseries')
	else
		echo "data_updated directory already exists skipping exporting"
	fi
}

function import_from_csv_and_upload {
	if [ ! -f data_updated.tar.gz ]; then 
		cargo r --release -- --data-dir data_updated import
		rm $(find data_updated | grep '.csv')
		tar -czf data_updated.tar.gz data_updated
	else
		echo "data_updated.tar.gz already exists skipping importing"
	fi

	rsync $(pwd)/data_updated.tar.gz sgc:/tmp/

	result=$(ssh sgc -q <<- ENDSSH
	set -e
	if [ -d /home/ha/data ]; then
		sudo mv /home/ha/data{,_backup}
	fi
	sudo tar -xzf /tmp/data_updated.tar.gz -C /home/ha
	sudo mv /home/ha/data{_updated,}
	sudo chown -R ha:ha /home/ha/data
	echo success
	ENDSSH
	)

	if [[ $result != *success ]]; then
		echo "Error, unpacking failed on server."
		echo "$result"
		exit
	fi

	ssh sgc -q <<- ENDSSH
	sudo systemctl start data-store
	ENDSSH

	sleep 5
	ssh sgc -q <<- ENDSSH
	sudo systemctl status data-store
	ENDSSH
}

echo -e "updating to a new encoding (range changed for example) is a two step process:\n\
 1. download encoded data and export with the current encoding\n\
 2. switch to the new data-store build\n\
 3. import the data from the exported csv's\n\
"

while true
do
	read -r -p 'at which step are you? (n to abort)' choice
    case "$choice" in
      n|N) break;;
      1) download_and_export_to_csv;;
      2) echo 'The script can not do that for you! do it then continue';;
      3) import_from_csv_and_upload && break;;
      *) echo 'Response not valid try: 1,2,3 or n';;
    esac
done
