use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Attachment {
    pub(crate) id: String,
    pub(crate) filename: String,
    pub(crate) size: u64,
    pub(crate) url: String,
}
