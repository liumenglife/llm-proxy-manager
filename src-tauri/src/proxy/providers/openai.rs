use super::{AiProvider, ProviderRequest, ProviderResponse};
use crate::models::TokenData;
use async_trait::async_trait;
use serde_json::Value;

pub struct OpenAIProvider;

#[async_trait]
impl AiProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn base_url(&self) -> &str {
        "https://api.openai.com/v1"
    }

    async fn send_request(&self, req: ProviderRequest) -> Result<ProviderResponse, String> {
        if req.account_id.trim().is_empty() {
            return Err("Missing Codex API key".to_string());
        }

        let mut body = req.body;
        if body
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .is_empty()
        {
            body["model"] = Value::String(req.model);
        }

        let response = rquest::Client::new()
            .post(format!("{}/responses", self.base_url()))
            .bearer_auth(req.account_id)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        let status = response.status().as_u16();
        let body = response
            .json::<Value>()
            .await
            .map_err(|e| format!("OpenAI response parsing failed: {}", e))?;

        Ok(ProviderResponse { status, body })
    }

    async fn refresh_token(&self, token: &TokenData) -> Result<TokenData, String> {
        crate::modules::oauth_codex::ensure_fresh_codex_token(token).await
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![
            "PasGPT-5-Codex",
            "GPT-5.1",
            "Codex GPT-5.1",
            "Codex Max GPT-5.1",
            "Codex mini",
            "GPT-5.2",
            "GPT-5.2 Codex",
            "GPT-5.3 Codex",
            "GPT-5.4",
            "GPT-5.4 Fast",
            "GPT-5.4 mini",
            "GPT-5.4 mini Fast",
            "GPT-5.5 Fast",
            "GPT-5.5 Pro",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn send_request_rejects_missing_api_key_without_panic() {
        let provider = OpenAIProvider;
        let req = ProviderRequest {
            model: "gpt-4o".to_string(),
            body: json!({"model": "gpt-4o", "input": "hello"}),
            account_id: String::new(),
        };

        let result = provider.send_request(req).await;

        assert_eq!(result.unwrap_err(), "Missing Codex API key");
    }
}
