use crate::discord::datatypes::message::{Message, MessageReference};

pub(crate) struct Bot {
    client: reqwest::Client,
    token: String,
}

impl Bot {
    pub(crate) fn new(token: String) -> Result<Self, reqwest::Error> {
        Ok(Bot {
            client: reqwest::Client::new(),
            token,
        })
    }

    pub(crate) async fn get_guilds(&self) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get("https://discord.com/api/v10/users/@me/guilds")
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await
    }

    pub(crate) async fn get_message(
        &self,
        channel_id: u64,
        message_id: u64,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get(format!(
                "https://discord.com/api/v10/channels/{}/messages/{}",
                channel_id, message_id
            ))
            .header("Authorization", format!("Bot {}", self.token))
            .send()
            .await
    }

    pub(crate) async fn send_message(
        &self,
        channel_id: u64,
        content: &str,
        attachment: Option<Vec<u8>>,
        reply: Option<String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut form = reqwest::multipart::Form::new().text("content", content.to_string());
        if let Some(attachment) = attachment {
            form = form.part(
                "file",
                reqwest::multipart::Part::bytes(attachment).file_name("file"),
            );
        }
        form = form.text(
            "payload_json",
            serde_json::to_string(&Message {
                id: None,
                content: content.to_string(),
                channel_id: channel_id.to_string(),
                attachments: vec![],
                message_reference: match reply {
                    None => None,
                    Some(reply) => Some(MessageReference { message_id: reply }),
                },
            })
            .unwrap(),
        );
        self.client
            .post(format!(
                "https://discord.com/api/v10/channels/{}/messages",
                channel_id
            ))
            .header("Authorization", format!("Bot {}", self.token))
            .multipart(form)
            .send()
            .await
    }
}
