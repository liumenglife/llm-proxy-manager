use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Codex CLI 使用的公开 OAuth Client ID（来自开源 Codex CLI 代码）
const DEFAULT_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZE_URL: &str = "https://auth.openai.com/authorize";
const TOKEN_URL: &str = "https://auth0.openai.com/oauth/token";
const USER_API_URL: &str = "https://api.openai.com/v1/me";
const TOKEN_REFRESH_SKEW_SECONDS: i64 = 480;

fn get_client_id() -> String {
    std::env::var("CODEX_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string())
}

fn base64url_encode(data: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn generate_pkce_pair() -> (String, String) {
    use rand::Rng;
    let verifier: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
    let verifier_b64 = base64url_encode(&verifier);
    let challenge = Sha256::digest(&verifier);
    let challenge_b64 = base64url_encode(&challenge);
    (verifier_b64, challenge_b64)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexUserInfo {
    pub email: Option<String>,
    pub name: Option<String>,
}

pub fn get_codex_auth_url(
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> Result<String, String> {
    let params = vec![
        ("client_id", get_client_id()),
        ("redirect_uri", redirect_uri.to_string()),
        ("response_type", "code"),
        ("scope", "openid profile email offline_access".to_string()),
        ("code_challenge", code_challenge.to_string()),
        ("code_challenge_method", "S256".to_string()),
        ("state", state.to_string()),
    ];

    let url = url::Url::parse_with_params(AUTHORIZE_URL, &params)
        .map_err(|e| format!("Invalid Auth URL: {}", e))?;
    Ok(url.to_string())
}

fn get_http_client() -> crate::utils::http::HttpClient {
    crate::utils::http::get_long_standard_client()
}

pub async fn exchange_codex_code(
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<CodexTokenResponse, String> {
    let client = get_http_client();

    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": get_client_id(),
        "code": code,
        "redirect_uri": redirect_uri,
        "code_verifier": code_verifier,
    });

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Codex token exchange failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexTokenResponse>()
            .await
            .map_err(|e| format!("Codex token parsing failed: {}", e))
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("Codex token exchange failed ({}): {}", status, text))
    }
}

pub async fn exchange_codex_for_api_key(
    id_token: &str,
) -> Result<String, String> {
    let client = get_http_client();

    let body = serde_json::json!({
        "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",
        "client_id": get_client_id(),
        "requested_token_type": "openai-api-key",
        "subject_token": id_token,
        "subject_token_type": "urn:ietf:params:oauth:token-type:id_token",
    });

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Codex API key exchange failed: {}", e))?;

    if response.status().is_success() {
        let data = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("API key response parsing failed: {}", e))?;

        data.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No access_token in API key response".to_string())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("API key exchange failed ({}): {}", status, text))
    }
}

pub async fn refresh_codex_token(refresh_token: &str) -> Result<CodexTokenResponse, String> {
    let client = get_http_client();

    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "client_id": get_client_id(),
        "refresh_token": refresh_token,
    });

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Codex token refresh failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexTokenResponse>()
            .await
            .map_err(|e| format!("Codex token refresh parsing failed: {}", e))
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("Codex token refresh failed ({}): {}", status, text))
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
    let client = get_http_client();

    let response = client
        .get(USER_API_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<CodexUserInfo>()
            .await
            .map_err(|e| format!("User info parsing failed: {}", e))
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("User info failed ({}): {}", status, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_codex_auth_url_contains_params() {
        let redirect_uri = "http://localhost:1455/auth/callback";
        let state = "test-state-123456";
        let (_, challenge) = generate_pkce_pair();
        let url = get_codex_auth_url(redirect_uri, state, &challenge).unwrap();

        assert!(url.contains("state=test-state-123456"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id="));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.starts_with("https://auth.openai.com/authorize"));
    }

    #[test]
    fn test_get_codex_auth_url_parseable() {
        let redirect_uri = "http://localhost:1455/auth/callback";
        let state = "test-state";
        let (_, challenge) = generate_pkce_pair();
        let url = get_codex_auth_url(redirect_uri, state, &challenge).unwrap();

        let parsed = url::Url::parse(&url).expect("Should be valid URL");
        let query_params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

        assert_eq!(query_params.get("response_type").map(|s| s.as_ref()), Some("code"));
        assert_eq!(query_params.get("code_challenge_method").map(|s| s.as_ref()), Some("S256"));
        assert!(query_params.contains_key("code_challenge"));
    }

    #[test]
    fn test_pkce_pair_generation() {
        let (verifier, challenge) = generate_pkce_pair();

        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());

        let expected_challenge = base64url_encode(&Sha256::digest(&verifier));
        assert_eq!(challenge, expected_challenge);
    }

    #[test]
    fn test_client_id_from_env_or_default() {
        std::env::remove_var("CODEX_CLIENT_ID");
        let id = get_client_id();
        assert_eq!(id, DEFAULT_CLIENT_ID);
    }
}
