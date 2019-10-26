use chrono::{DateTime, Utc};
use rustls::{NoClientAuth, ServerConfig};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use actix_web::HttpRequest;

use std::sync::{Arc, Mutex, RwLock, atomic::AtomicUsize};
use std::collections::HashMap;
use std::path::Path;
use std::io::BufReader;
use std::fs::File;

use crate::input;
use crate::controller::Event;

pub mod database;
pub mod login_redirect;
pub mod login;

pub use database::{PasswordDatabase,UserDatabase};
pub use login::make_random_cookie_key;
pub use login_redirect::CheckLogin;


pub struct Session {//TODO deprecate 
	last_login: DateTime<Utc>,
  //add more temporary user specific data as needed
}

pub struct State {
	pub controller_addr: crossbeam_channel::Sender<Event>,
	pub alarms: input::alarms::Alarms,
	pub passw_db: PasswordDatabase,
	pub user_db: UserDatabase,
	pub sessions: Arc<RwLock<HashMap<u16, Arc<Mutex<Session>> >>> ,
	pub free_session_ids: Arc<AtomicUsize>,
	pub youtube_dl: input::YoutubeDownloader,
}

pub fn make_tls_config<P: AsRef<Path>>(signed_cert_path: P, private_key_path: P) -> rustls::ServerConfig{
	let mut tls_config = ServerConfig::new(NoClientAuth::new());
	let cert_file = &mut BufReader::new(File::open(signed_cert_path).unwrap());
	let key_file = &mut BufReader::new(File::open(private_key_path).unwrap());
	let cert_chain = certs(cert_file).unwrap();
	let mut key = pkcs8_private_keys(key_file).unwrap();

	tls_config
		.set_single_cert(cert_chain, key.pop().unwrap())
		.unwrap();
	tls_config
}

pub fn index(_req: HttpRequest) -> &'static str {
    "Hello world!"
}