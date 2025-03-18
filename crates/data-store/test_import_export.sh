#!/usr/bin/env bash

# restore test file
cp data_updated/largebedroom/desk/bme280.csv{2,}

rm data_updated/largebedroom/desk/bme280.byteseries
rm data_updated/largebedroom/desk/bme280.byteseries_index
cargo r --release -- --data-dir data_updated import --only largebedroom/desk/bme280

rm data_updated/largebedroom/desk/bme280.csv
cargo r --release -- --data-dir data_updated export --only largebedroom/desk/bme280

diff -q data_updated/largebedroom/desk/bme280.csv{2,} 

echo "Tail of input/correct file"
tail -n 2 data_updated/largebedroom/desk/bme280.csv2
echo "Tail of output"
tail -n 2 data_updated/largebedroom/desk/bme280.csv
