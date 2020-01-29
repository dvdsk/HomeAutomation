use actix_web::web::{HttpResponse, Data, Bytes};
use actix_web::http::{StatusCode};

use reqwest;
use log::{warn, info ,error};
use serde_json;

use telegram_bot::types::update::Update;
use telegram_bot::types::update::UpdateKind;
use telegram_bot::types::message::MessageKind;
use telegram_bot::types::refs::{ChatId, UserId};

use crate::input::web_api::server::{State};

#[derive(Debug)]
pub enum Error{
	HttpClientError(reqwest::Error),
	CouldNotSetWebhook,
	InvalidServerResponse(reqwest::Response),
	InvalidServerResponseBlocking(reqwest::blocking::Response),
	UnhandledUpdateKind,
    UnhandledMessageKind,
    UnknownCommand(String)
}

impl From<reqwest::Error> for Error {
	fn from(error: reqwest::Error) -> Self {
		Error::HttpClientError(error)
	}
}

fn to_string_and_ids(update: Update) -> Result<(String, ChatId, UserId),Error>{
	if let UpdateKind::Message(message) = update.kind {
		let chat_id = message.chat.id();
		let user_id = message.from.id;
		if let MessageKind::Text{data, entities:_} = message.kind {
			return Ok((data, chat_id, user_id));
		} else {
			warn!("unhandled message kind");
			return Err(Error::UnhandledUpdateKind);
		}
	} else {
		warn!("unhandled update from telegram: {:?}", update);
		return Err(Error::UnhandledMessageKind);
	}
}

async fn handle_command(mut text: String, chat_id: ChatId,
	state: &State) -> Result<(), Error>{
	
	let token = &state.bot_token;

    let split = text.find(char::is_whitespace);
    let mut command = text;
    let args = command.split_off(split.unwrap_or(command.len()));
    match command.as_str() {
        "/test" => {
            send_text_reply(chat_id, token, "hi").await?; 
            return Ok(());
        }
        /*"/help" => {
            help::send(chat_id, &user, token).await?; 
            break;
        }
        "/plotables" => {
            plotables::send(chat_id, &user, state, token).await?;
            break;
        }
        "/show" => {
            show::send(chat_id, state, token, args, &user).await?;
            break;
        }
        "/keyboard" => {
            keyboard::show(chat_id, token, user).await?;
            break;
        }
        "/keyboard_add" => {
            keyboard::add_button(chat_id, state, token, args, user).await?;
            break;
        }
        "/keyboard_remove" => {
            keyboard::remove_button(chat_id, state, token, args, user).await?;
            break;
        }
        "/alarm" => {
            alarms::handle(chat_id, token, args, user, state).await?;
            break;
        }	
        "/alias" => {
            alias::send(chat_id, state, token, args, user).await?;
            break;
        }*/
        &_ => {}
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
		Error::UnhandledMessageKind => {
			String::from(UNHANDLED)}
		Error::UnhandledUpdateKind => {
			String::from(UNHANDLED)}
		Error::CouldNotSetWebhook => unreachable!(),
		Error::InvalidServerResponseBlocking(_) => unreachable!(),
	};
	if let Err(error) = send_text_reply(chat_id, token, error_message).await{
		error!("Could not send text reply to user: {:?}", error);
	}
}

async fn handle(update: Update, state: State){
	let token = &state.bot_token;
	if let Ok((text, chat_id, _user_id)) = to_string_and_ids(update){
		if let Err(error) = handle_command(text, chat_id, &state).await{
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

pub async fn send_text_reply<T: Into<String>>(chat_id: ChatId, token: &str, text: T)
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

pub fn send_text_reply_blocking<T: Into<String>>(chat_id: ChatId, token: &str, text: T)
	 -> Result<(), Error>{//add as arg generic ToChatRef (should get from Update)
	//TODO create a SendMessage, serialise it (use member function serialize) 
	//then use the HttpRequest fields, (url, method, and body) to send to telegram
	let url = format!("https://api.telegram.org/bot{}/sendMessage", token);	
	let form = reqwest::blocking::multipart::Form::new()
		.text("chat_id", chat_id.to_string())
		.text("text", text.into());

	let client = reqwest::blocking::Client::new();
	let resp = client.post(&url)
		.multipart(form).send()?;
	//https://stackoverflow.com/questions/57540455/error-blockingclientinfuturecontext-when-trying-to-make-a-request-from-within
	
	if resp.status() != reqwest::StatusCode::OK {
		Err(Error::InvalidServerResponseBlocking(resp))
	} else {
		info!("send message");
		Ok(())
	}
}

pub async fn set_webhook(domain: &str, token: &str, port: u16) -> Result<(), Error> {
	let url = format!("https://api.telegram.org/bot{}/setWebhook", token);
	let webhook_url = format!("{}:{}/{}",domain, port, token);
	dbg!(&webhook_url);

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