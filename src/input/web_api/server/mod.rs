use rustls::{NoClientAuth, ServerConfig};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use telegram_bot::types::refs::UserId;

use actix_rt::System;
use actix_web::{HttpServer,App, web, Responder};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_files as axtix_fs;
use actix_web::HttpRequest;

use std::thread;
use std::sync::{Arc, Mutex, RwLock, atomic::AtomicUsize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::fs;

use crate::input;
use crate::input::bot;
use crate::controller::Event;

pub mod database;
pub mod login_redirect;
pub mod login;

pub use database::PasswordDatabase;
pub use login::{make_random_cookie_key, login_page, login_get_and_check, logout};
pub use login_redirect::CheckLogin;
use super::{alarms, commands, music, sensors};

pub struct Session {}//TODO deprecate

#[derive(Clone)]
pub struct State {
	pub controller_addr: crossbeam_channel::Sender<Event>,
	pub alarms: input::alarms::Alarms,
	pub passw_db: PasswordDatabase,
	pub sessions: Arc<RwLock<HashMap<u16, Arc<Mutex<Session>> >>> ,
	pub free_session_ids: Arc<AtomicUsize>,
	pub youtube_dl: input::YoutubeDownloader,
	pub bot_token: String,
	pub valid_ids: Vec<UserId>,
}

impl State {
	pub fn new(passw_db: PasswordDatabase,
		controller_tx: crossbeam_channel::Sender<Event>,
		alarms: input::alarms::Alarms,
		youtube_dl: input::YoutubeDownloader,
		bot_token: String,
		valid_ids: Vec<i64>) -> Self {

			let free_session_ids = Arc::new(AtomicUsize::new(0));
			let sessions = Arc::new(RwLock::new(HashMap::new()));
			let valid_ids = valid_ids
				.into_iter()
				.map(|id| UserId::from(id))
				.collect();

			State {
				controller_addr: controller_tx,
				alarms: alarms,
				passw_db: passw_db,
				youtube_dl: youtube_dl,
				sessions: sessions,
				free_session_ids: free_session_ids,
				bot_token,
				valid_ids,
			}
		}
}

#[derive(Debug)]
pub enum Error {
	NoKeyFound,
	NoCertFound,
}

fn get_key_and_cert(domain: &str, dir: &Path) -> Result<(PathBuf, PathBuf), Error> {
	let mut cert_path = Err(Error::NoCertFound);
	let mut key_path = Err(Error::NoKeyFound);
	let domain = domain.replace(".", "_");
	for path in fs::read_dir(dir).unwrap()
		.filter_map(Result::ok)
		.map(|entry| entry.path()) {

		if let Some(stem) = path.file_stem().map(|s| s.to_str()).flatten(){
			if !stem.contains(&domain){ continue }
			if let Some(ext) = path.extension().map(|s| s.to_str()).flatten(){
				match ext {
					"key" => key_path = Ok(path),
					"crt" => cert_path = Ok(path),
					_ => continue,
				}
			}
		}
	}

	Ok((key_path?, cert_path?))
}

pub fn make_tls_config(domain: &str, key_dir: &Path) -> Result<rustls::ServerConfig, Error> {

	//find cert and key
	let (key_path, cert_path) = get_key_and_cert(domain, key_dir)?;

	let mut tls_config = ServerConfig::new(NoClientAuth::new());
	let cert_file = &mut BufReader::new(fs::File::open(&cert_path)
		.expect(&format!("could not open certificate file: {:?}", cert_path)));
	let key_file = &mut BufReader::new(fs::File::open(&key_path)
		.expect(&format!("could not open key file: {:?}", key_path)));

	let cert_chain = certs(cert_file).unwrap();
	let mut key = pkcs8_private_keys(key_file).unwrap();

	tls_config
		.set_single_cert(cert_chain, key.pop().unwrap())
		.unwrap();
	Ok(tls_config)
}

pub async fn index(_req: HttpRequest) -> impl Responder {
    "Hello world!"
}

pub fn start_webserver(key_dir: &Path, 
	state: State, port: u16, domain: String, ha_key: String)
	 -> Result<actix_web::dev::Server,Error> {

	let tls_config = make_tls_config(&domain, key_dir)?;
	let cookie_key = make_random_cookie_key();
	let (tx, rx) = crossbeam_channel::unbounded();

	thread::spawn(move || {
		let sys = System::new("HttpServer");
		let web_server = HttpServer::new(move || {		
				// data the webservers functions have access to
			let data = actix_web::web::Data::new(state.clone());

			App::new()
				.app_data(data)
				.wrap(IdentityService::new(
					CookieIdentityPolicy::new(&cookie_key[..])
					.domain(&domain)
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
				.service(web::resource("/commands/lamps/toggle")
					.to(commands::toggle))
				.service(web::resource("/commands/lamps/evening")
					.to(commands::evening))
				.service(web::resource("/commands/lamps/night")
					.to(commands::night))
				.service(web::resource("/commands/lamps/day")
					.to(commands::normal))
				.service(web::resource("/commands/lamps/dimmest")
					.to(commands::dimmest))
				.service(web::resource("/commands/lamps/dim")
					.to(commands::dim))
				.service(web::resource("/commands/lightloop")
					.to(commands::lightloop))
				.service(web::resource(&format!("/{}", &state.bot_token))
					.to(bot::handle_webhook))
				.service(web::resource(&format!("/{}", ha_key))
						.route(web::post().to(sensors::handle))
				)

				.service(web::scope("/")
					.wrap(CheckLogin{})
					
					.service(web::resource("").to(index))
					.service(web::resource("logout/").to(logout))
					.service(web::resource("add_song").to(music::add_song_from_url))
					.service(web::resource("set_alarm").to(alarms::set_alarm_unix_timestamp))
					.service(web::resource("list_alarms").to(alarms::list_alarms))
					//for all other urls we try to resolve to static files in the "web" dir
					.service(axtix_fs::Files::new("", "./web/"))
				)
		})
		.bind_rustls(&format!("0.0.0.0:{}", port), tls_config).unwrap()
		//.bind("0.0.0.0:8080").unwrap() //without tcp use with debugging (note: https -> http, wss -> ws)
		.shutdown_timeout(5)    // shut down 5 seconds after getting the signal to shut down
		.run();

		let _ = tx.send(web_server.clone());
		sys.run()
		//let mut rt = Runtime::new().unwrap();
		//rt.block_on(web_server).unwrap();
	});

	let web_handle = rx.recv().unwrap();
	Ok(web_handle)
}