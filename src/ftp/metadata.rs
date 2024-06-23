use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use libunftp::storage::{Fileinfo, Metadata};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Meta {
    pub(crate) len: u64,
    pub(crate) is_dir: bool,
    pub(crate) is_file: bool,
    pub(crate) is_symlink: bool,
    pub(crate) modified: SystemTime,
    pub(crate) gid: u32,
    pub(crate) uid: u32,
    pub(crate) ids_and_urls: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize)]
pub struct Files {
    pub(crate) files: HashMap<PathBuf, Meta>,
    pub(crate) folders: HashMap<PathBuf, Vec<PathBuf>>,
}

impl Meta {
    pub fn from_json(path: PathBuf) -> Self {
        serde_json::from_str::<Files>(&*fs::read_to_string("data.json").unwrap())
            .unwrap()
            .files
            .get(&path)
            .unwrap()
            .clone()
    }
}

impl Files {
    pub fn to_json(&self) -> std::io::Result<()> {
        let result = fs::write("data.json", serde_json::to_string(self).unwrap());
        result
    }
    pub fn from_json() -> Self {
        serde_json::from_str::<Files>(&*fs::read_to_string("data.json").unwrap()).unwrap()
    }

    pub fn to_vec_fileinfo(&self, path: PathBuf) -> Vec<Fileinfo<PathBuf, Meta>> {
        let mut fileinfo = Vec::new();
        if let Some(folders) = self.folders.get(&path) {
            for path in folders {
                fileinfo.push(Fileinfo {
                    path: path.clone(),
                    metadata: self.files.get(path).unwrap().clone(),
                });
            }
        }
        fileinfo
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
        Ok(self.modified)
    }

    fn gid(&self) -> u32 {
        self.gid
    }

    fn uid(&self) -> u32 {
        self.uid
    }
}
