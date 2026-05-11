use crate::error::{AuthError, Result};
use once_cell::sync::Lazy;
use std::env;
use tracing::debug;
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env().expect("Failed to load config"));
#[derive(Debug, Clone)]
pub struct Config {
    pub request_timeout_secs: u64,
    pub max_retries: u32,
    pub retry_initial_interval_ms: u64,
    pub max_redirects: u32,
    pub follow_redirects: bool,
    pub token_min_length: usize,
    pub token_max_length: usize,
    pub log_level: String,
    pub user_agent: String,
    pub email: String,
    pub password: String,
    pub login_verify_url: String,
    pub login_verify_text: String,
}
impl Config {
    pub fn from_env() -> Result<Self> {
        if let Ok(env_path) = env::var("LIB_CEKUNIT_AUTH_ENV_ENV_PATH") {
            dotenv::from_path(env_path).ok();
        } else {
            dotenv::dotenv().ok();
        }
        debug!("Loading environment variables (email/password hidden)");
        let parse_u64 = |key: &str, default: u64| -> u64 {
            env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        let parse_u32 = |key: &str, default: u32| -> u32 {
            env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        let parse_bool = |key: &str, default: bool| -> bool {
            env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        let parse_string =
            |key: &str, default: String| -> String { env::var(key).unwrap_or(default) };
        Ok(Config {
            request_timeout_secs: parse_u64("LIB_CEKUNIT_AUTH_ENV_TIMEOUT_SECS", 30),
            max_retries: parse_u32("LIB_CEKUNIT_AUTH_ENV_MAX_RETRIES", 3),
            retry_initial_interval_ms: parse_u64("LIB_CEKUNIT_AUTH_ENV_RETRY_INITIAL_MS", 500),
            max_redirects: parse_u32("LIB_CEKUNIT_AUTH_ENV_MAX_REDIRECTS", 5),
            follow_redirects: parse_bool("LIB_CEKUNIT_AUTH_ENV_FOLLOW_REDIRECTS", true),
            token_min_length: parse_u64("LIB_CEKUNIT_AUTH_ENV_TOKEN_MIN_LEN", 10)
                .try_into()
                .map_err(|_| AuthError::Config("min length too large".into()))?,
            token_max_length: parse_u64("LIB_CEKUNIT_AUTH_ENV_TOKEN_MAX_LEN", 1024)
                .try_into()
                .map_err(|_| AuthError::Config("max length too large".into()))?,
            log_level: parse_string("RUST_LOG", "info".to_string()),
            user_agent: parse_string(
                "LIB_CEKUNIT_AUTH_ENV_USER_AGENT",
                "LIB_CEKUNIT_AUTH_ENVUnit/2.0".to_string(),
            ),
            email: parse_string("LIB_CEKUNIT_AUTH_ENV_EMAIL", "".to_string()),
            password: parse_string("LIB_CEKUNIT_AUTH_ENV_PASSWORD", "".to_string()),
            login_verify_url: parse_string(
                "LIB_CEKUNIT_AUTH_ENV_LOGIN_VERIFY_URL",
                "/dashboard".to_string(),
            ),
            login_verify_text: parse_string(
                "LIB_CEKUNIT_AUTH_ENV_LOGIN_VERIFY_TEXT",
                "".to_string(),
            ),
        })
    }
}
