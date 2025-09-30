use std::{fs::File, path::{Path, PathBuf}};

use flacman_core::String;
use lofty::file::TaggedFileExt;
use crate::tagerror::Result;


pub struct MediaFile {
    pub path: PathBuf,
    metadata: Option<Metadata>,
}

impl MediaFile {
    
    pub fn new(path: &Path) -> Self {
        MediaFile { path: path.to_path_buf() }
    }

    fn 

    pub fn read(&mut self) -> Result<&Metadata> {

        if let Some(metadata) = &self.metadata {
            return Ok(metadata);
        }

        let mut file = File::open(&self.path)?;
        let tagged_file = lofty::read_from(&mut file)?;
        let p_tag = tagged_file.primary_tag();


        return ;
    } 
}

pub struct Metadata {
    pub track_name: String,
    pub album: String,
    pub author: String,
}