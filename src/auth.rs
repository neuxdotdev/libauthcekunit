use crate::client::RobustHttpClient;
use crate::config::CONFIG;
use crate::cookies::CookieJar;
use crate::error::{AuthError, Result};
use crate::parser::extract_token;
use tracing::{debug, info, warn};
pub fn login(base_url: &str) -> Result<CookieJar> {
    info!("Starting login process to {}", base_url);
    let client = RobustHttpClient::new()?;
    let login_page_url = format!("{}/login", base_url);
    let (html, jar) = client.get(&login_page_url, &CookieJar::new())?;
    let token = extract_token(&html)?;
    debug!("CSRF token obtained (length: {})", token.len());
    let email = &CONFIG.email;
    let password = &CONFIG.password;
    if email.is_empty() || password.is_empty() {
        return Err(AuthError::Config(
            "LIB_CEKUNIT_AUTH_ENV_EMAIL and LIB_CEKUNIT_AUTH_ENV_PASSWORD must be set".into(),
        ));
    }
    let login_url = format!("{}/login", base_url);
    let params = [
        ("_token", token.as_str()),
        ("email", email),
        ("password", password),
    ];
    let (_body, final_jar) = client.post_form(&login_url, &params, &jar)?;
    debug!("Login POST completed");
    if let Err(e) = verify_session(&client, base_url, &final_jar) {
        warn!("Session verification failed: {}", e);
        return Err(e);
    }
    info!("Login successful, session established");
    Ok(final_jar)
}
fn verify_session(client: &RobustHttpClient, base_url: &str, jar: &CookieJar) -> Result<()> {
    let verify_url = format!("{}{}", base_url, CONFIG.login_verify_url.as_str());
    debug!("Verifying session by accessing {}", verify_url);
    let (body, _) = client.get(&verify_url, jar)?;
    if !CONFIG.login_verify_text.is_empty() && !body.contains(&CONFIG.login_verify_text) {
        return Err(AuthError::LoginFailed(
            "Protected page missing expected content".into(),
        ));
    }
    Ok(())
}
pub fn logout(base_url: &str, cookie_jar: &CookieJar) -> Result<CookieJar> {
    info!("Logging out from {}", base_url);
    let client = RobustHttpClient::new()?;
    let dashboard_url = format!("{}/dashboard", base_url);
    let (html, jar_with_csrf) = client.get(&dashboard_url, cookie_jar)?;
    let token = extract_token(&html)?;
    debug!("Logout CSRF token obtained");
    let logout_url = format!("{}/logout", base_url);
    let params = [("_token", token.as_str())];
    let (_body, final_jar) = client.post_form(&logout_url, &params, &jar_with_csrf)?;
    info!("Logout completed");
    Ok(final_jar)
}
