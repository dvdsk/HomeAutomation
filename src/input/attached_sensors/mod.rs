extern crate dataserver;
extern crate chrono;
extern crate smallvec;
extern crate bytes;

use bytes::Bytes;
use dataserver::{httpserver, httpserver::timeseries_interface, httpserver::DataRouterHandle};
use self::chrono::{Utc};

mod buttons;
mod bme280;

use std::thread;
use std::time::{Duration};
use std::sync::{RwLock, Arc};

use std::sync::mpsc::Sender;
use crate::controller::{Command, Event};

//TODO mechanisme for shutdown
pub fn start_monitoring(tx: Sender<Event>, data_router_handle: DataRouterHandle, dataset_handle: Arc<RwLock<timeseries_interface::Data>>) {
	const LOCAL_SENSING_ID: timeseries_interface::DatasetId = 0;

	buttons::start(tx.clone());

	//load the requird dataset
	let mut data = dataset_handle.write().unwrap();
	let dataset = data.sets.get(&LOCAL_SENSING_ID).expect("dataset for local sensors is missing");
	let fields = dataset.metadata.fields.clone();
	drop(data);

	//init all sensors
	let bme = bme280::init();


	let mut line: Vec<u8> = Vec::with_capacity(64);
  thread::spawn(move || {//TODO figure out shutdown behaviour


		//get all measurements
		let (humidity, temperature, pressure) = bme280::measure_and_record(bme);

		//init new slow measurements
		//TODO
		let now = Utc::now();

		//process data into events and send off as event
		//TODO

		//encode all data
		fields[0].encode::<f32>(humidity, &mut line);
		fields[1].encode::<f32>(temperature, &mut line);
		fields[2].encode::<f32>(pressure, &mut line);

		//store data and send to active web_clients
		dataserver::httpserver::signal_newdata(data_router_handle, LOCAL_SENSING_ID, line.clone(), now.timestamp() );
		let mut data = dataset_handle.write().unwrap();
		let set = data.sets.get_mut(&LOCAL_SENSING_ID).unwrap();
		set.timeseries.append(now, &line).unwrap();
		drop(data);

		//sleep until 5 seconds are completed
		thread::sleep(Duration::from_secs(5));
	});
}
