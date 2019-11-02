#[macro_use]
extern crate log;
extern crate chrono;

use actix_web::{HttpServer,App, web};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_files as fs;

use sled;

use std::sync::atomic::{AtomicUsize};
use std::thread;

use std::sync::{Arc, RwLock};
use std::io::stdin;
use std::collections::HashMap;

mod controller;
use controller::Event;

mod input;
use input::web_api;
use input::web_api::server;
use server::{CheckLogin, PasswordDatabase, UserDatabase, index};
use server::login::{login_get_and_check, login_page, logout};

mod credentials;
mod errors;

#[cfg(feature = "live_server")]
const SOCKETADDR: &'static str = "0.0.0.0:8080";
#[cfg(not(feature = "live_server"))]
const SOCKETADDR: &'static str = "0.0.0.0:8070";

pub fn start_webserver(signed_cert: &str, private_key: &str,
	passw_db: PasswordDatabase,
	user_db: UserDatabase,
	controller_tx: crossbeam_channel::Sender<Event>,
	alarms: input::alarms::Alarms,
	youtube_dl: input::YoutubeDownloader,
	_mpd_status: input::MpdStatus,
	) -> actix_web::dev::Server {

	let tls_config = server::make_tls_config(signed_cert, private_key);
	let cookie_key = server::make_random_cookie_key();

  	let free_session_ids = Arc::new(AtomicUsize::new(0));
	let sessions = Arc::new(RwLock::new(HashMap::new()));
	let (tx, rx) = crossbeam_channel::unbounded();

	thread::spawn(move || {
		// Start data server actor in separate thread
		let sys = actix_rt::System::new("http-server");

		let web_server = HttpServer::new(move || {
			// data the webservers functions have access to
			let data = actix_web::web::Data::new(server::State {
				controller_addr: controller_tx.clone(),
				alarms: alarms.clone(),
				passw_db: passw_db.clone(),
				user_db: user_db.clone(),
				youtube_dl: youtube_dl.clone(),
				sessions: sessions.clone(),
				free_session_ids: free_session_ids.clone(),
		  });

		App::new()
			.register_data(data)
			.wrap(IdentityService::new(
				CookieIdentityPolicy::new(&cookie_key[..])
				.domain("deviousd.duckdns.org")
				.name("auth-cookie")
				.path("/")
				.secure(true), 
			))
			.service(
				web::scope("/login")
					.service(web::resource(r"/{path}")
						.route(web::post().to(login_get_and_check))
						.route(web::get().to(login_page))
			))
			.service(web::resource("/commands/lamps/toggle").to(web_api::toggle))
			.service(web::resource("/commands/lamps/evening").to(web_api::evening))
			.service(web::resource("/commands/lamps/night").to(web_api::night))
			.service(web::resource("/commands/lamps/day").to(web_api::normal))
			.service(web::resource("/commands/lamps/dimmest").to(web_api::dimmest))
			.service(web::resource("/commands/lamps/dim").to(web_api::dim))

			.service(web::resource("/commands/lightloop").to(web_api::lightloop))
			.service(web::scope("/")
				.wrap(CheckLogin)
				
				.service(web::resource("").to(index))
				.service(web::resource("logout/").to(logout))
				.service(web::resource("add_song").to(web_api::add_song_from_url))
				.service(web::resource("set_alarm").to(web_api::set_alarm_unix_timestamp))
				.service(web::resource("list_alarms").to(web_api::list_alarms))
				//for all other urls we try to resolve to static files in the "web" dir
				.service(fs::Files::new("", "./web/"))
			)
    })
    .bind_rustls(SOCKETADDR, tls_config).unwrap()
    //.bind("0.0.0.0:8080").unwrap() //without tcp use with debugging (note: https -> http, wss -> ws)
    .shutdown_timeout(5)    // shut down 5 seconds after getting the signal to shut down
    .start();

		let _ = tx.send(web_server);
		let _ = sys.run();
	});

	let web_handle = rx.recv().unwrap();
	web_handle
}

fn main() {

	let config = sled::ConfigBuilder::new()
			.path("database".to_owned())
			.flush_every_ms(None) //do not flush to disk unless explicitly asked
			.build();

	let db = sled::Db::start(config).unwrap();

	let passw_db = PasswordDatabase::from_db(&db).unwrap();
	let user_db = UserDatabase::from_db(&db).unwrap();

	let (controller_tx, controller_rx) = crossbeam_channel::unbounded();

	let _controller_thread = controller::start(controller_rx).unwrap();
	let (alarms, _waker_thread) = input::alarms::Alarms::setup(controller_tx.clone(), db.clone()).unwrap();
	let (youtube_dl, _downloader_thread) = input::YoutubeDownloader::init().unwrap();
	let (mpd_status, _mpd_watcher_thread, updater_tx) = input::MpdStatus::start_updating().unwrap();

	//start the webserver
	let web_handle = start_webserver("keys/cert.key", "keys/cert.cert"
	    ,passw_db.clone(), user_db.clone(), controller_tx.clone()
		,alarms.clone(), youtube_dl.clone(), mpd_status.clone());

	//TODO start the telegram server (for sending commands)

	//start monitoring local sensors
	#[cfg(feature = "sensors_connected")]
	input::sensors::start_monitoring(controller_tx.clone());
	#[cfg(feature = "sensors_connected")]
	input::buttons::start_monitoring(controller_tx.clone());

	println!("press: t to send test data, n: to add a new user, q to quit, a to add new dataset, u to add fields to a user");
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
	web_handle.stop(true);
	input::MpdStatus::stop_updating(updater_tx);
}
