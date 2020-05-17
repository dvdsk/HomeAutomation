use byteorder::{LittleEndian, ByteOrder};
use futures::future;

#[cfg(feature = "sensors_connected")]
mod local;
mod fields;
mod ble;

use std::thread;
use std::time::Duration;
use chrono::Utc;
use async_std::task;
use actix_rt;
use serde::{Serialize, Deserialize};

use crate::controller::Event;
use local::{TEMPERATURE, HUMIDITY, PRESSURE, STOP_ENCODE};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SensorValue {
	Temperature(f32),
	Humidity(f32),
	Pressure(f32),
}

use actix_rt::Runtime;
//const LOCAL_SENSING_ID: timeseries_interface::DatasetId = 0;
//TODO mechanisme for shutdown
#[cfg(feature = "sensors_connected")]
pub fn start_monitoring(tx: crossbeam_channel::Sender<Event>, 
	node_id: u16, dataserver_key: u64) {

	//init all local sensors
	let mut local_sensors = local::init();
	//subscribe and init remote sensors
	ble::init();

	thread::spawn(move || {//TODO figure out shutdown behaviour
		let mut rt = Runtime::new().unwrap();

		loop {
			//get all measurements
			let (hum, temp, pressure) = local::measure_and_record(&mut local_sensors);
			let now = Utc::now();

			//TODO
			//process data into events and send off as event
			tx.send(Event::Sensor(SensorValue::Temperature(temp))).unwrap();
			tx.send(Event::Sensor(SensorValue::Humidity(hum))).unwrap();	
			tx.send(Event::Sensor(SensorValue::Pressure(pressure))).unwrap();

			//encode all data
			let mut line = vec!(0u8; (STOP_ENCODE+8-1)/8);
			LittleEndian::write_u16(&mut line[0..2], node_id);
			LittleEndian::write_u64(&mut line[2..10], dataserver_key);

			TEMPERATURE.encode::<f32>(temp, &mut line);
			HUMIDITY.encode::<f32>(hum, &mut line);
			PRESSURE.encode::<f32>(pressure, &mut line);

			let client = reqwest::Client::new();
			let send = client.post("https://www.deviousd.duckdns.org:38972/post_data")
				.body(line)
				.send();
			let sleep = task::sleep(Duration::from_secs(5));
			//send data to dataserver and sleep up to 5 sec
			let (send_err, _) = rt.block_on(future::join(send, sleep));
			if let Err(e) = send_err {
				error!("could not send data to dataserver, error: {:?}", e);
			}
			
		}
	});
}
