use std::path::{Path, PathBuf};


pub enum Ext {
    FLAC,
    MP3,
    MP4,
    OGG,
}

pub struct Filter {
    pub ext: []
}

pub struct Scanner {
    pub path: Option<Path>,
    pub cached: Option<Vec<Path>>,
    pub filter: Filter,
}


