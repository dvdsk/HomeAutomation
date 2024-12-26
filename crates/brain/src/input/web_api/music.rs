use actix_web::http::StatusCode;
use actix_web::web::{Data, Form};

use serde::Deserialize;

use actix_web::HttpResponse;

use super::*;
use crate::input::youtube_downloader::FeedbackChannel;

#[derive(Deserialize, Debug)]
pub struct SongUrl {
    url: String,
}

pub async fn add_song_from_url(params: Form<SongUrl>, state: Data<State>) -> HttpResponse {
    let url = params.into_inner().url;

    if state
        .youtube_dl
        .add_song_to_queue(url, FeedbackChannel::None)
        .await
        .is_ok()
    {

        HttpResponse::Ok().finish()
    } else {
        make_error(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
