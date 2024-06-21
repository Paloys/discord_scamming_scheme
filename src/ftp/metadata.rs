use libunftp::storage::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Meta {
    len: u64,
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    modified: SystemTime,
    gid: u32,
    uid: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Files {
    files: HashMap<PathBuf, Meta>,
}

impl Meta {
    pub fn from_json(path: PathBuf) -> Self {
        serde_json::from_str::<Files>(&*fs::read_to_string("files.json").unwrap()).unwrap()[&path]
            .clone()
    }
}

impl Files {
    pub fn to_json(&self) {
        fs::write("files.json", serde_json::to_string(self).unwrap())
            .expect("Unable to write file");
    }
    pub fn from_json() -> Self {
        serde_json::from_str::<Files>(&*fs::read_to_string("files.json").unwrap()).unwrap()
    }
}

impl Metadata for Meta {
    fn len(&self) -> u64 {
        self.len
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        self.is_file
    }

    fn is_symlink(&self) -> bool {
        self.is_symlink
    }

    fn modified(&self) -> libunftp::storage::Result<SystemTime> {
        self.modified.map_err(Into::into)
    }

    fn gid(&self) -> u32 {
        self.gid
    }

    fn uid(&self) -> u32 {
        self.uid
    }
}
