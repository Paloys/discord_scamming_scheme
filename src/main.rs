use std::fmt::Debug;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{env, fs};

use async_trait::async_trait;
use libunftp::auth::UserDetail;
use libunftp::storage::{Error, ErrorKind, Fileinfo, Metadata, StorageBackend};
use libunftp::ServerBuilder;
use tokio::io::{AsyncRead, BufReader};

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
        let response = self.client.send_message("", Some(data), None).await.expect("Failed to send message");
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
        _user: &User,
        path: P,
        _start_pos: u64,
    ) -> libunftp::storage::Result<Box<dyn AsyncRead + Send + Sync + Unpin>> {
        let metadata = Meta::from_json(path.as_ref().to_path_buf());
        let mut data_vec = Vec::new();
        for (id, url) in metadata.ids_and_urls.iter() {
            let bytes = self.client.download_attachment(url, id).await.unwrap();
            data_vec.extend_from_slice(&bytes);
        }
        let cursor = Cursor::new(data_vec);
        let async_reader = BufReader::with_capacity(1 << 20, cursor);
        Ok(Box::new(async_reader))
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
            self.client.delete_message(id.parse().unwrap()).await.expect("Failed to delete message");
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
        let mut files = Files::from_json();
        let from = from.as_ref().to_path_buf();
        let to = to.as_ref().to_path_buf();
        if files.files.get(&from).unwrap().is_dir {
            return Err(Error::new(ErrorKind::CommandNotImplemented, "Cannot rename folders (yet)"));
            // TODO: Implement this for folders
        }
        files
            .folders
            .get_mut(&from.parent().unwrap().to_path_buf())
            .unwrap()
            .retain(|x| x != &from);
        files.folders.get_mut(&to.parent().unwrap().to_path_buf()).unwrap().push(to.clone());
        let metadata = files.files.remove(&from).unwrap();
        files.files.insert(to.clone(), metadata);
        Ok(files.to_json().expect("Failed to write to data.json"))
    }

    async fn rmd<P: AsRef<Path> + Send + Debug>(&self, _user: &User, path: P) -> libunftp::storage::Result<()> {
        let mut files = Files::from_json();
        let path = path.as_ref().to_path_buf();
        files.files.remove(&path);
        files
            .folders
            .get_mut(&path.parent().unwrap().to_path_buf())
            .unwrap()
            .retain(|x| x != &path);
        files.folders.remove(&path);
        Ok(files.to_json().expect("Failed to write to data.json"))
    }

    async fn cwd<P: AsRef<Path> + Send + Debug>(&self, _user: &User, _path: P) -> libunftp::storage::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    if !Path::new("data.json").exists() {
        let files = Files::new();
        files.to_json().expect("Failed to write to data.json");
    }
    let server = ServerBuilder::new(Box::new(|| {
        DiscordBackend::new(Bot::new(
            env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set"),
            env::var("DISCORD_CHANNEL_ID").expect("DISCORD_CHANNEL_ID not set").parse().unwrap(),
        ))
    }))
        .greeting("Welcome to my FTP server")
        .passive_ports(50000..65535)
        .build()
        .unwrap();

    println!("Server running on localhost:2121");
    server.listen("127.0.0.1:2121").await.expect("Server crashed");
}
