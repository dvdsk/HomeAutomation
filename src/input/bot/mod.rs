use actix_web::http::StatusCode;
use actix_web::web::{Bytes, Data, HttpResponse};

use log::{error, info, warn};
use reqwest;
use serde_json;

use telegram_bot::types::callback_query::CallbackQuery;
use telegram_bot::types::message::Message;
use telegram_bot::types::message::MessageKind;
use telegram_bot::types::refs::{ChatId, MessageId, UserId};
use telegram_bot::types::update::Update;
use telegram_bot::types::update::UpdateKind;

use crate::input::web_api::server::State;
use crate::input::youtube_downloader::{self, FeedbackChannel};

pub mod youtube_dl;
use youtube_dl::TelegramFeedback;
mod set_alarm;

#[derive(Debug)]
pub enum Error {
    HttpClientError(reqwest::Error),
    CouldNotSetWebhook,
    InvalidServerResponse(reqwest::Response),
    UnhandledUpdateKind,
    UnknownCommand(String),
    SongDownloadError(youtube_downloader::Error),
    Unauthorized(ChatId),
    SetAlarmError(set_alarm::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::HttpClientError(error)
    }
}

impl From<youtube_downloader::Error> for Error {
    fn from(e: youtube_downloader::Error) -> Self {
        Error::SongDownloadError(e)
    }
}

impl From<set_alarm::Error> for Error {
    fn from(err: set_alarm::Error) -> Self {
        Error::SetAlarmError(err)
    }
}

fn to_string_and_ids(message: Message) -> Result<(String, ChatId, UserId, MessageId), Error> {
    let chat_id = message.chat.id();
    let user_id = message.from.id;
    let message_id = message.id;

    if let MessageKind::Text { data, entities: _ } = message.kind {
        return Ok((data, chat_id, user_id, message_id));
    } else {
        warn!("unhandled message kind");
        return Err(Error::UnhandledUpdateKind);
    }
}

async fn handle_command(
    text: String,
    chat_id: ChatId,
    message_id: MessageId,
    userid: UserId,
    state: &State,
) -> Result<(), Error> {
    if !state.valid_ids.contains(&userid) {
        return Err(Error::Unauthorized(chat_id));
    }

    let token = &state.bot_token;
    if text.contains("https://") || text.contains("http://") {
        let fb = TelegramFeedback {
            chat_id,
            message_id,
        };
        state
            .youtube_dl
            .add_song_to_queue(text, FeedbackChannel::Telegram(fb))
            .await?;
        return Ok(());
    }

    let mut split = text.split_whitespace();
    let command = split.next().unwrap();
    match command {
        "/test" => {
            send_text(chat_id, token, "hi").await?;
        }
        "/alarm" => {
            set_alarm::handle(chat_id, token, split, state).await?;
        }
        &_ => {
            warn!("no known command or alias: {:?}", &command);
            Err(Error::UnknownCommand(command.to_owned()))?;
        }
    }

    return Ok(());
}

async fn handle_callback(callback: CallbackQuery, state: &State) {
    if !state.valid_ids.contains(&callback.from.id) {
        warn!("Unauthorized callback, userid: {}", callback.from.id);
        return; //no need for error handling, just drop callback
    }

    if callback.data.is_none() {
        debug!("no data in callback");
        return; //not allowed, drop callback
    }
    let data = callback.data.as_ref().unwrap();
    let header = data.split_terminator(":").next();
    let res = match header {
        Some("ytdl") => youtube_dl::handle_callback(data, state).await,
        _ => {
            warn!(
                "unhandled callback, data: {}\nfull_callback: {:?}",
                data, callback
            );
            Ok(()) //user need not be warned as user is dev
        }
    };
    if let Err(e) = res {
        error!("ran into error handling callback: {:?}", e);
    }
}

const INT_ERR_TEXT: &str = "apologies, an internal error happend this has been reported and will be fixed as soon as possible";
const UNHANDLED: &str = "sorry I can not understand your input";
async fn handle_error(error: Error, chat_id: ChatId, token: &str) {
    let error_message = match error {
        Error::UnknownCommand(input) => format!("Unknown command: {}", input),
        Error::HttpClientError(err) => {
            error!("Internal error in http client: {}", err);
            String::from(INT_ERR_TEXT)
        }
        Error::InvalidServerResponse(resp) => {
            error!("Incorrect bot api response {:?}", resp);
            String::from(INT_ERR_TEXT)
        }
        Error::SongDownloadError(e) => {
            error!("song download error {:?}", e);
            String::from("Could not download song due to critical internal error")
        }
        Error::SetAlarmError(e) => {
            error!("could not set alarm {:?}", e);
            String::from("Could not set alarm")
        }
        Error::Unauthorized(chat_id) => format!(
            "You are not authorized to use this bot, if you \
			believe this is a mistake ask for your telegram chat id \
			{} to be added",
            chat_id
        ),

        Error::UnhandledUpdateKind => String::from(UNHANDLED),
        Error::CouldNotSetWebhook => unreachable!(),
    };
    if let Err(error) = send_text(chat_id, token, error_message).await {
        error!("Could not send text reply to user: {:?}", error);
    }
}

async fn handle(update: Update, state: State) {
    //TODO change to ref
    let token = &state.bot_token;
    match update.kind {
        UpdateKind::Message(messg) => {
            if let Ok((text, chat_id, user_id, message_id)) = to_string_and_ids(messg) {
                if let Err(error) = handle_command(text, chat_id, message_id, user_id, &state).await
                {
                    handle_error(error, chat_id, token).await;
                }
            }
        }
        UpdateKind::CallbackQuery(callback) => {
            //let chat_id = callback.message.chat.id();
            handle_callback(callback, &state).await;
        }
        _ => {
            warn!("unhandled update from telegram: {:?}", update);
        }
    }
}

pub async fn handle_webhook(state: Data<State>, raw_update: Bytes) -> HttpResponse {
    let update: Update = serde_json::from_slice(&raw_update.to_vec()).unwrap();
    let state_cpy = state.get_ref().clone();
    handle(update, state_cpy).await;

    HttpResponse::Ok().status(StatusCode::OK).body("{}")
}

pub async fn send_text<T: Into<String>>(
    chat_id: ChatId,
    token: &str,
    text: T,
) -> Result<(), Error> {
    //add as arg generic ToChatRef (should get from Update)
    //TODO create a SendMessage, serialise it (use member function serialize)
    //then use the HttpRequest fields, (url, method, and body) to send to telegram
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let form = reqwest::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
        .text("text", text.into());

    let client = reqwest::Client::new();
    let resp = client.post(&url).multipart(form).send().await?;
    //https://stackoverflow.com/questions/57540455/error-blockingclientinfuturecontext-when-trying-to-make-a-request-from-within

    if resp.status() != reqwest::StatusCode::OK {
        Err(Error::InvalidServerResponse(resp))
    } else {
        info!("send message");
        Ok(())
    }
}

//Ports currently supported for Webhooks: 443, 80, 88, 8443.
pub async fn set_webhook(domain: &str, token: &str, port: u16) -> Result<(), Error> {
    let url = format!("https://api.telegram.org/bot{}/setWebhook", token);
    let webhook_url = format!("{}:{}/{}", domain, port, token);

    let params = [("url", &webhook_url)];
    let client = reqwest::Client::new();
    let res = client.post(url.as_str()).form(&params).send().await?;

    if res.status() != reqwest::StatusCode::OK {
        Err(Error::CouldNotSetWebhook)
    } else {
        info!("set webhook to: {}", webhook_url);
        Ok(())
    }
}
