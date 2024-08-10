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

    pub(crate) async fn download_attachment(&self, url: &String, message_id: &String) -> Result<Vec<u8>, Error> {
        let mut response = self.client.get(url).send().await?;

        let text = response.text().await?;
        if text == "This content is no longer available." {
            let url2 = self.client
                .get(format!(
                    "https://discord.com/api/v10/channels/{}/messages/{}",
                    self.channel_id, message_id
                ))
                .header("Authorization", format!("Bot {}", self.token))
                .send()
                .await?
                .json::<Message>()
                .await?
                .attachments
                .get(0)
                .unwrap()
                .url
                .clone();

            response = self.client.get(&url2).send().await?;
        } else {
            // If the content is not "This content is no longer available."
            // you may want to handle `response` as bytes instead of converting to text.
            response = self.client.get(url).send().await?;
        }

        response.bytes().await.map(|bytes| bytes.to_vec())
    }
}
