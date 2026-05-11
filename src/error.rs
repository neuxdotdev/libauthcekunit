use thiserror::Error;
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("CSRF token not found in HTML")]
    TokenNotFound,
    #[error("Token validation failed: {reason}")]
    TokenInvalid { reason: String },
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    #[error("Unexpected HTTP status: {0}")]
    HttpStatus(u16),
    #[error("Login failed: {0}")]
    LoginFailed(String),
}
pub type Result<T> = std::result::Result<T, AuthError>;
