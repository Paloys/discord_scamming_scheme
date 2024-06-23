use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{env, fs};

use async_trait::async_trait;
use libunftp::auth::UserDetail;
use libunftp::storage::{Fileinfo, Metadata, StorageBackend};
use libunftp::ServerBuilder;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use discord::bot::Bot;
use ftp::limited_reader::LimitedAsyncReader;
use ftp::metadata::Meta;

use crate::discord::datatypes::message::Message;
use crate::ftp::metadata::Files;

mod discord;
mod ftp;

#[derive(Debug)]
pub struct DiscordBackend {
    client: Bot,
}

impl DiscordBackend {
    pub fn new(client: Bot) -> Self {
        DiscordBackend { client }
    }

    async fn handle_file(&self, data: Vec<u8>) -> Result<(u64, String, String), String> {
        let size = data.len();
        let response = self
            .client
            .send_message(1252591907670982795, "", Some(data), None)
            .await
            .expect("Failed to send message");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let message = response.json::<Message>().await.unwrap();
        Ok((size as u64, message.id.unwrap(), message.attachments[0].url.clone()))
    }
}

#[async_trait]
impl<User: UserDetail> StorageBackend<User> for DiscordBackend {
    type Metadata = Meta;

    async fn metadata<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<Self::Metadata> {
        Ok(Meta::from_json(path.as_ref().to_path_buf()))
    }

    async fn list<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<Vec<Fileinfo<PathBuf, Self::Metadata>>>
    where
        <Self as StorageBackend<User>>::Metadata: Metadata,
    {
        Ok(serde_json::from_str::<Files>(&*fs::read_to_string("data.json").unwrap())
            .unwrap()
            .to_vec_fileinfo(path.as_ref().to_path_buf()))
    }

    async fn get<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
        start_pos: u64,
    ) -> libunftp::storage::Result<Box<dyn AsyncRead + Send + Sync + Unpin>> {
        todo!("get")
    }

    async fn put<P: AsRef<Path> + Send + Debug, R: AsyncRead + Send + Sync + Unpin + 'static>(
        &self,
        _user: &User,
        input: R,
        path: P,
        _start_pos: u64,
    ) -> libunftp::storage::Result<u64> {
        let mut files = Files::from_json();
        files
            .folders
            .get_mut(&path.as_ref().parent().unwrap().to_path_buf())
            .unwrap()
            .push(path.as_ref().to_path_buf());
        let mut metadata = Meta {
            len: 0,
            is_dir: false,
            is_file: true,
            is_symlink: false,
            modified: std::time::SystemTime::now(),
            gid: 0,
            uid: 0,
            ids_and_urls: Vec::new(),
        };
        let path = path.as_ref().to_path_buf();
        let mut tot = 0u64;

        let mut reader = BufReader::with_capacity(1 << 20, input);

        loop {
            let mut limited_reader = LimitedAsyncReader::new(&mut reader, 25_000_000);

            let mut buffer = Vec::with_capacity(25_000_000);

            let bytes_copied = tokio::io::copy(&mut limited_reader, &mut buffer).await?;

            if bytes_copied == 0 {
                break;
            }

            let (bytes_uploaded, id, url) = self.handle_file(buffer).await.expect("Failed to handle file");
            metadata.len += bytes_uploaded;
            metadata.ids_and_urls.push((id, url));

            tot += bytes_uploaded;
        }
        files.files.insert(path, metadata);
        files.to_json().expect("Failed to write to data.json");
        Ok(tot)
    }

    async fn del<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<()> {
        let mut files = Files::from_json();
        let path = path.as_ref().to_path_buf();
        for (id, _) in files.files.get(&path).unwrap().ids_and_urls.iter() {
            self.client
                .delete_message(1252591907670982795, id.parse().unwrap())
                .await
                .expect("Failed to delete message");
        }
        files.files.remove(&path);
        files
            .folders
            .get_mut(&path.parent().unwrap().to_path_buf())
            .unwrap()
            .retain(|x| x != &path);
        Ok(files.to_json().expect("Failed to write to data.json"))
    }

    async fn mkd<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<()> {
        let mut files = Files::from_json();
        let path = path.as_ref().to_path_buf();
        files.folders.insert(path.clone(), Vec::new());
        files.folders.get_mut(&path.parent().unwrap().to_path_buf()).unwrap().push(path.clone());
        files.files.insert(
            path,
            Meta {
                len: 0,
                is_dir: true,
                is_file: false,
                is_symlink: false,
                modified: std::time::SystemTime::now(),
                gid: 0,
                uid: 0,
                ids_and_urls: Vec::new(),
            },
        );
        Ok(files.to_json().expect("Failed to write to data.json"))
    }

    async fn rename<P: AsRef<Path> + Send + Debug>(&self, _user: &User, from: P, to: P) -> libunftp::storage::Result<()> {
        todo!("rename")
    }

    async fn rmd<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<()> {
        todo!("rmd")
    }

    async fn cwd<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new(Box::new(|| {
        DiscordBackend::new(Bot::new(env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set")))
    }))
    .greeting("Welcome to my FTP server")
    .passive_ports(50000..65535)
    .build()
    .unwrap();

    println!("Server running on localhost:2121");
    server.listen("127.0.0.1:2121").await.expect("Server crashed");
}
