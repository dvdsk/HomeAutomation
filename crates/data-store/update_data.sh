#!/usr/bin/env bash

set -e

if [ ! -d data ]; then 
	ssh sgc -q << ENDSSH
set -e
sudo systemctl stop data-store
cd /home/ha
sudo tar -czf data.tar.gz data
ENDSSH

	rsync sgc:/home/david/data.tar.gz .
	tar -xzf data.tar.gz
else 
	echo "data directory already exists, skipping fetch data from server"
fi

if [ ! -d data_updated ]; then 
	cargo r --release -- --data-dir data export
	cp -r data data_updated
	rm $(find data_updated | grep '.byteseries')
else
	echo "data_updated.tar.gz already exists skipping exporting"
fi

if [ ! -f data_updated.tar.gz ]; then 
	cargo r --release -- --data-dir data_updated import
	rm $(find data_updated | grep '.csv')
	tar -czf data_updated.tar.gz data_updated
else
	echo "data_updated.tar.gz already exists skipping importing"
fi

rsync $(pwd)/data_updated.tar.gz sgc:/home/david/

result=$(ssh sgc -q << ENDSSH
set -e
if [ -d /home/ha/data ]; then
	sudo mv /home/ha/data{,_backup}
fi
sudo tar -xzf data_updated.tar.gz -C /home/ha
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

ssh sgc -q << ENDSSH
sudo systemctl start data-store
ENDSSH

sleep 5
ssh sgc -q << ENDSSH
sudo systemctl status data-store
ENDSSH
