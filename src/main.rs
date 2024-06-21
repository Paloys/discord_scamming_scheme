use async_trait::async_trait;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};

use ftp::metadata::Meta;
use libunftp::auth::UserDetail;
use libunftp::storage::{Fileinfo, Metadata, StorageBackend};
use tokio::io::AsyncRead;
use tokio::sync::RwLock;

mod discord;
mod ftp;

#[derive(Debug)]
pub struct DiscordBackend {
    files: Arc<RwLock<Vec<PathBuf>>>,
}

impl DiscordBackend {
    pub fn new() -> Self {
        DiscordBackend {
            files: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl<User: UserDetail> StorageBackend<User> for DiscordBackend {
    type Metadata = Meta;

    async fn metadata<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<Self::Metadata> {
        todo!("metadata")
    }

    async fn list<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<Vec<Fileinfo<PathBuf, Self::Metadata>>>
    where
        <Self as StorageBackend<User>>::Metadata: Metadata,
    {
        serde_json::from_str::<crate::ftp::metadata::Files>(
            &*fs::read_to_string("files.json").unwrap(),
        )
        .unwrap()
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
        user: &User,
        input: R,
        path: P,
        start_pos: u64,
    ) -> libunftp::storage::Result<u64> {
        todo!("put")
    }

    async fn del<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<()> {
        todo!("del")
    }

    async fn mkd<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<()> {
        todo!("mkd")
    }

    async fn rename<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        from: P,
        to: P,
    ) -> libunftp::storage::Result<()> {
        todo!("rename")
    }

    async fn rmd<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<()> {
        todo!("rmd")
    }

    async fn cwd<P: AsRef<Path> + Send + Debug>(
        &self,
        user: &User,
        path: P,
    ) -> libunftp::storage::Result<()> {
        todo!("cwd")
    }
}

#[tokio::main]
async fn main() {
    //let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
}
