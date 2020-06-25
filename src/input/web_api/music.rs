use actix_web::web::{Form, Data};
use actix_web::http::StatusCode;

use serde::Deserialize;

use actix_web::HttpResponse;

use crate::input::youtube_downloader::FeedbackChannel;
use super::*;


#[derive(Deserialize, Debug)]
pub struct SongUrl {
	url: String,
}

pub async fn add_song_from_url(params: Form<SongUrl>, state: Data<State>) -> HttpResponse {
	let url = params.into_inner().url;
	dbg!();

	if state.youtube_dl.add_song_to_queue(url, FeedbackChannel::None)
		.await.is_ok(){
		dbg!();
		
		HttpResponse::Ok().finish()
	} else {
		make_error(StatusCode::INTERNAL_SERVER_ERROR)
	}
}
