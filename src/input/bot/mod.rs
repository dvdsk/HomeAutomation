use actix_web::web::{HttpResponse, Data, Bytes};
use actix_web::http::{StatusCode};

use reqwest;
use log::{warn, info ,error};
use serde_json;

use telegram_bot::types::update::Update;
use telegram_bot::types::update::UpdateKind;
use telegram_bot::types::message::MessageKind;
use telegram_bot::types::refs::{ChatId, MessageId, UserId};

use crate::input::youtube_downloader::{self,FeedbackChannel};
use crate::input::web_api::server::{State};

pub mod youtube_dl;
use youtube_dl::TelegramFeedback;

#[derive(Debug)]
pub enum Error{
	HttpClientError(reqwest::Error),
	CouldNotSetWebhook,
	InvalidServerResponse(reqwest::Response),
	UnhandledUpdateKind,
    UnhandledMessageKind,
	UnknownCommand(String),
	SongDownloadError(youtube_downloader::Error),
	Unauthorized(ChatId),
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

fn to_string_and_ids(update: Update) 
-> Result<(String, ChatId, UserId, MessageId),Error>{
	
	if let UpdateKind::Message(message) = update.kind {
		let chat_id = message.chat.id();
		let user_id = message.from.id;
		let message_id = message.id;
		
		if let MessageKind::Text{data, entities:_} = message.kind {
			return Ok((data, chat_id, user_id, message_id));
		} else {
			warn!("unhandled message kind");
			return Err(Error::UnhandledUpdateKind);
		}
	} else {
		warn!("unhandled update from telegram: {:?}", update);
		return Err(Error::UnhandledMessageKind);
	}
}

async fn handle_command(text: String, chat_id: ChatId, message_id: MessageId,
	userid: UserId, state: &State) -> Result<(), Error>{
	
	if !state.valid_ids.contains(&userid){
		return Err(Error::Unauthorized(chat_id));
	}

	let token = &state.bot_token;
	if text.contains("https://") || text.contains("http://") {
		let fb = TelegramFeedback{chat_id, message_id};
		state.youtube_dl.add_song_to_queue(text, 
			FeedbackChannel::Telegram(fb)).await?;
		return Ok(());
	}

    //let split = text.find(char::is_whitespace);
    let command = text;
    //let args = command.split_off(split.unwrap_or(command.len()));
    match command.as_str() {
        "/test" => {
            send_text(chat_id, token, "hi").await?; 
            return Ok(());
        }
        &_ => {
			
		}
    }
		
	warn!("no known command or alias: {:?}", &command);
	return Err(Error::UnknownCommand(command));
}

const INT_ERR_TEXT: &str = "apologies, an internal error happend this has been reported and will be fixed as soon as possible";
const UNHANDLED: &str = "sorry I can not understand your input";
async fn handle_error(error: Error, chat_id: ChatId, token: &str) {
	let error_message = match error {
        Error::UnknownCommand(input) => 
            format!("Unknown command: {}", input),
        Error::HttpClientError(err) => {
			error!("Internal error in http client: {}", err);
			String::from(INT_ERR_TEXT)},
		Error::InvalidServerResponse(resp) => {
			error!("Incorrect bot api response {:?}", resp);
			String::from(INT_ERR_TEXT)},
		Error::SongDownloadError(e) => {
			error!("song download error {:?}", e);
			String::from("Could not download song due to critical internal error")
		}
		Error::Unauthorized(chat_id) => {
			format!("You are not authorized to use this bot, if you \
			believe this is a mistake ask for your telegram chat id \
			{} to be added", chat_id)
		}
		Error::UnhandledMessageKind => {
			String::from(UNHANDLED)}
		Error::UnhandledUpdateKind => {
			String::from(UNHANDLED)}
		Error::CouldNotSetWebhook => unreachable!(),
	};
	if let Err(error) = send_text(chat_id, token, error_message).await{
		error!("Could not send text reply to user: {:?}", error);
	}
}

async fn handle(update: Update, state: State){
	let token = &state.bot_token;
	if let Ok((text, chat_id, user_id, message_id)) = to_string_and_ids(update){
		if let Err(error) = handle_command(text, chat_id, 
			message_id, user_id, &state).await{
			handle_error(error, chat_id, token).await;
		}
	}
}

pub async fn handle_webhook(state: Data<State>, raw_update: Bytes)
	 -> HttpResponse {
	let update: Update = serde_json::from_slice(&raw_update.to_vec()).unwrap();
	let state_cpy = state.get_ref().clone();
	handle(update, state_cpy).await;

	HttpResponse::Ok()
		.status(StatusCode::OK)
		.body("{}")
}

pub async fn send_text<T: Into<String>>(chat_id: ChatId, token: &str, text: T)
	 -> Result<(), Error>{//add as arg generic ToChatRef (should get from Update)
	//TODO create a SendMessage, serialise it (use member function serialize) 
	//then use the HttpRequest fields, (url, method, and body) to send to telegram
	let url = format!("https://api.telegram.org/bot{}/sendMessage", token);	
	let form = reqwest::multipart::Form::new()
		.text("chat_id", chat_id.to_string())
		.text("text", text.into());

	let client = reqwest::Client::new();
	let resp = client.post(&url)
		.multipart(form).send().await?;
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
	let webhook_url = format!("{}:{}/{}",domain, port, token);

	let params = [("url", &webhook_url)];
	let client = reqwest::Client::new();
	let res = client.post(url.as_str())
	      .form(&params)
		  .send().await?;
	
	if res.status() != reqwest::StatusCode::OK {
		Err(Error::CouldNotSetWebhook)
	} else {
		info!("set webhook to: {}", webhook_url);
		Ok(())
	}
}