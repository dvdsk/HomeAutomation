extern crate chrono;
extern crate smallvec;
extern crate bytes;
use byteorder::{LittleEndian, ByteOrder};

#[cfg(feature = "sensors_connected")]
mod local;
mod fields;
mod ble;

use std::thread;
use std::time::Duration;
use chrono::Utc;

use crate::credentials;
use crate::controller::Event;
use local::{TEMPERATURE, HUMIDITY, PRESSURE, STOP_ENCODE};

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
			let mut line = vec!(0u8; (STOP_ENCODE+8-1)/8);
			LittleEndian::write_u16(&mut line[0..2], credentials::NODE_ID);
			LittleEndian::write_u64(&mut line[2..10], credentials::DATASERVER_KEY);

			TEMPERATURE.encode::<f32>(temp, &mut line);
			HUMIDITY.encode::<f32>(hum, &mut line);
			PRESSURE.encode::<f32>(pressure, &mut line);

			dbg!(&line);
			
			//send data to dataserver
			let client = reqwest::Client::new();
			if let Err(e) = client.post("https://www.deviousd.duckdns.org:38972/post_data")
				.body(line)
				.send(){

				error!("could not send data to dataserver, error: {:?}", e);
			}

			//sleep until 5 seconds are completed
			thread::sleep(Duration::from_secs(5));
		}
	});
}
