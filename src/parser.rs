use crate::config::CONFIG;
use crate::error::{AuthError, Result};
use regex::Regex;
use tracing::debug;
pub fn extract_token(html: &str) -> Result<String> {
    debug!("Extracting token from HTML ({} bytes)", html.len());
    let re = Regex::new(r#"(?i)name=["']_token["']\s+value=["']([^"'<>]+)["']"#)?;
    if let Some(caps) = re.captures(html) {
        let token = caps[1].to_string();
        debug!("Regex found token: {}...", &token[..token.len().min(8)]);
        return validate_token(&token);
    }
    if let Some(token) = manual_extract(html) {
        debug!("Manual scan found token");
        return validate_token(&token);
    }
    Err(AuthError::TokenNotFound)
}
fn manual_extract(html: &str) -> Option<String> {
    let needle_name = "name=\"_token\"";
    let needle_name_single = "name='_token'";
    let value_pat = "value=";
    let mut start = 0;
    while let Some(pos) = html[start..].find(needle_name) {
        let abs = start + pos;
        if let Some(token) = extract_value_after(&html[abs..]) {
            return Some(token);
        }
        start = abs + 1;
    }
    start = 0;
    while let Some(pos) = html[start..].find(needle_name_single) {
        let abs = start + pos;
        if let Some(token) = extract_value_after(&html[abs..]) {
            return Some(token);
        }
        start = abs + 1;
    }
    let mut idx = 0;
    while let Some(val_pos) = html[idx..].find(value_pat) {
        let abs = val_pos + idx;
        let after = &html[abs + 6..];
        let quote = after.chars().next()?;
        if quote != '"' && quote != '\'' {
            idx = abs + 1;
            continue;
        }
        let end = after[1..].find(quote)?;
        let token = &after[1..=end];
        let ctx_start = abs.saturating_sub(200);
        let context = &html[ctx_start..abs];
        if context.contains("name=\"_token\"") || context.contains("name='_token'") {
            return Some(token.to_string());
        }
        idx = abs + 1;
    }
    None
}
fn extract_value_after(slice: &str) -> Option<String> {
    let value_pat = "value=";
    let pos = slice.find(value_pat)?;
    let after = &slice[pos + 6..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after[1..].find(quote)?;
    Some(after[1..=end].to_string())
}
fn validate_token(token: &str) -> Result<String> {
    let len = token.len();
    if len < CONFIG.token_min_length {
        return Err(AuthError::TokenInvalid {
            reason: format!("too short (min {})", CONFIG.token_min_length),
        });
    }
    if len > CONFIG.token_max_length {
        return Err(AuthError::TokenInvalid {
            reason: format!("too long (max {})", CONFIG.token_max_length),
        });
    }
    let valid = token.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '=' || c == '/'
    });
    if !valid {
        return Err(AuthError::TokenInvalid {
            reason: "contains invalid characters".into(),
        });
    }
    Ok(token.to_string())
}
