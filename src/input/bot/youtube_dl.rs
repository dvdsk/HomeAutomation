use telegram_bot::types::refs::{ChatId, MessageId};
use telegram_bot::types::callback_query::CallbackQuery;
use async_trait::async_trait;

use crate::input::youtube_downloader::{self, Feedback, JobStatus, MetaGuess};
use crate::input::web_api::server::{State};
use super::send_text;
use super::Error as BotError;

#[derive(Debug, Clone)]
pub struct TelegramFeedback {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

//present options: 
//  Swap
//  None
//  Ok
//  Lookup
impl TelegramFeedback {
    async fn ask_name_artist(&self, token: &str, meta: MetaGuess, id: u64)
    -> Result<(), BotError> {
        dbg!();
        let keyboard_json = format!("[\
            [{{\"text\":\"swap\", \"callback_data\":\"ytdl:swap:{id}:{title}:{artist}\"}}],\
            [{{\"text\":\"no\", \"callback_data\":\"ytdl:no:{id}:{title}:{artist}\"}}],\
            [{{\"text\":\"ok\", \"callback_data\":\"ytdl:ok:{id}:{title}:{artist}\"}}],\
            [{{\"text\":\"lookup\", \"callback_data\":\"ytdl:lookup:{id}:{title}:{artist}\"}}]\
        ]",id=id,title=meta.title, artist=meta.artist);
        let reply_markup = format!("{{\"inline_keyboard\":{} }}", keyboard_json);
        let text = format!("is _{}_ the title and _{}_ the artist?", 
            meta.title, meta.artist);
        //TODO: should always base this on metadata guess, should make that a
        //seperate type

        dbg!(&text);
        dbg!(&reply_markup);
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);	
        let form = reqwest::multipart::Form::new()
            .text("chat_id", self.chat_id.to_string())
            .text("text", text)
            .text("parse_mode", "Markdown")
            .text("reply_to_message_id", self.message_id.to_string());
            //.text("reply_markup", reply_markup);

        dbg!();
        let client = reqwest::Client::new();
        let resp = client.post(&url)
            .multipart(form).send().await?;
        //https://stackoverflow.com/questions/57540455/error-blockingclientinfuturecontext-when-trying-to-make-a-request-from-within
        
        if resp.status() != reqwest::StatusCode::OK {
            error!("telegram gave invalid response: {:?}", resp);
            Err(BotError::InvalidServerResponse(resp))
        } else {
            dbg!("send message");
            Ok(())
        }

    }
}

#[async_trait]
impl Feedback for TelegramFeedback {
    //errors during feedback must be handled within feedback channel
    async fn feedback(&self, status: JobStatus, token: &str) {
        let res: Result<(),BotError> = match status {
            JobStatus::Finished => {
                send_text(self.chat_id, token, "finished").await
            },
            JobStatus::Downloaded => {
                send_text(self.chat_id, token, "done downloading").await
            },
            JobStatus::Queued(meta_data, id) => {
                self.ask_name_artist(token, meta_data, id).await
                    .map_err(|e| e.into())
            },
            JobStatus::Error => {
                send_text(self.chat_id, token, "ran into error").await
            },
        };

        if let Err(e) = res {
            error!("ran into error within feedback function: {:?}", e);
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    YoutubeDlError(youtube_downloader::Error),
    NotEnoughData,
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseError(err)
    }
}

impl From<youtube_downloader::Error> for Error {
    fn from(err: youtube_downloader::Error) -> Error {
        Error::YoutubeDlError(err)
    }
}

pub async fn handle_callback(callback: CallbackQuery, state: &State)
    -> Result<(), Error> {

    let mut split = callback.data.split_terminator(":").skip(1);
    let command = split.next();
    match command {
        Some("swap") => {
            let id = split.next().ok_or(Error::NotEnoughData)?.parse()?;
            let title = split.next().ok_or(Error::NotEnoughData)?;
            let artist = split.next().ok_or(Error::NotEnoughData)?;
            state.youtube_dl.set_meta(id, artist, title).await? //artist an title swapped
        }
        Some("ok") => {
            let id = split.next().ok_or(Error::NotEnoughData)?.parse()?;
            let title = split.next().ok_or(Error::NotEnoughData)?;
            let artist = split.next().ok_or(Error::NotEnoughData)?;
            dbg!("handling callback");
            state.youtube_dl.set_meta(id, title, artist).await?
        }
        Some("no") => {
            let id = split.next().ok_or(Error::NotEnoughData)?.parse()?;
            let title = split.next().ok_or(Error::NotEnoughData)?;
            state.youtube_dl.no_meta(id, title).await?
        }
        Some("lookup") => todo!(),
        _ => {
            error!("cant handle youtube_dl callback command: {:?}", command);
        },
    }
    Ok(())
}