extern crate dataserver;
extern crate chrono;
extern crate smallvec;
extern crate bytes;

use dataserver::{httpserver::timeseries_interface, httpserver::DataRouterHandle};
use timeseries_interface::specifications::{MetaDataSpec, FieldSpec, FieldLength, FieldResolution};
use self::chrono::{Utc};

use crossbeam_channel;

mod buttons;
mod bme280;

use std::thread;
use std::time::{Duration};
use std::sync::{RwLock, Arc};

use crate::controller::{Event};

const LOCAL_SENSING_ID: timeseries_interface::DatasetId = 0;
//TODO mechanisme for shutdown
pub fn start_monitoring(tx: crossbeam_channel::Sender<Event>, data_router_handle: DataRouterHandle, dataset_handle: Arc<RwLock<timeseries_interface::Data>>) {

	buttons::start(tx.clone());

	//load the requird dataset
	let data = dataset_handle.read().unwrap();
	let dataset = data.sets.get(&LOCAL_SENSING_ID).expect("dataset for local sensors is missing");
	let fields = dataset.metadata.fields.clone();
	drop(data);

	//init all sensors
	let mut bme = bme280::init();

  thread::spawn(move || {//TODO figure out shutdown behaviour

		loop {
			//get all measurements
			let (humidity, temperature, pressure) = bme280::measure_and_record(&mut bme);

			//init new slow measurements
			//TODO
			let now = Utc::now();

			//process data into events and send off as event
			//TODO

			//encode all data
			//dbg!(fields);
			let mut line: Vec<u8> = vec!(0;64);

			fields[0].encode::<f32>(humidity, &mut line);
			fields[1].encode::<f32>(temperature, &mut line);
			fields[2].encode::<f32>(pressure, &mut line);
			//store data and send to active web_clients
			dataserver::httpserver::signal_newdata(&data_router_handle, LOCAL_SENSING_ID, line.clone(), now.timestamp() );
			let mut data = dataset_handle.write().unwrap();
			let set = data.sets.get_mut(&LOCAL_SENSING_ID).unwrap();
			set.timeseries.append(now, &line).unwrap();
			//dbg!(&line);
			drop(data);

			//sleep until 5 seconds are completed
			thread::sleep(Duration::from_secs(5));
		}
	});
}

fn make_local_sensing_spec() -> MetaDataSpec {
	let humidity_desk = FieldSpec::BitLength( FieldLength {
		name: String::from("humidity below desk"),
		min_value: 0.0f32,
		max_value: 100f32,
		numb_of_bits: 10u8, //bits (max 32 bit variables)
	});
	let temperature_desk = FieldSpec::Resolution( FieldResolution {
		name: String::from("temperature below desk"),
		min_value: -20f32,
		max_value: 60f32,
		resolution: 0.05f32, //bits (max 32 bit variables)
	});
	let pressure_desk = FieldSpec::Resolution( FieldResolution {
		name: String::from("pressure below desk"),
		min_value: 100_000f32-50_000f32,
		max_value: 100_000f32+50_000f32,
		resolution: 0.1f32,
	});

	MetaDataSpec {
		name: String::from("home automation sensors"),
		description: String::from("Data from homeautomation sensors"),
		fields: vec!(humidity_desk, temperature_desk, pressure_desk),
	}
}

pub fn check_local_sensing_dataset(data: &Arc<RwLock<timeseries_interface::Data>>) -> Result<(),()>{
	let mut data = data.write().unwrap();
	if let Some(dataset) = data.sets.get(&LOCAL_SENSING_ID){
		//check if name is right
		if dataset.metadata.name == "home automation sensors" {
			info!("loaded home automation data set");
			Ok(())
		} else {
			error!("No existing home automation dataset, cant create one without overwriting, dataset id already taken by dataset: {}",dataset.metadata.name);
			Err(())
		}
	} else {
		//TODO write specification that we can use
		let spec = make_local_sensing_spec();
		data.add_specific_set(spec).expect("could not create the dataset for the given spec");
		info!("created home automation dataset");
		Ok(())
	}
}
