#[macro_use]
extern crate log;

use structopt::StructOpt;
use sled;
use std::path::PathBuf;

mod controller;

mod input;
use input::web_api::server;
use input::web_api::server::{PasswordDatabase, State};
use input::bot;

mod errors;

#[derive(StructOpt)]
#[structopt(name = "homeautomation")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "8080")]
	port: u16,
	#[structopt(short = "e", long = "external-port")]
	external_port: Option<u16>,
    #[structopt(short = "t", long = "token")]
	token: String,
    #[structopt(short = "d", long = "domain")]
	domain: String,
    #[structopt(short = "a", long = "admin-password")]
	password: String,
    #[structopt(short = "k", long = "keys", default_value="keys")]
	key_dir: PathBuf,
    #[structopt(short = "i", long = "allowed-telegram-ids")]
	valid_ids: Vec<i64>,
    #[structopt(short = "h", long = "ha-key")]
	ha_key: String,
}

#[actix_rt::main]
async fn main() {
	let opt = Opt::from_args();
	errors::setup_logging(1).unwrap();

	let db = sled::Config::default() //651ms
			.path("database")
			.flush_every_ms(None) //do not flush to disk unless explicitly asked
			.cache_capacity(1024 * 1024 * 32) //32 mb cache 
			.open().unwrap();

	let mut passw_db = PasswordDatabase::from_db(&db).unwrap();
	passw_db.add_admin(&opt.password).unwrap();

	let (controller_tx, controller_rx) = crossbeam_channel::unbounded();

	let (alarms, _waker_thread) = input::alarms::Alarms::setup(
		controller_tx.clone(), 
		db.clone()).unwrap();
	let (youtube_dl, _downloader_thread) = input::YoutubeDownloader::init(
		opt.token.clone(), db.clone()).await.unwrap();
	let (mpd_status, _mpd_watcher_thread, _updater_tx) = 
		input::MpdStatus::start_updating()
		.unwrap();

	let _controller_thread = controller::start(
		controller_rx, 
		mpd_status.clone()).unwrap();

	let state = State::new(
		passw_db, 
		controller_tx.clone(),
		alarms, 
		youtube_dl, 
		opt.token.clone(),
		opt.valid_ids.clone());

	//start the webserver
	let _web_handle = server::start_webserver(
		&opt.key_dir, state, opt.port, 
		opt.domain.clone(), opt.ha_key.clone()).unwrap();
	

	bot::set_webhook(
		&opt.domain, &opt.token, 
		opt.external_port.unwrap_or_else(|| opt.port)).await.unwrap();

	loop {
		std::thread::park();
	}

	/*println!("press: q to quit");
	loop {
		let mut input = String::new();
		stdin().read_line(&mut input).unwrap();
		match input.as_str() {
			//"n\n" => helper::add_user(& passw_db),
			"q\n" => break,
			_ => println!("unhandled"),
		};
	}
	println!("shutting down");
	web_handle.stop(true).await;
	input::MpdStatus::stop_updating(updater_tx);*/
}
