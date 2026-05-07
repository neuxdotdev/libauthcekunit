
pub mod auth;
pub mod client;
pub mod config;
pub mod cookies;
pub mod error;
pub mod logging;
pub mod parser;
pub mod types;
pub use auth::{login, logout};
pub use cookies::CookieJar;
pub use error::{AuthError, Result};
pub use types::{CsrfToken, FetchOptions};
pub use config::CONFIG;
pub use logging::init_logging;
pub use parser::extract_token;
pub mod ffi;
use tracing::info;
pub fn fetch_token_and_cookies(url: &str) -> Result<(String, CookieJar)> {
    init_logging();
    info!("Fetching token and cookies from {}", url);
    let http_client = client::RobustHttpClient::new()?;
    let (html, cookie_jar) = http_client.get(url, &CookieJar::new())?;
    let token = extract_token(&html)?;
    info!("Token extracted: {}...", &token[..token.len().min(8)]);
    Ok((token, cookie_jar))
}
pub fn fetch_token(url: &str) -> Result<String> {
    let (token, _) = fetch_token_and_cookies(url)?;
    Ok(token)
}
pub fn fetch_cookies(url: &str) -> Result<CookieJar> {
    let (_, jar) = fetch_token_and_cookies(url)?;
    Ok(jar)
}