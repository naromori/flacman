use thiserror::Error;


#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Capacity Error: {0}")]
    CapacityError(#[from] heapless::CapacityError),

    #[error("ParseError: {0}")]
    ParseError(#[from] std::string::ParseError)

}

pub type Result<T> = std::result::Result<T, CoreError>;