use std::path::PathBuf;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum TagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path was not found: {0}")]
    NotFound(PathBuf),

    #[error("You have no permission to work with that file: {0}")]
    PermissionError(PathBuf),

    #[error("Cannot operate on directory: {0}")]
    NotAFile(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Error reading file metadata: {0}")]
    LoftyReadError(#[from] lofty::error::LoftyError)
    
}

pub type Result<T> = std::result::Result<T, TagError>;