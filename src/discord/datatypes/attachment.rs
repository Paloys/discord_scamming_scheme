use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Attachment {
    pub(crate) id: String,
    pub(crate) filename: String,
    pub(crate) size: u64,
    pub(crate) url: String,
}

impl Attachment {
    #[allow(dead_code)]
    pub(crate) async fn download(&self) -> Vec<u8> {
        reqwest::get(&self.url).await.unwrap().bytes().await.unwrap().to_vec()
    }
}
