use crate::discord::datatypes::message::{Message, MessageReference};
use reqwest::multipart::{Form, Part};
use reqwest::{Error, Response};

#[derive(Debug)]
pub struct Bot {
    client: reqwest::Client,
    token: String,
    channel_id: u64,
}

impl Bot {
    pub(crate) fn new(token: String, channel_id: u64) -> Self {
        Bot {
            client: reqwest::Client::new(),
            token,
            channel_id,
        }
    }

    pub(crate) async fn send_message(&self, content: &str, attachment: Option<Vec<u8>>, reply: Option<String>) -> Result<Response, Error> {
        let mut form = Form::new().text("content", content.to_string());
        if let Some(attachment) = attachment {
            form = form.part("file", Part::bytes(attachment).file_name("file"));
        }
        form = form.text(
            "payload_json",
            serde_json::to_string(&Message {
                id: None,
                content: content.to_string(),
                channel_id: self.channel_id.to_string(),
                attachments: vec![],
                message_reference: match reply {
                    None => None,
                    Some(reply) => Some(MessageReference { message_id: reply }),
                },
            })
            .unwrap(),
        );
        self.client
            .post(format!("https://discord.com/api/v10/channels/{}/messages", self.channel_id))
            .header("Authorization", format!("Bot {}", self.token))
            .multipart(form)
            .send()
            .await
    }

    pub(crate) async fn delete_message(&self, message_id: u64) -> Result<Response, Error> {
        self.client
            .delete(format!(
                "https://discord.com/api/v10/channels/{}/messages/{}",
                self.channel_id, message_id
            ))
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await
    }

    pub(crate) async fn download_attachment(&self, url: &String) -> Result<Vec<u8>, Error> {
        self.client
            .get(url)
            // .header("Authorization", format!("Bot {}", self.token)) // Unneeded ?
            .send()
            .await?
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
    }
}
