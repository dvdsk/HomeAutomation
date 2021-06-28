use sensor_value::SensorValue;
use telegram_bot::types::refs::UserId;

use actix_files as axtix_fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_rt::System;
use actix_web::web::{Bytes, Data};
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::{web, App, HttpServer, Responder};

use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc, Mutex, RwLock};
use std::thread;

use crate::controller::Event;
use crate::input;
use crate::input::bot;

pub mod database;
pub mod login;
pub mod login_redirect;

use super::{wakeup, commands, music};
pub use database::PasswordDatabase;
pub use login::{login_get_and_check, login_page, logout, make_random_cookie_key};
pub use login_redirect::CheckLogin;

pub struct Session {} //TODO deprecate

#[derive(Clone)]
pub struct State {
    pub controller_addr: crossbeam_channel::Sender<Event>,
    pub jobs: input::jobs::Jobs,
    pub wakeup: input::jobs::WakeUp,
    pub passw_db: PasswordDatabase,
    pub sessions: Arc<RwLock<HashMap<u16, Arc<Mutex<Session>>>>>,
    pub free_session_ids: Arc<AtomicUsize>,
    pub youtube_dl: input::YoutubeDownloader,
    pub bot_token: String,
    pub valid_ids: Vec<UserId>,
}

impl State {
    pub fn new(
        passw_db: PasswordDatabase,
        controller_tx: crossbeam_channel::Sender<Event>,
        jobs: input::jobs::Jobs,
        wakeup: input::jobs::WakeUp,
        youtube_dl: input::YoutubeDownloader,
        bot_token: String,
        valid_ids: Vec<i64>,
    ) -> Self {
        let free_session_ids = Arc::new(AtomicUsize::new(0));
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let valid_ids = valid_ids.into_iter().map(|id| UserId::from(id)).collect();

        State {
            controller_addr: controller_tx,
            jobs,
            wakeup,
            passw_db,
            youtube_dl,
            sessions,
            free_session_ids,
            bot_token,
            valid_ids,
        }
    }
}

pub async fn index(_req: HttpRequest) -> impl Responder {
    "Hello world!"
}

pub fn start_webserver(
    state: State,
    port: u16,
    domain: String,
    ha_key: String,
) -> actix_web::dev::Server {
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
                    web::scope("/login").service(
                        web::resource(r"/{path}")
                            .route(web::post().to(login_get_and_check))
                            .route(web::get().to(login_page)),
                    ),
                )
                .service(web::resource("/commands/lamps/toggle").to(commands::toggle))
                .service(web::resource("/commands/lamps/evening").to(commands::evening))
                .service(web::resource("/commands/lamps/night").to(commands::night))
                .service(web::resource("/commands/lamps/day").to(commands::normal))
                .service(web::resource("/commands/lamps/dimmest").to(commands::dimmest))
                .service(web::resource("/commands/lamps/dim").to(commands::dim))
                .service(web::resource("/commands/lightloop").to(commands::lightloop))
                .service(web::resource("/alarm/tomorrow").to(wakeup::tomorrow))
                .service(web::resource("/alarm/usually").to(wakeup::usually))
                .service(web::resource(&format!("/{}", &state.bot_token)).to(bot::handle_webhook))
                .service(
                    web::resource(&format!("/{}", ha_key)).route(web::post().to(handle_sensor)),
                )
                .service(
                    web::scope("/")
                        .wrap(CheckLogin {})
                        .service(web::resource("").to(index))
                        .service(web::resource("logout/").to(logout))
                        .service(web::resource("add_song").to(music::add_song_from_url))
                        //for all other urls we try to resolve to static files in the "web" dir
                        .service(axtix_fs::Files::new("", "./web/")),
                )
        })
        .bind(&format!("127.0.0.1:{}", port)) // SEC: disallow connections from the outside
        .unwrap()
        .shutdown_timeout(5) // shut down 5 seconds after getting the signal to shut down
        .run();

        let _ = tx.send(web_server.clone());
        sys.run()
    });

    let web_handle = rx.recv().unwrap();
    web_handle
}

pub fn handle_sensor(body: Bytes, state: Data<State>) -> HttpResponse {
    let res = bincode::deserialize::<SensorValue>(&body[..]);
    match res {
        Err(err) => error!("deserialize sensorval failed: {:?}", err),
        Ok(event) => state.controller_addr.send(Event::Sensor(event)).unwrap(),
    }
    HttpResponse::Ok().finish()
}
