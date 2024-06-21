use crate::discord::datatypes::attachment::Attachment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Message {
    pub(crate) id: Option<String>,
    pub(crate) content: String,
    pub(crate) channel_id: String,
    pub(crate) attachments: Vec<Attachment>,
    pub(crate) message_reference: Option<MessageReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MessageReference {
    pub(crate) message_id: String,
}
