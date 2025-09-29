use std::path::PathBuf;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path was not found: {0}")]
    PathNotFound(PathBuf),

    #[error("File already exists: {0}")]
    FileAlreadyExists(PathBuf),

    #[error("Source and destination are the same: {0}")]
    SameFile(PathBuf),

    #[error("You have no permission to edit that file: {0}")]
    PermissionError(PathBuf),

    #[error("Cannot operate on directory: {0}")]
    NotAFile(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Error while walking directory")]
    WalkDir(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, FsError>;