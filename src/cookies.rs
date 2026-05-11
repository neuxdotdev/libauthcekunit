use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::debug;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<SystemTime>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}
impl Cookie {
    pub fn from_set_cookie(header: &str, request_url: &url::Url) -> Option<Self> {
        let parts: Vec<&str> = header.split(';').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return None;
        }
        let name_value = parts[0];
        let (name, value) = match name_value.find('=') {
            Some(idx) => (
                name_value[..idx].to_string(),
                name_value[idx + 1..].to_string(),
            ),
            None => (name_value.to_string(), String::new()),
        };
        let mut cookie = Cookie {
            name,
            value,
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        };
        for attr in &parts[1..] {
            if let Some(eq_pos) = attr.find('=') {
                let key = attr[..eq_pos].trim().to_lowercase();
                let val = attr[eq_pos + 1..].trim().to_string();
                match key.as_str() {
                    "domain" => {
                        cookie.domain = Some(val.trim_start_matches('.').to_string());
                    }
                    "path" => cookie.path = Some(val),
                    "expires" => {
                        if let Ok(exp) = DateTime::parse_from_rfc2822(&val) {
                            cookie.expires = Some(exp.into());
                        } else if let Ok(exp) = DateTime::parse_from_rfc3339(&val) {
                            cookie.expires = Some(exp.into());
                        }
                    }
                    "max-age" => {
                        if let Ok(secs) = val.parse::<i64>() {
                            cookie.expires = Some(
                                SystemTime::now()
                                    .checked_add(std::time::Duration::from_secs(secs as u64))
                                    .unwrap_or(SystemTime::now()),
                            );
                        }
                    }
                    "samesite" => cookie.same_site = Some(val.to_lowercase()),
                    _ => {}
                }
            } else {
                let key = attr.trim().to_lowercase();
                match key.as_str() {
                    "secure" => cookie.secure = true,
                    "httponly" => cookie.http_only = true,
                    _ => {}
                }
            }
        }
        if cookie.domain.is_none() {
            if let Some(host) = request_url.host_str() {
                cookie.domain = Some(host.to_string());
            }
        }
        if cookie.path.is_none() {
            cookie.path = Some(
                request_url
                    .path()
                    .rsplit_once('/')
                    .map(|(dir, _)| if dir.is_empty() { "/" } else { dir })
                    .unwrap_or("/")
                    .to_string(),
            );
        }
        if let Some(exp) = cookie.expires {
            if SystemTime::now() >= exp {
                return None;
            }
        }
        Some(cookie)
    }
    pub fn matches(&self, url: &url::Url) -> bool {
        if let Some(exp) = self.expires {
            if SystemTime::now() >= exp {
                return false;
            }
        }
        if let Some(ref domain) = self.domain {
            let host = match url.host_str() {
                Some(h) => h,
                None => return false,
            };
            let domain_with_dot = format!(".{}", domain);
            if host != domain && !host.ends_with(&domain_with_dot) {
                return false;
            }
        }
        if let Some(ref path) = self.path {
            let request_path = url.path();
            if request_path == path {
            } else if request_path.starts_with(path) {
                if !path.ends_with('/') && request_path.as_bytes().get(path.len()) != Some(&b'/') {
                    return false;
                }
            } else {
                return false;
            }
        }
        if self.secure && url.scheme() != "https" {
            return false;
        }
        true
    }
    pub fn to_header(&self) -> String {
        format!("{}={}", self.name, self.value)
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CookieJar {
    cookies: HashMap<String, Cookie>,
}
impl CookieJar {
    pub fn len(&self) -> usize {
        self.cookies.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_from_set_cookie(&mut self, header: &str, request_url: &url::Url) {
        if let Some(cookie) = Cookie::from_set_cookie(header, request_url) {
            debug!("Adding cookie: {}={}", cookie.name, cookie.value);
            self.cookies.insert(cookie.name.clone(), cookie);
        }
    }
    pub fn get_cookies_for_url(&self, url: &url::Url) -> Vec<&Cookie> {
        self.cookies.values().filter(|c| c.matches(url)).collect()
    }
    pub fn cookie_header(&self, url: &url::Url) -> String {
        let cookies = self.get_cookies_for_url(url);
        if cookies.is_empty() {
            return String::new();
        }
        cookies
            .iter()
            .map(|c| c.to_header())
            .collect::<Vec<_>>()
            .join("; ")
    }
    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let mut jar: Self = serde_json::from_str(&data)?;
        jar.clear_expired();
        Ok(jar)
    }
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.cookies.retain(|_, c| {
            if let Some(exp) = c.expires {
                now < exp
            } else {
                true
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;
    #[test]
    fn test_parse_set_cookie() {
        let url = Url::parse("http://example.com").unwrap();
        let header = "sessionid=abc123; Path=/; HttpOnly; Secure; SameSite=Lax";
        let cookie = Cookie::from_set_cookie(header, &url).unwrap();
        assert_eq!(cookie.name, "sessionid");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.path, Some("/".to_string()));
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.same_site, Some("lax".to_string()));
    }
    #[test]
    fn test_domain_matching_rfc() {
        let cookie = Cookie {
            name: "test".into(),
            value: "1".into(),
            domain: Some("example.com".into()),
            path: Some("/".into()),
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        };
        assert!(cookie.matches(&Url::parse("http://example.com/").unwrap()));
        assert!(cookie.matches(&Url::parse("http://sub.example.com/").unwrap()));
        assert!(!cookie.matches(&Url::parse("http://notexample.com/").unwrap()));
    }
    #[test]
    fn test_path_matching() {
        let cookie = Cookie {
            name: "test".into(),
            value: "1".into(),
            domain: None,
            path: Some("/foo".into()),
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        };
        assert!(cookie.matches(&Url::parse("http://x.com/foo").unwrap()));
        assert!(cookie.matches(&Url::parse("http://x.com/foo/bar").unwrap()));
        assert!(!cookie.matches(&Url::parse("http://x.com/foobar").unwrap()));
    }
    #[test]
    fn test_secure_only() {
        let cookie = Cookie {
            name: "test".into(),
            value: "1".into(),
            domain: None,
            path: None,
            expires: None,
            secure: true,
            http_only: false,
            same_site: None,
        };
        assert!(!cookie.matches(&Url::parse("http://x.com/").unwrap()));
        assert!(cookie.matches(&Url::parse("https://x.com/").unwrap()));
    }
    #[test]
    fn test_expired_cookie_rejected() {
        let cookie = Cookie {
            name: "test".into(),
            value: "1".into(),
            domain: None,
            path: None,
            expires: Some(SystemTime::now() - std::time::Duration::from_secs(3600)),
            secure: false,
            http_only: false,
            same_site: None,
        };
        assert!(!cookie.matches(&Url::parse("http://x.com/").unwrap()));
    }
    #[test]
    fn test_jar_clears_expired_on_load() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let jar = CookieJar {
            cookies: {
                let mut m = HashMap::new();
                m.insert(
                    "expired".into(),
                    Cookie {
                        name: "expired".into(),
                        value: "x".into(),
                        domain: None,
                        path: None,
                        expires: Some(SystemTime::now() - std::time::Duration::from_secs(1)),
                        secure: false,
                        http_only: false,
                        same_site: None,
                    },
                );
                m
            },
        };
        serde_json::to_writer(&mut tmp, &jar).unwrap();
        let path = tmp.path().to_str().unwrap();
        let loaded = CookieJar::load_from_file(path).unwrap();
        assert!(loaded.cookies.is_empty());
    }
    #[test]
    fn test_default_path_from_url() {
        let url = Url::parse("http://example.com/foo/bar").unwrap();
        let header = "c=1";
        let cookie = Cookie::from_set_cookie(header, &url).unwrap();
        assert_eq!(cookie.path.as_deref(), Some("/foo"));
    }
}
