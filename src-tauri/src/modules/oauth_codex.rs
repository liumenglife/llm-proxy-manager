use serde::{Deserialize, Serialize};

const GITHUB_CLIENT_ID: &str = "Iv23liYYMXqioBpr4S8N";
const GITHUB_CLIENT_SECRET: &str = "";
const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_API_URL: &str = "https://api.github.com/user";
const TOKEN_REFRESH_SKEW_SECONDS: i64 = 900;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexUserInfo {
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn get_codex_auth_url(redirect_uri: &str, state: &str) -> Result<String, String> {
    let params = vec![
        ("client_id", GITHUB_CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", "read:user user:email"),
        ("state", state),
    ];

    url::Url::parse_with_params(AUTH_URL, &params)
        .map(|u| u.to_string())
        .map_err(|e| format!("Invalid Auth URL: {}", e))
}

pub async fn exchange_codex_code(code: &str, redirect_uri: &str) -> Result<CodexTokenResponse, String> {
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(None, 60).await
    } else {
        crate::utils::http::get_long_standard_client()
    };

    let params = [
        ("client_id", GITHUB_CLIENT_ID),
        ("client_secret", GITHUB_CLIENT_SECRET),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let response = client
        .post(TOKEN_URL)
        .header(rquest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
        .header(rquest::header::ACCEPT, "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexTokenResponse>()
            .await
            .map_err(|e| format!("Token parsing failed: {}", e))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token exchange failed ({}): {}", status, error_text))
    }
}

pub async fn refresh_codex_token(refresh_token: &str) -> Result<CodexTokenResponse, String> {
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(None, 60).await
    } else {
        crate::utils::http::get_long_standard_client()
    };

    let params = [
        ("client_id", GITHUB_CLIENT_ID),
        ("client_secret", GITHUB_CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let response = client
        .post(TOKEN_URL)
        .header(rquest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
        .header(rquest::header::ACCEPT, "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexTokenResponse>()
            .await
            .map_err(|e| format!("Refresh data parsing failed: {}", e))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Refresh failed ({}): {}", status, error_text))
    }
}

pub async fn ensure_fresh_codex_token(
    current_token: &crate::models::TokenData,
) -> Result<crate::models::TokenData, String> {
    let now = chrono::Local::now().timestamp();

    if current_token.expiry_timestamp > now + TOKEN_REFRESH_SKEW_SECONDS {
        return Ok(current_token.clone());
    }

    crate::modules::logger::log_info("Codex token expiring soon, refreshing...");
    let response = refresh_codex_token(&current_token.refresh_token).await?;

    let refresh_token = response
        .refresh_token
        .unwrap_or_else(|| current_token.refresh_token.clone());

    Ok(crate::models::TokenData::new(
        response.access_token,
        refresh_token,
        response.expires_in,
        current_token.email.clone(),
        None,
        None,
        false,
    ))
}

pub async fn get_codex_user_info(access_token: &str) -> Result<CodexUserInfo, String> {
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_client(None, 15).await
    } else {
        crate::utils::http::get_client()
    };

    let response = client
        .get(USER_API_URL)
        .bearer_auth(access_token)
        .header(rquest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexUserInfo>()
            .await
            .map_err(|e| format!("User info parsing failed: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Failed to get user info: {}", error_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_codex_auth_url_contains_params() {
        let redirect_uri = "http://localhost:8080/callback";
        let state = "test-state-123456";
        let url = get_codex_auth_url(redirect_uri, state).unwrap();

        assert!(url.contains("state=test-state-123456"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id="));
        assert!(url.contains("scope=read%3Auser+user%3Aemail"));
    }

    #[test]
    fn test_get_codex_auth_url_parseable() {
        let redirect_uri = "http://localhost:8080/callback";
        let state = "test-state";
        let url = get_codex_auth_url(redirect_uri, state).unwrap();

        let parsed = url::Url::parse(&url).expect("Should be valid URL");
        let query_params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

        assert_eq!(
            query_params.get("client_id").map(|s| s.as_ref()),
            Some(GITHUB_CLIENT_ID)
        );
        assert_eq!(
            query_params.get("response_type").map(|s| s.as_ref()),
            Some("code")
        );
        assert_eq!(
            query_params.get("redirect_uri").map(|s| s.as_ref()),
            Some(redirect_uri)
        );
        assert_eq!(
            query_params.get("state").map(|s| s.as_ref()),
            Some(state)
        );
        assert_eq!(
            query_params.get("scope").map(|s| s.as_ref()),
            Some("read:user user:email")
        );
    }
}
