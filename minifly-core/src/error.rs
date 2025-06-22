use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Machine not found: {0}")]
    MachineNotFound(String),
    
    #[error("App not found: {0}")]
    AppNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Docker error: {0}")]
    DockerError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Lease conflict")]
    LeaseConflict,
    
    #[error("Invalid lease nonce")]
    InvalidLeaseNonce,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("LiteFS error: {0}")]
    LiteFSError(String),
    
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;