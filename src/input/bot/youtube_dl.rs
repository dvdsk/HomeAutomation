use telegram_bot::types::refs::{ChatId, MessageId};
use async_trait::async_trait;

use crate::input::youtube_downloader::{Feedback, JobStatus, MetaData};
use super::{send_text, Error};

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
    async fn ask_name_artist(&self, token: &str, metadata: MetaData)
    -> Result<(), Error> {
        dbg!();
        let keyboard_json = "[\
            [{\"text\":\"swap\", \"callback_data\":\"swap\"}],\
            [{\"text\":\"none\", \"callback_data\":\"none\"}],\
            [{\"text\":\"ok\", \"callback_data\":\"none\"}],\
            [{\"text\":\"lookup\", \"callback_data\":\"none\"}]\
        ]";
        let reply_markup = format!("{{\"inline_keyboard\":{} }}", keyboard_json);
        let text = format!("is _{}_ the title and the {} the artist?", 
            metadata.title, metadata.artist);

        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);	
        let form = reqwest::multipart::Form::new()
            .text("chat_id", self.chat_id.to_string())
            .text("text", text)
            .text("parse_mode", "MarkdownV2");
            //.text("reply_to_message_id", self.message_id.to_string())
            //.text("reply_markup", reply_markup);

        dbg!();
        let client = reqwest::Client::new();
        let resp = client.post(&url)
            .multipart(form).send().await?;
        //https://stackoverflow.com/questions/57540455/error-blockingclientinfuturecontext-when-trying-to-make-a-request-from-within
        
        if resp.status() != reqwest::StatusCode::OK {
            error!("telegram gave invalid response: {:?}", resp);
            Err(Error::InvalidServerResponse(resp))
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
        let res = match status {
            JobStatus::Finished => {
                send_text(self.chat_id, token, "finished").await
            },
            JobStatus::Downloaded => {
                send_text(self.chat_id, token, "done downloading").await
            },
            JobStatus::Queued(meta_data) => {
                self.ask_name_artist(token, meta_data).await
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