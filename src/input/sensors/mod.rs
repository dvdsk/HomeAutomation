extern crate chrono;
extern crate smallvec;
extern crate bytes;
use byteorder::{LittleEndian, WriteBytesExt};

#[cfg(feature = "sensors_connected")]
mod local;
mod fields;
mod ble;

use std::thread;
use std::time::Duration;
use chrono::Utc;

use crate::credentials;
use crate::controller::Event;
use local::{TEMPERATURE, HUMIDITY, PRESSURE};

pub enum SensorValue {
	Temperature(f32),
	Humidity(f32),
	Pressure(f32),
}

//const LOCAL_SENSING_ID: timeseries_interface::DatasetId = 0;
//TODO mechanisme for shutdown
#[cfg(feature = "sensors_connected")]
pub fn start_monitoring(tx: crossbeam_channel::Sender<Event>) {

	//init all local sensors
	let mut local_sensors = local::init();
	//subscribe and init remote sensors
	ble::init();

	thread::spawn(move || {//TODO figure out shutdown behaviour
		loop {
			//get all measurements
			let (hum, temp, pressure) = local::measure_and_record(&mut local_sensors);
			let now = Utc::now();

			//TODO
			//process data into events and send off as event
			tx.send(Event::Sensor(SensorValue::Temperature(temp)));
			tx.send(Event::Sensor(SensorValue::Humidity(hum)));
			tx.send(Event::Sensor(SensorValue::Pressure(pressure)));			

			//encode all data
			let mut line: Vec<u8> = vec!(0;64);
			
			line.write_u16::<LittleEndian>(credentials::NODE_ID).unwrap();
			line.write_u64::<LittleEndian>(credentials::DATASERVER_KEY).unwrap();

			HUMIDITY.encode::<f32>(hum, &mut line);
			TEMPERATURE.encode::<f32>(temp, &mut line);
			PRESSURE.encode::<f32>(pressure, &mut line);
			
			//send data to dataserver
			let client = reqwest::Client::new();
			client.post("http://www.deviousd.duckdns.org:8080/post_data")
				.body(line)
				.send().unwrap();

			//sleep until 5 seconds are completed
			thread::sleep(Duration::from_secs(5));
		}
	});
}
