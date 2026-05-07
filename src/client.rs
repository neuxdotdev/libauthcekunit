use crate::config::CONFIG;
use crate::cookies::CookieJar;
use crate::error::{AuthError, Result};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use reqwest::blocking::Client;
use serde_urlencoded;
use std::time::Duration;
use tracing::{debug, error, warn};
pub struct RobustHttpClient {
    inner: Client,
}
impl RobustHttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(CONFIG.request_timeout_secs))
            .user_agent(&CONFIG.user_agent)
            .redirect(if CONFIG.follow_redirects {
                reqwest::redirect::Policy::limited(CONFIG.max_redirects as usize)
            } else {
                reqwest::redirect::Policy::none()
            })
            .cookie_store(false)
            .build()
            .map_err(|e| AuthError::Http(e))?;
        Ok(Self { inner: client })
    }
    pub fn get(&self, url: &str, cookies: &CookieJar) -> Result<(String, CookieJar)> {
        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(CONFIG.retry_initial_interval_ms),
            max_interval: Duration::from_secs(5),
            max_elapsed_time: Some(Duration::from_secs(
                CONFIG.request_timeout_secs * CONFIG.max_retries as u64,
            )),
            ..Default::default()
        };
        loop {
            debug!("Sending GET request to {}", url);
            let url_parsed =
                url::Url::parse(url).map_err(|_| AuthError::Config("Invalid URL".into()))?;
            let cookie_header = cookies.cookie_header(&url_parsed);
            let mut req = self.inner.get(url);
            if !cookie_header.is_empty() {
                req = req.header("Cookie", cookie_header);
            }
            let response = req.send();
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let mut new_jar = cookies.clone();
                        for cookie_header in resp.headers().get_all(reqwest::header::SET_COOKIE) {
                            if let Ok(cookie_str) = cookie_header.to_str() {
                                new_jar.add_from_set_cookie(cookie_str, &url_parsed);
                            }
                        }
                        match resp.text() {
                            Ok(body) => {
                                new_jar.clear_expired();
                                debug!("Received {} bytes", body.len());
                                return Ok((body, new_jar));
                            }
                            Err(e) => return Err(AuthError::Http(e)),
                        }
                    } else if status.is_server_error() {
                        warn!("Server error {}, will retry", status);
                        if let Some(delay) = backoff.next_backoff() {
                            std::thread::sleep(delay);
                            continue;
                        } else {
                            error!("Max retries exceeded");
                            return Err(AuthError::MaxRetriesExceeded);
                        }
                    } else {
                        error!("Client error {}, not retrying", status);
                        return Err(AuthError::HttpStatus(status.as_u16()));
                    }
                }
                Err(e) => {
                    warn!("Request failed: {}, will retry", e);
                    if let Some(delay) = backoff.next_backoff() {
                        std::thread::sleep(delay);
                        continue;
                    } else {
                        error!("Max retries exceeded for error: {}", e);
                        return Err(AuthError::MaxRetriesExceeded);
                    }
                }
            }
        }
    }
    pub fn post_form(
        &self,
        url: &str,
        params: &[(&str, &str)],
        cookies: &CookieJar,
    ) -> Result<(String, CookieJar)> {
        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(CONFIG.retry_initial_interval_ms),
            max_interval: Duration::from_secs(5),
            max_elapsed_time: Some(Duration::from_secs(
                CONFIG.request_timeout_secs * CONFIG.max_retries as u64,
            )),
            ..Default::default()
        };
        loop {
            debug!("Sending POST request to {}", url);
            let url_parsed =
                url::Url::parse(url).map_err(|_| AuthError::Config("Invalid URL".into()))?;
            let cookie_header = cookies.cookie_header(&url_parsed);
            let body = serde_urlencoded::to_string(params)
                .map_err(|e| AuthError::Config(format!("Failed to encode form: {}", e)))?;
            let mut req = self
                .inner
                .post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body);
            if !cookie_header.is_empty() {
                req = req.header("Cookie", cookie_header);
            }
            let response = req.send();
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status == 302 || status == 303 {
                        let mut new_jar = cookies.clone();
                        for cookie_header in resp.headers().get_all(reqwest::header::SET_COOKIE) {
                            if let Ok(cookie_str) = cookie_header.to_str() {
                                new_jar.add_from_set_cookie(cookie_str, &url_parsed);
                            }
                        }
                        match resp.text() {
                            Ok(body) => {
                                new_jar.clear_expired();
                                debug!("Received {} bytes, status {}", body.len(), status);
                                return Ok((body, new_jar));
                            }
                            Err(e) => return Err(AuthError::Http(e)),
                        }
                    } else if status.is_server_error() {
                        warn!("Server error {}, will retry", status);
                        if let Some(delay) = backoff.next_backoff() {
                            std::thread::sleep(delay);
                            continue;
                        } else {
                            error!("Max retries exceeded");
                            return Err(AuthError::MaxRetriesExceeded);
                        }
                    } else {
                        error!("Client error {}, not retrying", status);
                        return Err(AuthError::HttpStatus(status.as_u16()));
                    }
                }
                Err(e) => {
                    warn!("Request failed: {}, will retry", e);
                    if let Some(delay) = backoff.next_backoff() {
                        std::thread::sleep(delay);
                        continue;
                    } else {
                        error!("Max retries exceeded for error: {}", e);
                        return Err(AuthError::MaxRetriesExceeded);
                    }
                }
            }
        }
    }
}
